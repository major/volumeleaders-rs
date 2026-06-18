//! Server-side authentication helpers for VolumeLeaders.
//!
//! Provides XSRF token extraction from the authenticated
//! ExecutiveSummary page and login page detection.

use scraper::{Html, Selector};
use tracing::instrument;

use crate::client::Client;
use crate::error::{ClientError, Result};
use crate::session::Session;

/// Path used to fetch the XSRF token from the authenticated site.
const EXECUTIVE_SUMMARY_PATH: &str = "/ExecutiveSummary";

/// HTML input name containing the ASP.NET request verification token.
const XSRF_INPUT_NAME: &str = "__RequestVerificationToken";

/// Fetches the hidden XSRF token from the authenticated ExecutiveSummary page.
///
/// Sends a GET request to `/ExecutiveSummary` using the client's session
/// cookies and parses the `__RequestVerificationToken` hidden input from
/// the HTML response.
///
/// # Errors
///
/// Returns [`ClientError::SessionExpired`] if the server redirects to a
/// login page, or [`ClientError::UnexpectedContent`] if the HTML does not
/// contain the expected hidden input.
#[instrument(skip_all)]
pub async fn extract_xsrf_token(client: &Client) -> Result<String> {
    let body = client.get(EXECUTIVE_SUMMARY_PATH).await?;
    parse_xsrf_token(&body)
}

/// Returns `true` if the URL or body indicates a login page.
///
/// Checks whether the URL path contains a `login` segment (case-insensitive) or
/// the body contains a password input field.
pub fn is_login_page(url: &str, body: &str) -> bool {
    url_has_login_segment(url) || body.contains(r#"type="password""#)
}

fn url_has_login_segment(url: &str) -> bool {
    url::Url::parse(url)
        .ok()
        .and_then(|url| {
            url.path_segments()
                .map(|mut segments| segments.any(|segment| segment.eq_ignore_ascii_case("login")))
        })
        .unwrap_or_else(|| {
            url.split(['?', '#'])
                .next()
                .unwrap_or(url)
                .split('/')
                .any(|segment| segment.eq_ignore_ascii_case("login"))
        })
}

/// Rebuilds a session with a fresh XSRF token from the server.
///
/// Calls [`extract_xsrf_token`] and creates a new [`Session`] with
/// the existing cookies and the updated token.
///
/// # Errors
///
/// Returns any error from [`extract_xsrf_token`].
#[instrument(skip_all)]
pub async fn refresh_session(client: &Client, session: &Session) -> Result<Session> {
    let xsrf_token = extract_xsrf_token(client).await?;
    Ok(Session::new(session.cookies().to_vec(), xsrf_token))
}

/// Parses the XSRF token value from an HTML document.
///
/// Looks for `<input name="__RequestVerificationToken" value="...">` and
/// returns the value attribute. The token value is never included in error
/// messages.
fn parse_xsrf_token(html: &str) -> Result<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse(r#"input[name="__RequestVerificationToken"]"#)
        .expect("hardcoded CSS selector is valid");

    let input =
        document
            .select(&selector)
            .next()
            .ok_or_else(|| ClientError::UnexpectedContent {
                expected: format!("hidden {XSRF_INPUT_NAME} input"),
                actual: "[REDACTED XSRF]".to_string(),
                url: EXECUTIVE_SUMMARY_PATH.to_string(),
            })?;

    let value = input.value().attr("value").unwrap_or("");
    if value.is_empty() {
        return Err(ClientError::UnexpectedContent {
            expected: format!("non-empty {XSRF_INPUT_NAME} value"),
            actual: "[REDACTED XSRF]".to_string(),
            url: EXECUTIVE_SUMMARY_PATH.to_string(),
        });
    }

    Ok(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ClientConfig;
    use crate::test_support::test_session;

    /// Creates a test client config pointing at the mock server.
    fn test_config(server: &mockito::Server) -> ClientConfig {
        ClientConfig {
            base_url: server.url(),
            ..ClientConfig::default()
        }
    }

    // --- parse_xsrf_token unit tests ---

    #[test]
    fn parse_xsrf_token_extracts_value() {
        let html = r#"<html><input name="__RequestVerificationToken" type="hidden" value="xsrf-123"></html>"#;

        assert_eq!(parse_xsrf_token(html).unwrap(), "xsrf-123");
    }

    #[test]
    fn parse_xsrf_token_handles_reversed_attributes() {
        let html = r#"<input type="hidden" value="xsrf-456" name="__RequestVerificationToken">"#;

        assert_eq!(parse_xsrf_token(html).unwrap(), "xsrf-456");
    }

    #[test]
    fn parse_xsrf_token_rejects_missing_input() {
        let html = "<html><body>no token here</body></html>";

        let err = parse_xsrf_token(html).unwrap_err();
        assert!(matches!(err, ClientError::UnexpectedContent { .. }));
    }

    #[test]
    fn parse_xsrf_token_rejects_empty_value() {
        let html = r#"<input name="__RequestVerificationToken" value="">"#;

        let err = parse_xsrf_token(html).unwrap_err();
        assert!(matches!(err, ClientError::UnexpectedContent { .. }));
    }

    #[test]
    fn parse_xsrf_token_rejects_missing_value_attribute() {
        let html = r#"<input name="__RequestVerificationToken">"#;

        let err = parse_xsrf_token(html).unwrap_err();
        assert!(matches!(err, ClientError::UnexpectedContent { .. }));
    }

    // --- is_login_page tests ---

    #[test]
    fn is_login_page_url_contains_login() {
        assert!(is_login_page("/login", ""));
    }

    #[test]
    fn is_login_page_url_case_insensitive() {
        assert!(is_login_page("/Login/Account", ""));
    }

    #[test]
    fn is_login_page_body_contains_password_input() {
        assert!(is_login_page(
            "/other",
            r#"<input type="password" name="Password">"#
        ));
    }

    #[test]
    fn is_login_page_normal_page_returns_false() {
        assert!(!is_login_page("/home", "<p>Hello</p>"));
    }

    #[test]
    fn is_login_page_incidental_login_string_ignored() {
        // URL doesn't contain /login, body doesn't have password input
        assert!(!is_login_page(
            "/ExecutiveSummary",
            r#"<script>if ("ExecutiveSummary" != "Login") { init(); }</script>"#
        ));
        assert!(!is_login_page("/api/relogin", ""));
        assert!(!is_login_page("/not-login", ""));
    }

    // --- extract_xsrf_token integration tests ---

    #[tokio::test]
    async fn extract_xsrf_token_happy_path() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/ExecutiveSummary")
            .with_status(200)
            .with_body(
                r#"<html><input name="__RequestVerificationToken" type="hidden" value="xsrf-123"></html>"#,
            )
            .create_async()
            .await;
        let client = Client::with_config(test_session(), test_config(&server)).unwrap();

        let token = extract_xsrf_token(&client).await.unwrap();

        assert_eq!(token, "xsrf-123");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn extract_xsrf_token_missing_input_returns_unexpected_content() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/ExecutiveSummary")
            .with_status(200)
            .with_body("<html></html>")
            .create_async()
            .await;
        let client = Client::with_config(test_session(), test_config(&server)).unwrap();

        let err = extract_xsrf_token(&client).await.unwrap_err();

        assert!(matches!(err, ClientError::UnexpectedContent { .. }));
    }

    #[tokio::test]
    async fn extract_xsrf_token_login_redirect_returns_session_expired() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/ExecutiveSummary")
            .with_status(302)
            .with_header("location", "/login")
            .create_async()
            .await;
        server
            .mock("GET", "/login")
            .with_status(200)
            .with_body(r#"<html><input type="password"></html>"#)
            .create_async()
            .await;
        let client = Client::with_config(test_session(), test_config(&server)).unwrap();

        let err = extract_xsrf_token(&client).await.unwrap_err();

        assert!(err.is_session_expired());
    }

    #[tokio::test]
    async fn extract_xsrf_token_login_page_body_returns_session_expired() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/ExecutiveSummary")
            .with_status(200)
            .with_body(
                r#"<html><form action="/Login/Login"><input type="password" name="Password"></form></html>"#,
            )
            .create_async()
            .await;
        let client = Client::with_config(test_session(), test_config(&server)).unwrap();

        let err = extract_xsrf_token(&client).await.unwrap_err();

        assert!(err.is_session_expired());
    }

    #[tokio::test]
    async fn extract_xsrf_token_ignores_incidental_login_string() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/ExecutiveSummary")
            .with_status(200)
            .with_body(
                r#"<html><body>
                <input name="__RequestVerificationToken" type="hidden" value="xsrf-ok">
                <script>if ("ExecutiveSummary" != "Login") { init(); }</script>
                </body></html>"#,
            )
            .create_async()
            .await;
        let client = Client::with_config(test_session(), test_config(&server)).unwrap();

        let token = extract_xsrf_token(&client).await.unwrap();

        assert_eq!(token, "xsrf-ok");
    }

    // --- refresh_session tests ---

    #[tokio::test]
    async fn refresh_session_returns_session_with_new_token() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/ExecutiveSummary")
            .with_status(200)
            .with_body(
                r#"<html><input name="__RequestVerificationToken" type="hidden" value="new-xsrf"></html>"#,
            )
            .create_async()
            .await;
        let session = test_session();
        let client = Client::with_config(session.clone(), test_config(&server)).unwrap();

        let new_session = refresh_session(&client, &session).await.unwrap();

        assert_eq!(new_session.xsrf_token(), "new-xsrf");
        assert_eq!(new_session.cookies().len(), session.cookies().len());
    }

    // --- XSRF token never appears in errors ---

    #[test]
    fn error_messages_never_contain_xsrf_token() {
        // Parse a token successfully first to confirm the value
        let html = r#"<html><input name="__RequestVerificationToken" type="hidden" value="super-secret-xsrf"></html>"#;
        let token = parse_xsrf_token(html).unwrap();
        assert_eq!(token, "super-secret-xsrf");

        // Verify error cases don't leak token values
        let missing_err = parse_xsrf_token("<html></html>").unwrap_err();
        let debug = format!("{missing_err:?}");
        let display = missing_err.to_string();
        assert!(
            !debug.contains("super-secret"),
            "Debug must not contain token value"
        );
        assert!(
            !display.contains("super-secret"),
            "Display must not contain token value"
        );
        assert!(
            debug.contains("[REDACTED]"),
            "Debug should show [REDACTED], got: {debug}"
        );
    }
}
