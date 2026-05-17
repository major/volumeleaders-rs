//! Authenticated HTTP client for VolumeLeaders browser-session requests.
//!
//! The client intentionally avoids reqwest's cookie jar. VolumeLeaders sessions
//! come from a browser profile, so every request injects the caller-supplied
//! cookies by header and POST requests add the XSRF token explicitly.

use std::error::Error as StdError;
use std::time::Duration;

use reqwest::header::{
    ACCEPT, ACCEPT_LANGUAGE, CONTENT_TYPE, COOKIE, HeaderMap, HeaderValue, USER_AGENT,
};
use serde::Serialize;
use tracing::instrument;
use url::Url;

use crate::error::{ClientError, Result};
use crate::session::Session;

const DEFAULT_BASE_URL: &str = "https://www.volumeleaders.com";
const DEFAULT_BODY_LIMIT: usize = 10 * 1024 * 1024;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36";
const MAX_REDIRECTS: usize = 10;
const PASSWORD_INPUT_MARKER: &str = r#"<input type="password"#;
const X_REQUESTED_WITH: &str = "X-Requested-With";
const XSRF_HEADER: &str = "X-XSRF-TOKEN";

/// HTTP client configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientConfig {
    /// VolumeLeaders origin used to resolve relative request paths.
    pub base_url: String,
    /// Maximum response body size read into memory.
    pub body_limit: usize,
    /// Per-request HTTP timeout.
    pub timeout: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            body_limit: DEFAULT_BODY_LIMIT,
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

/// Browser-session-backed VolumeLeaders HTTP client.
#[derive(Clone, Debug)]
pub struct Client {
    session: Session,
    http: reqwest::Client,
    base_url: Url,
    config: ClientConfig,
}

impl Client {
    /// Creates a client using default HTTP configuration.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError::SessionValidation`] when required session
    /// material is missing, or [`ClientError::Http`] if the reqwest client
    /// cannot be built.
    pub fn new(session: Session) -> Result<Self> {
        Self::with_config(session, ClientConfig::default())
    }

    /// Creates a client from explicit configuration.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError::SessionValidation`] when required session
    /// material is missing, [`ClientError::UnexpectedContent`] when the base URL
    /// is invalid, or [`ClientError::Http`] if the reqwest client cannot be
    /// built.
    pub fn with_config(session: Session, config: ClientConfig) -> Result<Self> {
        session.validate()?;
        let base_url = parse_base_url(&config.base_url)?;
        let http = reqwest::Client::builder()
            .default_headers(default_headers())
            .timeout(config.timeout)
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                if attempt.previous().len() >= MAX_REDIRECTS {
                    return attempt.error(format!("stopped after {MAX_REDIRECTS} redirects"));
                }

                let Some(first) = attempt.previous().first() else {
                    return attempt.follow();
                };

                if same_origin(first, attempt.url()) {
                    return attempt.follow();
                }

                let from = first.to_string();
                let to = attempt.url().to_string();
                attempt.error(ClientError::CrossOriginRedirect { from, to })
            }))
            .build()?;

        Ok(Self {
            session,
            http,
            base_url,
            config,
        })
    }

    /// Sends an authenticated GET and returns the response body.
    #[instrument(skip_all)]
    pub(crate) async fn get(&self, path: &str) -> Result<String> {
        let url = self.resolve(path);
        let response = self
            .http
            .get(url)
            .header(COOKIE, self.cookie_header())
            .send()
            .await
            .map_err(map_reqwest_error)?;

        self.response_text(response).await
    }

    /// Sends an authenticated URL-encoded form POST and returns the response body.
    #[instrument(skip_all)]
    pub(crate) async fn post_form(
        &self,
        path: &str,
        pairs: Vec<(String, String)>,
    ) -> Result<String> {
        let url = self.resolve(path);
        let response = self
            .http
            .post(url)
            .header(COOKIE, self.cookie_header())
            .header(XSRF_HEADER, self.session.xsrf_token())
            .header(X_REQUESTED_WITH, "XMLHttpRequest")
            .header(
                CONTENT_TYPE,
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .header(ACCEPT, "application/json, text/javascript, */*; q=0.01")
            .body(encode_form_pairs(&pairs))
            .send()
            .await
            .map_err(map_reqwest_error)?;

        self.response_text(response).await
    }

    /// Sends an authenticated multipart form POST and treats the expected
    /// redirect target as success.
    #[instrument(skip_all)]
    pub(crate) async fn post_multipart_form(
        &self,
        path: &str,
        form: reqwest::multipart::Form,
        success_redirect: &str,
    ) -> Result<()> {
        let url = self.resolve(path);
        let response = self
            .http
            .post(url)
            .header(COOKIE, self.cookie_header())
            .header(
                ACCEPT,
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            )
            .multipart(form)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        if response.url().path().ends_with(success_redirect) {
            return Ok(());
        }

        if redirect_location_matches(&response, success_redirect) {
            return Ok(());
        }

        self.response_text(response).await.map(|_| ())
    }

    /// Sends an authenticated JSON POST and returns the response body.
    #[instrument(skip_all)]
    pub(crate) async fn post_json<T>(&self, path: &str, payload: &T) -> Result<String>
    where
        T: Serialize + ?Sized,
    {
        let url = self.resolve(path);
        let body = serde_json::to_string(payload)?;
        let response = self
            .http
            .post(url)
            .header(COOKIE, self.cookie_header())
            .header(XSRF_HEADER, self.session.xsrf_token())
            .header(X_REQUESTED_WITH, "XMLHttpRequest")
            .header(CONTENT_TYPE, "application/json; charset=UTF-8")
            .header(ACCEPT, "application/json, text/javascript, */*; q=0.01")
            .body(body)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        self.response_text(response).await
    }

    fn resolve(&self, path: &str) -> Url {
        let mut url = self.base_url.clone();
        let base_path = self.base_url.path().trim_end_matches('/');
        let request_path = path.trim_start_matches('/');
        url.set_path(&format!("{base_path}/{request_path}"));
        url.set_query(None);
        url
    }

    fn cookie_header(&self) -> String {
        self.session
            .cookies()
            .iter()
            .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
            .collect::<Vec<_>>()
            .join("; ")
    }

    async fn response_text(&self, response: reqwest::Response) -> Result<String> {
        let status = response.status();
        let url = response.url().clone();

        if redirected_to_login(&url) {
            return Err(ClientError::SessionExpired {
                url: url.to_string(),
            });
        }

        let body = read_limited_body(response, self.config.body_limit).await?;
        if body.contains(PASSWORD_INPUT_MARKER) {
            return Err(ClientError::SessionExpired {
                url: url.to_string(),
            });
        }

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ClientError::RateLimit { retry_after: None });
        }

        if !status.is_success() {
            return Err(ClientError::Status {
                code: status.as_u16(),
                url: url.to_string(),
                body,
            });
        }

        Ok(body)
    }
}

fn parse_base_url(raw: &str) -> Result<Url> {
    let url = Url::parse(raw).map_err(|error| ClientError::UnexpectedContent {
        expected: "absolute base URL".to_string(),
        actual: error.to_string(),
        url: raw.to_string(),
    })?;
    if url.scheme().is_empty() || url.host_str().is_none() {
        return Err(ClientError::UnexpectedContent {
            expected: "absolute base URL".to_string(),
            actual: raw.to_string(),
            url: raw.to_string(),
        });
    }
    Ok(url)
}

fn default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
    );
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
    headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("document"));
    headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("navigate"));
    headers.insert("Sec-Fetch-Site", HeaderValue::from_static("same-origin"));
    headers.insert(USER_AGENT, HeaderValue::from_static(DEFAULT_USER_AGENT));
    headers
}

fn same_origin(first: &Url, next: &Url) -> bool {
    first.scheme() == next.scheme()
        && first.host_str() == next.host_str()
        && first.port_or_known_default() == next.port_or_known_default()
}

fn redirected_to_login(url: &Url) -> bool {
    url.path().to_ascii_lowercase().contains("/login")
}

fn redirect_location_matches(response: &reqwest::Response, success_redirect: &str) -> bool {
    response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|location| response.url().join(location).ok())
        .is_some_and(|url| url.path().ends_with(success_redirect))
}

async fn read_limited_body(mut response: reqwest::Response, limit: usize) -> Result<String> {
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        body.extend_from_slice(&chunk);
        if body.len() > limit {
            return Err(ClientError::BodyLimit {
                limit,
                actual: body.len(),
            });
        }
    }

    Ok(String::from_utf8_lossy(&body).into_owned())
}

fn encode_form_pairs(pairs: &[(String, String)]) -> String {
    pairs
        .iter()
        .map(|(key, value)| format!("{key}={}", encode_form_value(value)))
        .collect::<Vec<_>>()
        .join("&")
}

fn encode_form_value(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(char::from(byte));
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push('%');
                encoded.push(hex_digit(byte >> 4));
                encoded.push(hex_digit(byte & 0x0f));
            }
        }
    }
    encoded
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'A' + value - 10),
        _ => unreachable!("nibble values are always in 0..=15"),
    }
}

fn map_reqwest_error(error: reqwest::Error) -> ClientError {
    let mut source = StdError::source(&error);
    while let Some(err) = source {
        if let Some(ClientError::CrossOriginRedirect { from, to }) =
            err.downcast_ref::<ClientError>()
        {
            return ClientError::CrossOriginRedirect {
                from: from.clone(),
                to: to.clone(),
            };
        }
        source = err.source();
    }

    ClientError::Http(error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{COOKIE_DOMAIN, Cookie, FORMS_AUTH_COOKIE_NAME, SESSION_COOKIE_NAME};

    fn valid_session() -> Session {
        Session::new(
            vec![
                Cookie::new(SESSION_COOKIE_NAME, "session-123", COOKIE_DOMAIN),
                Cookie::new(FORMS_AUTH_COOKIE_NAME, "auth-456", COOKIE_DOMAIN),
            ],
            "xsrf-789",
        )
    }

    fn test_config(server: &mockito::Server) -> ClientConfig {
        ClientConfig {
            base_url: server.url(),
            body_limit: DEFAULT_BODY_LIMIT,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    #[tokio::test]
    async fn client_get_success_injects_cookies() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/endpoint")
            .match_header(
                "cookie",
                "ASP.NET_SessionId=session-123; .ASPXAUTH=auth-456",
            )
            .with_status(200)
            .with_body("ok")
            .create_async()
            .await;
        let client = Client::with_config(valid_session(), test_config(&server)).unwrap();

        let body = client.get("/endpoint").await.unwrap();

        assert_eq!(body, "ok");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn client_post_success_encodes_form_and_injects_xsrf() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/submit")
            .match_header(
                "cookie",
                "ASP.NET_SessionId=session-123; .ASPXAUTH=auth-456",
            )
            .match_header("x-xsrf-token", "xsrf-789")
            .match_header(
                "content-type",
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .match_body("name=A+B&columns[0][data]=x%2Fy")
            .with_status(200)
            .with_body("{}")
            .create_async()
            .await;
        let client = Client::with_config(valid_session(), test_config(&server)).unwrap();

        let body = client
            .post_form(
                "/submit",
                vec![
                    ("name".to_string(), "A B".to_string()),
                    ("columns[0][data]".to_string(), "x/y".to_string()),
                ],
            )
            .await
            .unwrap();

        assert_eq!(body, "{}");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn client_body_limit_exceeded_returns_body_limit() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/large")
            .with_status(200)
            .with_body("12345")
            .create_async()
            .await;
        let mut config = test_config(&server);
        config.body_limit = 4;
        let client = Client::with_config(valid_session(), config).unwrap();

        let error = client.get("/large").await.unwrap_err();

        match error {
            ClientError::BodyLimit { limit, actual } => {
                assert_eq!(limit, 4);
                assert!(actual > limit);
            }
            other => panic!("expected BodyLimit, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn client_session_expired_on_login_redirect() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/endpoint")
            .with_status(302)
            .with_header("location", "/login")
            .create_async()
            .await;
        server
            .mock("GET", "/login")
            .with_status(200)
            .with_body(r#"<html><input type="password" /></html>"#)
            .create_async()
            .await;
        let client = Client::with_config(valid_session(), test_config(&server)).unwrap();

        let error = client.get("/endpoint").await.unwrap_err();

        assert!(matches!(error, ClientError::SessionExpired { .. }));
    }

    #[tokio::test]
    async fn client_session_expired_on_password_input_body() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/expired")
            .with_status(200)
            .with_body(r#"<form><input type="password" name="Password"></form>"#)
            .create_async()
            .await;
        let client = Client::with_config(valid_session(), test_config(&server)).unwrap();

        let error = client.get("/expired").await.unwrap_err();

        assert!(matches!(error, ClientError::SessionExpired { .. }));
    }

    #[tokio::test]
    async fn client_cross_origin_redirect_blocked() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/endpoint")
            .with_status(302)
            .with_header("location", "https://example.com/login")
            .create_async()
            .await;
        let client = Client::with_config(valid_session(), test_config(&server)).unwrap();

        let error = client.get("/endpoint").await.unwrap_err();

        match error {
            ClientError::CrossOriginRedirect { from, .. } => {
                assert!(from.ends_with("/endpoint"));
            }
            other => panic!("expected CrossOriginRedirect, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn client_post_multipart_success_redirect() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("POST", "/save")
            .with_status(302)
            .with_header("location", "/success")
            .create_async()
            .await;
        server
            .mock("GET", "/success")
            .with_status(200)
            .with_body("saved")
            .create_async()
            .await;
        let client = Client::with_config(valid_session(), test_config(&server)).unwrap();

        client
            .post_multipart_form(
                "/save",
                reqwest::multipart::Form::new().text("name", "value"),
                "/success",
            )
            .await
            .unwrap();
    }
}
