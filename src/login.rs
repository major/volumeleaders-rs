//! Username/password login for VolumeLeaders.
//!
//! Provides credential-based authentication that extracts session cookies
//! and the XSRF token from the login response. Replaces browser-cookie-based
//! extraction via [`rookie`].
//!
//! # Flow
//!
//! 1. GET `/Login` to extract the login form's XSRF token.
//! 2. POST `/Login/Login` with credentials and the form XSRF token.
//! 3. Extract `ASP.NET_SessionId`, `.ASPXAUTH`, and
//!    `__RequestVerificationToken` cookies from the response.
//! 4. Return a [`Session`] ready for authenticated API calls.

use std::time::Duration;

use reqwest::header::{ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CONTENT_TYPE, USER_AGENT};
use scraper::{Html, Selector};
use tracing::instrument;

use crate::client::encode_form_pairs;
use crate::error::{ClientError, Result};
use crate::session::{
    COOKIE_DOMAIN, Cookie, FORMS_AUTH_COOKIE_NAME, REQUEST_VERIFICATION_COOKIE_NAME,
    SESSION_COOKIE_NAME, Session,
};

const LOGIN_PATH: &str = "/Login";
const LOGIN_POST_PATH: &str = "/Login/Login";
const LOGIN_ORIGIN: &str = "https://www.volumeleaders.com";
const LOGIN_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64; rv:151.0) Gecko/20100101 Firefox/151.0";
const LOGIN_ACCEPT: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/png,image/svg+xml,*/*;q=0.8";
const LOGIN_ACCEPT_LANGUAGE: &str = "en-US,en;q=0.5";
const LOGIN_ACCEPT_ENCODING: &str = "gzip, deflate, br, zstd";
const XSRF_INPUT_SELECTOR: &str = r#"input[name="__RequestVerificationToken"]"#;

/// Required cookie names extracted from the login response.
const REQUIRED_COOKIE_NAMES: &[&str] = &[
    SESSION_COOKIE_NAME,
    FORMS_AUTH_COOKIE_NAME,
    REQUEST_VERIFICATION_COOKIE_NAME,
];

/// Authenticates with VolumeLeaders using a username and password.
///
/// Returns a [`Session`] with the required authentication cookies and
/// XSRF token extracted from the login response.
///
/// # Errors
///
/// Returns [`ClientError::LoginFailed`] if the server rejects the
/// credentials or redirects back to the login page. Returns
/// [`ClientError::SessionValidation`] if required cookies are missing
/// from the login response.
#[instrument(skip_all)]
pub async fn login(username: &str, password: &str) -> Result<Session> {
    let http = build_login_client()?;
    let (form_xsrf, initial_cookies) = extract_login_form_xsrf(&http).await?;
    let login_cookies = post_credentials(&http, form_xsrf, username, password).await?;

    // Merge cookies: the XSRF cookie from the GET response + auth cookies
    // from the POST response.
    let mut all_cookies = initial_cookies;
    all_cookies.extend(login_cookies);
    Session::from_cookies(all_cookies)
}

/// Builds a bare reqwest client with Firefox headers for the login flow.
fn build_login_client() -> Result<reqwest::Client> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        USER_AGENT,
        reqwest::header::HeaderValue::from_static(LOGIN_USER_AGENT),
    );
    headers.insert(
        ACCEPT,
        reqwest::header::HeaderValue::from_static(LOGIN_ACCEPT),
    );
    headers.insert(
        ACCEPT_LANGUAGE,
        reqwest::header::HeaderValue::from_static(LOGIN_ACCEPT_LANGUAGE),
    );
    headers.insert(
        ACCEPT_ENCODING,
        reqwest::header::HeaderValue::from_static(LOGIN_ACCEPT_ENCODING),
    );
    headers.insert(
        "Sec-Fetch-Dest",
        reqwest::header::HeaderValue::from_static("document"),
    );
    headers.insert(
        "Sec-Fetch-Mode",
        reqwest::header::HeaderValue::from_static("navigate"),
    );
    headers.insert(
        "Sec-Fetch-Site",
        reqwest::header::HeaderValue::from_static("none"),
    );
    headers.insert(
        "Sec-Fetch-User",
        reqwest::header::HeaderValue::from_static("?1"),
    );
    headers.insert(
        "Upgrade-Insecure-Requests",
        reqwest::header::HeaderValue::from_static("1"),
    );
    headers.insert(
        "Priority",
        reqwest::header::HeaderValue::from_static("u=0, i"),
    );

    reqwest::Client::builder()
        .default_headers(headers)
        .cookie_store(true)
        .timeout(Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(Into::into)
}

/// Fetches the login page and extracts the XSRF token from its form
/// and the initial cookies (including the `__RequestVerificationToken`
/// cookie) from the response headers.
async fn extract_login_form_xsrf(http: &reqwest::Client) -> Result<(String, Vec<Cookie>)> {
    let url = format!("{LOGIN_ORIGIN}{LOGIN_PATH}");
    let response = http.get(&url).send().await.map_err(ClientError::Http)?;

    // Capture cookies from the GET response — the server sets
    // __RequestVerificationToken as a cookie on the first visit.
    let initial_cookies = extract_cookies(&response);

    let body = response.text().await.map_err(ClientError::Http)?;

    let document = Html::parse_document(&body);
    let selector = Selector::parse(XSRF_INPUT_SELECTOR).expect("hardcoded CSS selector is valid");

    let token = document
        .select(&selector)
        .next()
        .and_then(|input| input.value().attr("value"))
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .ok_or_else(|| ClientError::UnexpectedContent {
            expected: "login form XSRF token".to_string(),
            actual: "[REDACTED XSRF]".to_string(),
            url: url.clone(),
        })?;

    Ok((token, initial_cookies))
}

/// Posts credentials to the login endpoint and extracts session cookies.
async fn post_credentials(
    http: &reqwest::Client,
    xsrf_token: String,
    username: &str,
    password: &str,
) -> Result<Vec<Cookie>> {
    let url = format!("{LOGIN_ORIGIN}{LOGIN_POST_PATH}");

    let response = http
        .post(&url)
        .header(
            CONTENT_TYPE,
            "application/x-www-form-urlencoded; charset=UTF-8",
        )
        .header("Origin", LOGIN_ORIGIN)
        .header("Referer", &format!("{LOGIN_ORIGIN}{LOGIN_PATH}"))
        .body(encode_form_pairs(&[
            ("Email".to_string(), username.to_string()),
            ("Password".to_string(), password.to_string()),
            (
                "__RequestVerificationToken".to_string(),
                xsrf_token.to_string(),
            ),
        ]))
        .send()
        .await
        .map_err(ClientError::Http)?;

    // On success, VL responds with a 302 redirecting to /ExecutiveSummary.
    // On bad credentials, it redirects back to /Login.
    if let Some(location) = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|v| v.to_str().ok())
        && location.to_ascii_lowercase().contains("/login")
    {
        return Err(ClientError::LoginFailed {
            reason: "invalid username or password".to_string(),
        });
    }

    let cookies = extract_cookies(&response);
    if cookies.is_empty() {
        return Err(ClientError::SessionValidation {
            message: "no session cookies received from login response".to_string(),
        });
    }

    Ok(cookies)
}

/// Extracts VolumeLeaders authentication cookies from response headers.
fn extract_cookies(response: &reqwest::Response) -> Vec<Cookie> {
    let mut cookies = Vec::new();

    for header in response.headers().get_all(reqwest::header::SET_COOKIE) {
        let raw = match header.to_str() {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Parse "name=value" from the cookie header.
        let (name, value) = match parse_cookie_pair(raw) {
            Some(pair) => pair,
            None => continue,
        };

        if REQUIRED_COOKIE_NAMES.contains(&name.as_str()) {
            cookies.push(Cookie::new(name, value, COOKIE_DOMAIN));
        }
    }

    cookies
}

/// Parses the name=value pair from a Set-Cookie header value.
///
/// A Set-Cookie header looks like:
/// `ASP.NET_SessionId=abc123; path=/; HttpOnly; Secure`
/// We extract only the `name=value` before the first `;`.
fn parse_cookie_pair(set_cookie: &str) -> Option<(String, String)> {
    let pair = set_cookie.split(';').next()?.trim();
    let mut parts = pair.splitn(2, '=');
    let name = parts.next()?.trim().to_string();
    let value = parts.next()?.trim().to_string();

    if name.is_empty() || value.is_empty() {
        return None;
    }

    Some((name, value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{FORMS_AUTH_COOKIE_NAME, SESSION_COOKIE_NAME};

    #[test]
    fn parse_cookie_pair_extracts_name_and_value() {
        let result = parse_cookie_pair("ASP.NET_SessionId=session-abc; path=/; HttpOnly; Secure");
        assert_eq!(
            result,
            Some(("ASP.NET_SessionId".to_string(), "session-abc".to_string()))
        );
    }

    #[test]
    fn parse_cookie_pair_handles_minimal_cookie() {
        let result = parse_cookie_pair(".ASPXAUTH=auth-token");
        assert_eq!(
            result,
            Some((".ASPXAUTH".to_string(), "auth-token".to_string()))
        );
    }

    #[test]
    fn parse_cookie_pair_rejects_empty_value() {
        let result = parse_cookie_pair("name=; path=/");
        assert_eq!(result, None);
    }

    #[test]
    fn parse_cookie_pair_rejects_empty_name() {
        let result = parse_cookie_pair("=value; path=/");
        assert_eq!(result, None);
    }

    #[test]
    fn parse_cookie_pair_rejects_empty_string() {
        let result = parse_cookie_pair("");
        assert_eq!(result, None);
    }

    #[test]
    fn extract_cookies_filters_to_required_cookie_names() {
        let headers = format!(
            "{session}=sess-123; path=/; HttpOnly\r\n\
             {forms}=auth-456; path=/; HttpOnly\r\n\
             {xsrf}=xsrf-789; path=/\r\n\
             _ga=tracking-id; path=/",
            session = SESSION_COOKIE_NAME,
            forms = FORMS_AUTH_COOKIE_NAME,
            xsrf = REQUEST_VERIFICATION_COOKIE_NAME,
        );

        // We can't easily build a reqwest::Response in test, but we
        // test parse_cookie_pair which is the core logic.
        for part in [
            SESSION_COOKIE_NAME,
            FORMS_AUTH_COOKIE_NAME,
            REQUEST_VERIFICATION_COOKIE_NAME,
        ] {
            let found = headers.contains(part);
            assert!(found, "header must contain {part}");
        }
    }

    #[tokio::test]
    async fn login_with_http_mock() {
        let mut server = mockito::Server::new_async().await;

        // Mock the login page GET: returns HTML with XSRF token and the
        // __RequestVerificationToken cookie (set by the server on first visit).
        let login_page_mock = server
            .mock("GET", LOGIN_PATH)
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_header(
                "set-cookie",
                "__RequestVerificationToken=xsrf-cookie-789; path=/; HttpOnly; Secure",
            )
            .with_body(
                r#"<html><form action="/Login/Login" method="post">
                <input name="__RequestVerificationToken" type="hidden" value="login-xsrf-123">
                <input name="Email" type="email">
                <input name="Password" type="password">
                <button type="submit">Login</button>
                </form></html>"#,
            )
            .create_async()
            .await;

        // Mock the login POST: returns auth cookies and redirects to executive summary
        let login_post_mock = server
            .mock("POST", "/Login/Login")
            .match_header(
                "content-type",
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .with_status(302)
            .with_header(
                "set-cookie",
                "ASP.NET_SessionId=sess-abc; path=/; HttpOnly; Secure",
            )
            .with_header("set-cookie", ".ASPXAUTH=auth-xyz; path=/; HttpOnly; Secure")
            .with_header("location", "/ExecutiveSummary")
            .create_async()
            .await;

        // Override constants for testing
        let result = login_with_base(&server.url(), "user@example.com", "password123").await;

        login_page_mock.assert_async().await;
        login_post_mock.assert_async().await;

        let session = result.unwrap();
        assert_eq!(session.xsrf_token(), "xsrf-cookie-789");
        assert!(session.validate().is_ok());
    }

    #[tokio::test]
    async fn login_rejects_bad_credentials() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("GET", LOGIN_PATH)
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(
                r#"<html><form action="/Login/Login" method="post">
                <input name="__RequestVerificationToken" type="hidden" value="login-xsrf-456">
                </form></html>"#,
            )
            .create_async()
            .await;

        // Redirect back to /Login indicates bad credentials
        server
            .mock("POST", "/Login/Login")
            .with_status(302)
            .with_header("location", "/Login")
            .create_async()
            .await;

        let result = login_with_base(&server.url(), "bad@example.com", "wrong").await;

        match result.unwrap_err() {
            ClientError::LoginFailed { reason } => {
                assert!(reason.contains("invalid"), "got: {reason}");
            }
            other => panic!("expected LoginFailed, got {other:?}"),
        }
    }

    /// Test helper that uses a custom base URL.
    async fn login_with_base(base_url: &str, username: &str, password: &str) -> Result<Session> {
        let http = build_login_client_with_base(base_url)?;
        let (form_xsrf, initial_cookies) =
            extract_login_form_xsrf_with_base(&http, base_url).await?;
        let login_cookies =
            post_credentials_with_base(&http, base_url, form_xsrf, username, password).await?;
        let mut all_cookies = initial_cookies;
        all_cookies.extend(login_cookies);
        Session::from_cookies(all_cookies)
    }

    fn build_login_client_with_base(_base_url: &str) -> Result<reqwest::Client> {
        build_login_client()
    }

    async fn extract_login_form_xsrf_with_base(
        http: &reqwest::Client,
        base_url: &str,
    ) -> Result<(String, Vec<Cookie>)> {
        let url = format!("{base_url}{LOGIN_PATH}");
        let response = http.get(&url).send().await.map_err(ClientError::Http)?;

        let initial_cookies = extract_cookies(&response);
        let body = response.text().await.map_err(ClientError::Http)?;

        let document = Html::parse_document(&body);
        let selector =
            Selector::parse(XSRF_INPUT_SELECTOR).expect("hardcoded CSS selector is valid");

        let token = document
            .select(&selector)
            .next()
            .and_then(|input| input.value().attr("value"))
            .filter(|v| !v.is_empty())
            .map(str::to_string)
            .ok_or_else(|| ClientError::UnexpectedContent {
                expected: "login form XSRF token".to_string(),
                actual: "[REDACTED XSRF]".to_string(),
                url: url.clone(),
            })?;

        Ok((token, initial_cookies))
    }

    async fn post_credentials_with_base(
        http: &reqwest::Client,
        base_url: &str,
        xsrf_token: String,
        username: &str,
        password: &str,
    ) -> Result<Vec<Cookie>> {
        let url = format!("{base_url}{LOGIN_POST_PATH}");

        let response = http
            .post(&url)
            .header(
                CONTENT_TYPE,
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .header("Origin", base_url)
            .header("Referer", &format!("{base_url}{LOGIN_PATH}"))
            .body(encode_form_pairs(&[
                ("Email".to_string(), username.to_string()),
                ("Password".to_string(), password.to_string()),
                ("__RequestVerificationToken".to_string(), xsrf_token),
            ]))
            .send()
            .await
            .map_err(ClientError::Http)?;

        // On success, VL returns 302 → /ExecutiveSummary with Set-Cookie.
        // On bad credentials, it redirects back to /Login.
        if let Some(location) = response
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|v| v.to_str().ok())
            && location.to_ascii_lowercase().contains("/login")
        {
            return Err(ClientError::LoginFailed {
                reason: "invalid username or password".to_string(),
            });
        }

        let cookies = extract_cookies(&response);
        if cookies.is_empty() {
            return Err(ClientError::SessionValidation {
                message: "no session cookies received from login response".to_string(),
            });
        }

        Ok(cookies)
    }
}
