//! Local environment and auth readiness diagnostics.

use std::io;

use serde::Serialize;

use crate::alerts::AlertConfigsRequest;
use crate::cli::DoctorArgs;
use crate::cli::common::auth::VL_DOMAIN;
use crate::cli::error::{CliErrorKind, client_error_kind};
use crate::cli::output::{finish_output, print_json};
use crate::client::{Client, ClientConfig};
use crate::error::{ClientError, Result};
use crate::session::{
    Cookie, FORMS_AUTH_COOKIE_NAME, REQUEST_VERIFICATION_COOKIE_NAME, SESSION_COOKIE_NAME, Session,
};

const LIVE_CONNECTIVITY_SKIP_REASON: &str = "doctor performs local checks by default";
const LIVE_CONNECTIVITY_SUCCESS_REASON: &str = "alert configs endpoint responded successfully";
const LIVE_CONNECTIVITY_AUTH_REASON: &str = "authenticated live check could not be started";
const LIVE_CONNECTIVITY_FAILURE_REASON: &str = "authenticated live check failed";

/// Emit local or live readiness diagnostics as compact JSON.
pub async fn handle(args: &DoctorArgs) -> i32 {
    let report = build_report(args.live).await;
    let exit_code = doctor_exit_code(&report);
    finish_doctor_output(print_json(&report), exit_code)
}

#[derive(Debug, Serialize)]
struct DoctorReport {
    ok: bool,
    version: &'static str,
    auth: AuthReport,
    live_connectivity: LiveConnectivityReport,
}

#[derive(Debug, Serialize)]
struct AuthReport {
    kind: &'static str,
    cookie_source: &'static str,
    cookies_found: bool,
    session_cookie_found: bool,
    forms_auth_cookie_found: bool,
    xsrf_token_found: bool,
    status: AuthStatus,
    message: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum AuthStatus {
    Missing,
    Invalid,
    Ok,
}

#[derive(Debug, Serialize)]
struct LiveConnectivityReport {
    checked: bool,
    status: LiveConnectivityStatus,
    reason: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum LiveConnectivityStatus {
    Skipped,
    Ok,
    AuthError,
    HttpError,
    ApiError,
    JsonError,
}

async fn build_report(live: bool) -> DoctorReport {
    build_report_from_cookies(crate::extract_browser_cookies(VL_DOMAIN), live).await
}

async fn build_report_from_cookies(cookies: Result<Vec<Cookie>>, live: bool) -> DoctorReport {
    build_report_from_cookies_with_config(cookies, live, ClientConfig::default()).await
}

async fn build_report_from_cookies_with_config(
    cookies: Result<Vec<Cookie>>,
    live: bool,
    config: ClientConfig,
) -> DoctorReport {
    let (auth, cookie_values) = match cookies {
        Ok(cookies) => (auth_report_from_cookies(cookies.clone()), Some(cookies)),
        Err(err) => (
            AuthReport {
                kind: "browser_cookies",
                cookie_source: "chrome_or_firefox",
                cookies_found: false,
                session_cookie_found: false,
                forms_auth_cookie_found: false,
                xsrf_token_found: false,
                status: AuthStatus::Missing,
                message: Some(err.to_string()),
            },
            None,
        ),
    };
    let auth_ok = matches!(auth.status, AuthStatus::Ok);
    let live_connectivity = if live {
        match cookie_values {
            Some(cookies) if auth_ok => live_connectivity_from_cookies(cookies, config).await,
            _ => LiveConnectivityReport {
                checked: true,
                status: LiveConnectivityStatus::AuthError,
                reason: LIVE_CONNECTIVITY_AUTH_REASON,
                message: auth.message.clone(),
            },
        }
    } else {
        skipped_live_connectivity()
    };
    let live_ok = matches!(
        live_connectivity.status,
        LiveConnectivityStatus::Skipped | LiveConnectivityStatus::Ok
    );
    let ok = auth_ok && live_ok;

    DoctorReport {
        ok,
        version: env!("CARGO_PKG_VERSION"),
        auth,
        live_connectivity,
    }
}

fn skipped_live_connectivity() -> LiveConnectivityReport {
    LiveConnectivityReport {
        checked: false,
        status: LiveConnectivityStatus::Skipped,
        reason: LIVE_CONNECTIVITY_SKIP_REASON,
        message: None,
    }
}

async fn live_connectivity_from_cookies(
    cookies: Vec<Cookie>,
    config: ClientConfig,
) -> LiveConnectivityReport {
    match live_client_from_cookies(cookies, config).await {
        Ok(client) => live_connectivity_from_client(&client).await,
        Err(err) => live_connectivity_from_error(&err),
    }
}

async fn live_client_from_cookies(cookies: Vec<Cookie>, config: ClientConfig) -> Result<Client> {
    let session = Session::from_cookies(cookies)?;
    let bootstrap_client = Client::with_config(session.clone(), config.clone())?;
    let xsrf_token = crate::extract_xsrf_token(&bootstrap_client).await?;
    let refreshed_session = Session::new(session.cookies().to_vec(), xsrf_token);
    Client::with_config(refreshed_session, config)
}

async fn live_connectivity_from_client(client: &Client) -> LiveConnectivityReport {
    let request = AlertConfigsRequest::new().with_length(1);
    match client.get_alert_configs(&request).await {
        Ok(_) => LiveConnectivityReport {
            checked: true,
            status: LiveConnectivityStatus::Ok,
            reason: LIVE_CONNECTIVITY_SUCCESS_REASON,
            message: None,
        },
        Err(err) => live_connectivity_from_error(&err),
    }
}

fn live_connectivity_from_error(err: &ClientError) -> LiveConnectivityReport {
    LiveConnectivityReport {
        checked: true,
        status: live_status_from_error(err),
        reason: LIVE_CONNECTIVITY_FAILURE_REASON,
        message: Some(err.to_string()),
    }
}

fn live_status_from_error(err: &ClientError) -> LiveConnectivityStatus {
    match client_error_kind(err) {
        CliErrorKind::AuthError => LiveConnectivityStatus::AuthError,
        CliErrorKind::HttpError => LiveConnectivityStatus::HttpError,
        CliErrorKind::ApiError => LiveConnectivityStatus::ApiError,
        CliErrorKind::JsonError | CliErrorKind::UsageError | CliErrorKind::EmptyResult => {
            LiveConnectivityStatus::JsonError
        }
    }
}

fn auth_report_from_cookies(cookies: Vec<Cookie>) -> AuthReport {
    let session_cookie_found = has_cookie(&cookies, SESSION_COOKIE_NAME);
    let forms_auth_cookie_found = has_cookie(&cookies, FORMS_AUTH_COOKIE_NAME);
    let xsrf_token_found = has_cookie(&cookies, REQUEST_VERIFICATION_COOKIE_NAME);
    let session = Session::from_cookies(cookies);
    let validation = session.and_then(|session| session.validate());
    let (status, message) = match validation {
        Ok(()) => (AuthStatus::Ok, None),
        Err(err) => (AuthStatus::Invalid, Some(err.to_string())),
    };

    AuthReport {
        kind: "browser_cookies",
        cookie_source: "chrome_or_firefox",
        cookies_found: session_cookie_found || forms_auth_cookie_found || xsrf_token_found,
        session_cookie_found,
        forms_auth_cookie_found,
        xsrf_token_found,
        status,
        message,
    }
}

fn has_cookie(cookies: &[Cookie], name: &str) -> bool {
    cookies
        .iter()
        .any(|cookie| cookie.name() == name && !cookie.value().is_empty())
}

fn doctor_exit_code(report: &DoctorReport) -> i32 {
    match report.live_connectivity.status {
        LiveConnectivityStatus::Ok | LiveConnectivityStatus::Skipped if report.ok => 0,
        LiveConnectivityStatus::AuthError | LiveConnectivityStatus::Skipped => {
            CliErrorKind::AuthError.exit_code()
        }
        LiveConnectivityStatus::HttpError => CliErrorKind::HttpError.exit_code(),
        LiveConnectivityStatus::ApiError => CliErrorKind::ApiError.exit_code(),
        LiveConnectivityStatus::JsonError => CliErrorKind::JsonError.exit_code(),
        LiveConnectivityStatus::Ok => CliErrorKind::AuthError.exit_code(),
    }
}

fn finish_doctor_output(result: io::Result<()>, exit_code: i32) -> i32 {
    match result {
        Ok(()) => exit_code,
        Err(err) => finish_output(Err(err)),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::alerts::ALERT_CONFIGS_GET_ALERT_CONFIGS_PATH;
    use crate::client::ClientConfig;
    use crate::session::{
        COOKIE_DOMAIN, Cookie, FORMS_AUTH_COOKIE_NAME, REQUEST_VERIFICATION_COOKIE_NAME,
        SESSION_COOKIE_NAME,
    };
    use crate::test_support::{datatables_body, test_client};
    use crate::{ClientError, ClientError as Error};

    use super::{
        AuthReport, AuthStatus, DoctorReport, LIVE_CONNECTIVITY_FAILURE_REASON,
        LiveConnectivityReport, LiveConnectivityStatus, build_report_from_cookies,
        build_report_from_cookies_with_config, doctor_exit_code, finish_doctor_output, has_cookie,
        live_connectivity_from_client,
    };

    fn valid_cookies() -> Vec<Cookie> {
        vec![
            Cookie::new(SESSION_COOKIE_NAME, "session-123", COOKIE_DOMAIN),
            Cookie::new(FORMS_AUTH_COOKIE_NAME, "auth-456", COOKIE_DOMAIN),
            Cookie::new(REQUEST_VERIFICATION_COOKIE_NAME, "xsrf-789", COOKIE_DOMAIN),
        ]
    }

    #[tokio::test]
    async fn doctor_report_includes_static_contract() {
        let report: Value = serde_json::to_value(
            build_report_from_cookies(
                Err(ClientError::SessionValidation {
                    message: "no cookies found".to_string(),
                }),
                false,
            )
            .await,
        )
        .unwrap();

        assert_eq!(report["ok"], false);
        assert_eq!(report["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(report["auth"]["kind"], "browser_cookies");
        assert_eq!(report["auth"]["cookies_found"], false);
        assert_eq!(report["auth"]["status"], "missing");
        assert_eq!(report["live_connectivity"]["checked"], false);
        assert_eq!(report["live_connectivity"]["status"], "skipped");
    }

    #[tokio::test]
    async fn doctor_report_marks_complete_cookie_set_ok() {
        let report: Value =
            serde_json::to_value(build_report_from_cookies(Ok(valid_cookies()), false).await)
                .unwrap();

        assert_eq!(report["ok"], true);
        assert_eq!(report["auth"]["cookies_found"], true);
        assert_eq!(report["auth"]["session_cookie_found"], true);
        assert_eq!(report["auth"]["forms_auth_cookie_found"], true);
        assert_eq!(report["auth"]["xsrf_token_found"], true);
        assert_eq!(report["auth"]["status"], "ok");
        assert_eq!(report["auth"]["message"], Value::Null);
    }

    #[tokio::test]
    async fn doctor_report_marks_partial_cookie_set_invalid() {
        let report: Value = serde_json::to_value(
            build_report_from_cookies(
                Ok(vec![
                    Cookie::new(SESSION_COOKIE_NAME, "session-123", COOKIE_DOMAIN),
                    Cookie::new(REQUEST_VERIFICATION_COOKIE_NAME, "xsrf-789", COOKIE_DOMAIN),
                ]),
                false,
            )
            .await,
        )
        .unwrap();

        assert_eq!(report["ok"], false);
        assert_eq!(report["auth"]["cookies_found"], true);
        assert_eq!(report["auth"]["forms_auth_cookie_found"], false);
        assert_eq!(report["auth"]["status"], "invalid");
        assert!(
            report["auth"]["message"]
                .as_str()
                .unwrap()
                .contains(FORMS_AUTH_COOKIE_NAME)
        );
    }

    #[test]
    fn empty_cookie_values_are_not_counted() {
        let cookies = vec![Cookie::new(SESSION_COOKIE_NAME, "", COOKIE_DOMAIN)];

        assert!(!has_cookie(&cookies, SESSION_COOKIE_NAME));
    }

    #[test]
    fn doctor_output_exit_code_follows_report_status_and_write_errors() {
        assert_eq!(finish_doctor_output(Ok(()), 0), 0);
        assert_eq!(finish_doctor_output(Ok(()), 3), 3);
        assert_eq!(
            finish_doctor_output(Err(std::io::Error::other("stdout closed")), 0),
            6
        );
    }

    #[tokio::test]
    async fn live_connectivity_reports_success_from_alert_configs() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", ALERT_CONFIGS_GET_ALERT_CONFIGS_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"draw":1,"recordsTotal":0,"recordsFiltered":0,"data":[]}"#)
            .create_async()
            .await;
        let client = test_client(&server);

        let report: Value =
            serde_json::to_value(live_connectivity_from_client(&client).await).unwrap();

        assert_eq!(report["checked"], true);
        assert_eq!(report["status"], "ok");
        assert_eq!(report["message"], Value::Null);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn live_report_uses_cookies_to_refresh_session_and_check_alert_configs() {
        let mut server = mockito::Server::new_async().await;
        let token_mock = server
            .mock("GET", "/ExecutiveSummary")
            .with_status(200)
            .with_body(
                r#"<html><input name="__RequestVerificationToken" type="hidden" value="fresh-xsrf"></html>"#,
            )
            .create_async()
            .await;
        let alert_mock = server
            .mock("POST", ALERT_CONFIGS_GET_ALERT_CONFIGS_PATH)
            .match_header("x-xsrf-token", "fresh-xsrf")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(datatables_body::<Value>(Vec::new()))
            .create_async()
            .await;

        let report = build_report_from_cookies_with_config(
            Ok(valid_cookies()),
            true,
            ClientConfig {
                base_url: server.url(),
                ..ClientConfig::default()
            },
        )
        .await;
        let value: Value = serde_json::to_value(&report).unwrap();

        assert_eq!(doctor_exit_code(&report), 0);
        assert_eq!(value["ok"], true);
        assert_eq!(value["live_connectivity"]["checked"], true);
        assert_eq!(value["live_connectivity"]["status"], "ok");
        token_mock.assert_async().await;
        alert_mock.assert_async().await;
    }

    #[tokio::test]
    async fn live_report_maps_client_setup_errors_to_json_status() {
        let report = build_report_from_cookies_with_config(
            Ok(valid_cookies()),
            true,
            ClientConfig {
                base_url: "not a url".to_string(),
                ..ClientConfig::default()
            },
        )
        .await;
        let value: Value = serde_json::to_value(&report).unwrap();

        assert_eq!(doctor_exit_code(&report), 6);
        assert_eq!(value["ok"], false);
        assert_eq!(value["live_connectivity"]["checked"], true);
        assert_eq!(value["live_connectivity"]["status"], "json_error");
    }

    #[tokio::test]
    async fn live_connectivity_reports_api_error_from_alert_configs() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("POST", ALERT_CONFIGS_GET_ALERT_CONFIGS_PATH)
            .with_status(500)
            .with_header("content-type", "text/plain")
            .with_body("server error")
            .create_async()
            .await;
        let client = test_client(&server);

        let report: Value =
            serde_json::to_value(live_connectivity_from_client(&client).await).unwrap();

        assert_eq!(report["checked"], true);
        assert_eq!(report["status"], "api_error");
    }

    #[test]
    fn live_connectivity_classifies_client_errors() {
        let auth = Error::SessionValidation {
            message: "missing cookies".to_string(),
        };
        let api = Error::Status {
            code: 500,
            url: "/AlertConfigs/GetAlertConfigs".to_string(),
            body: "server error".to_string(),
        };
        let json = Error::UnexpectedContent {
            expected: "JSON".to_string(),
            actual: "not json".to_string(),
            url: "/AlertConfigs/GetAlertConfigs".to_string(),
        };

        let auth_report: Value =
            serde_json::to_value(super::live_connectivity_from_error(&auth)).unwrap();
        let api_report: Value =
            serde_json::to_value(super::live_connectivity_from_error(&api)).unwrap();
        let json_report: Value =
            serde_json::to_value(super::live_connectivity_from_error(&json)).unwrap();

        assert_eq!(auth_report["status"], "auth_error");
        assert_eq!(api_report["status"], "api_error");
        assert_eq!(json_report["status"], "json_error");
    }

    #[tokio::test]
    async fn live_connectivity_classifies_http_errors() {
        let err = reqwest::Client::new()
            .get("http://127.0.0.1:9")
            .send()
            .await
            .unwrap_err();

        let report: Value =
            serde_json::to_value(super::live_connectivity_from_error(&Error::Http(err))).unwrap();

        assert_eq!(report["status"], "http_error");
    }

    #[test]
    fn doctor_exit_code_maps_live_statuses() {
        let mut report = build_report_with_live_status(LiveConnectivityStatus::HttpError, false);
        assert_eq!(doctor_exit_code(&report), 4);

        report.live_connectivity.status = LiveConnectivityStatus::ApiError;
        assert_eq!(doctor_exit_code(&report), 5);

        report.live_connectivity.status = LiveConnectivityStatus::JsonError;
        assert_eq!(doctor_exit_code(&report), 6);

        report.live_connectivity.status = LiveConnectivityStatus::Ok;
        assert_eq!(doctor_exit_code(&report), 3);
    }

    fn build_report_with_live_status(status: LiveConnectivityStatus, ok: bool) -> DoctorReport {
        DoctorReport {
            ok,
            version: env!("CARGO_PKG_VERSION"),
            auth: AuthReport {
                kind: "browser_cookies",
                cookie_source: "chrome_or_firefox",
                cookies_found: true,
                session_cookie_found: true,
                forms_auth_cookie_found: true,
                xsrf_token_found: true,
                status: AuthStatus::Ok,
                message: None,
            },
            live_connectivity: LiveConnectivityReport {
                checked: true,
                status,
                reason: LIVE_CONNECTIVITY_FAILURE_REASON,
                message: None,
            },
        }
    }

    #[tokio::test]
    async fn live_report_marks_invalid_local_auth_without_network() {
        let report = build_report_from_cookies(
            Ok(vec![
                Cookie::new(SESSION_COOKIE_NAME, "session-123", COOKIE_DOMAIN),
                Cookie::new(REQUEST_VERIFICATION_COOKIE_NAME, "xsrf-789", COOKIE_DOMAIN),
            ]),
            true,
        )
        .await;
        let value: Value = serde_json::to_value(&report).unwrap();

        assert_eq!(doctor_exit_code(&report), 3);
        assert_eq!(value["ok"], false);
        assert_eq!(value["live_connectivity"]["checked"], true);
        assert_eq!(value["live_connectivity"]["status"], "auth_error");
    }
}
