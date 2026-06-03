//! Local environment and auth readiness diagnostics.

use std::io;

use serde::Serialize;

use crate::ResolvedCredentials;
use crate::alerts::AlertConfigsRequest;
use crate::cli::DoctorArgs;
use crate::cli::error::{CliErrorKind, client_error_kind};
use crate::cli::output::{finish_output, print_json};
use crate::client::{Client, ClientConfig};
use crate::config::{ENV_PASSWORD, ENV_USERNAME, default_config_path};
use crate::error::{ClientError, Result};
use crate::session::{FORMS_AUTH_COOKIE_NAME, SESSION_COOKIE_NAME, Session};

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
    credentials_set: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    credential_config_path: Option<String>,
    cached_session: bool,
    session_cookie_found: bool,
    forms_auth_cookie_found: bool,
    xsrf_token_found: bool,
    status: AuthStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    actions: Vec<String>,
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
    let cached = crate::load_cached_session();
    let cached_session_valid = cached
        .as_ref()
        .is_some_and(|session| session.validate().is_ok());
    let mut credential_resolution = (!cached_session_valid).then(crate::resolve_credentials);
    let mut cached_session_problem = None;

    let live_connectivity = if live {
        match (&cached, cached_session_valid) {
            (Some(session), true) => {
                match live_connectivity_from_session(session.clone(), ClientConfig::default()).await
                {
                    Ok(report) if matches!(report.status, LiveConnectivityStatus::AuthError) => {
                        cached_session_problem = report.message.clone();
                        credential_resolution = Some(crate::resolve_credentials());
                        live_connectivity_from_resolved_credentials(
                            credential_resolution.as_ref(),
                            cached_session_problem.as_deref(),
                        )
                        .await
                    }
                    Ok(report) => report,
                    Err(err) if matches!(client_error_kind(&err), CliErrorKind::AuthError) => {
                        cached_session_problem = Some(err.to_string());
                        credential_resolution = Some(crate::resolve_credentials());
                        live_connectivity_from_resolved_credentials(
                            credential_resolution.as_ref(),
                            cached_session_problem.as_deref(),
                        )
                        .await
                    }
                    Err(err) => live_connectivity_from_error(&err),
                }
            }
            _ => {
                live_connectivity_from_resolved_credentials(credential_resolution.as_ref(), None)
                    .await
            }
        }
    } else {
        skipped_live_connectivity()
    };

    let auth = auth_report_from_cache(
        &cached,
        credential_resolution.as_ref(),
        cached_session_problem.as_deref(),
    );
    let auth_ok = matches!(auth.status, AuthStatus::Ok);
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

async fn live_connectivity_from_resolved_credentials(
    credential_resolution: Option<&Result<ResolvedCredentials>>,
    cached_session_problem: Option<&str>,
) -> LiveConnectivityReport {
    match credential_resolution {
        Some(Ok(credentials)) => match live_login_and_check(credentials).await {
            Ok(report) => report,
            Err(err) => live_connectivity_from_error(&err),
        },
        Some(Err(err)) => LiveConnectivityReport {
            checked: true,
            status: LiveConnectivityStatus::AuthError,
            reason: LIVE_CONNECTIVITY_AUTH_REASON,
            message: Some(match cached_session_problem {
                Some(problem) => format!(
                    "cached session failed live auth check: {problem}; fallback credentials are unavailable: {err}"
                ),
                None => err.to_string(),
            }),
        },
        None => LiveConnectivityReport {
            checked: true,
            status: LiveConnectivityStatus::AuthError,
            reason: LIVE_CONNECTIVITY_AUTH_REASON,
            message: cached_session_problem.map(str::to_string),
        },
    }
}

async fn live_login_and_check(credentials: &ResolvedCredentials) -> Result<LiveConnectivityReport> {
    let credentials = credentials.credentials();
    let session = crate::login(credentials.username(), credentials.password()).await?;
    let _ = crate::save_session(&session);

    live_connectivity_from_session(session, ClientConfig::default()).await
}

async fn live_connectivity_from_session(
    session: Session,
    config: ClientConfig,
) -> std::result::Result<LiveConnectivityReport, ClientError> {
    let bootstrap_client = Client::with_config(session.clone(), config.clone())?;
    let xsrf_token = crate::extract_xsrf_token(&bootstrap_client).await?;
    let refreshed_session = Session::new(session.cookies().to_vec(), xsrf_token);
    let client = Client::with_config(refreshed_session, config)?;

    let request = AlertConfigsRequest::new().with_length(1);
    match client.get_alert_configs(&request).await {
        Ok(_) => Ok(LiveConnectivityReport {
            checked: true,
            status: LiveConnectivityStatus::Ok,
            reason: LIVE_CONNECTIVITY_SUCCESS_REASON,
            message: None,
        }),
        Err(err) => Ok(live_connectivity_from_error(&err)),
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

fn auth_report_from_cache(
    cached: &Option<Session>,
    credential_resolution: Option<&Result<ResolvedCredentials>>,
    cached_session_problem: Option<&str>,
) -> AuthReport {
    match cached {
        Some(session) => {
            let cookies = session.cookies();
            let session_cookie_found = has_cookie(cookies, SESSION_COOKIE_NAME);
            let forms_auth_cookie_found = has_cookie(cookies, FORMS_AUTH_COOKIE_NAME);
            let xsrf_token_found = !session.xsrf_token().is_empty();
            let validation = match cached_session_problem {
                Some(problem) => Err(problem.to_string()),
                None => session.validate().map_err(|err| err.to_string()),
            };
            let credential_report = credential_report(credential_resolution);
            let (status, source, message, actions) = match validation {
                Ok(()) => (AuthStatus::Ok, Some("xdg_cache"), None, Vec::new()),
                Err(_err) if credential_report.credentials_set => (
                    AuthStatus::Ok,
                    credential_report.source,
                    Some(format!(
                        "cached session is invalid, but credentials are available from {}; the next live command can log in and refresh the cache",
                        credential_report.source.unwrap_or("configured credentials")
                    )),
                    credential_report.actions,
                ),
                Err(err) => (
                    AuthStatus::Invalid,
                    None,
                    Some(format!(
                        "cached session is invalid and no fallback credentials are available: {err}; {}",
                        credential_report
                            .message
                            .unwrap_or_else(|| "configure credentials".to_string())
                    )),
                    credential_report.actions,
                ),
            };

            AuthReport {
                kind: "credentials",
                credentials_set: credential_report.credentials_set,
                source,
                credential_config_path: credential_report.config_path,
                cached_session: true,
                session_cookie_found,
                forms_auth_cookie_found,
                xsrf_token_found,
                status,
                message,
                actions,
            }
        }
        None => {
            let credential_report = credential_report(credential_resolution);
            let (status, source, message, actions) = if credential_report.credentials_set {
                (
                    AuthStatus::Ok,
                    credential_report.source,
                    Some(format!(
                        "credentials are available from {}; no cached session exists yet, so the first live command will log in and create one",
                        credential_report.source.unwrap_or("configured credentials")
                    )),
                    credential_report.actions,
                )
            } else {
                (
                    AuthStatus::Missing,
                    None,
                    credential_report.message,
                    credential_report.actions,
                )
            };

            AuthReport {
                kind: "credentials",
                credentials_set: credential_report.credentials_set,
                source,
                credential_config_path: credential_report.config_path,
                cached_session: false,
                session_cookie_found: false,
                forms_auth_cookie_found: false,
                xsrf_token_found: false,
                status,
                message,
                actions,
            }
        }
    }
}

#[derive(Debug)]
struct CredentialReport {
    credentials_set: bool,
    source: Option<&'static str>,
    config_path: Option<String>,
    message: Option<String>,
    actions: Vec<String>,
}

fn credential_report(resolution: Option<&Result<ResolvedCredentials>>) -> CredentialReport {
    match resolution {
        Some(Ok(resolved)) => CredentialReport {
            credentials_set: true,
            source: Some(resolved.source().kind()),
            config_path: resolved
                .source()
                .path()
                .map(|path| path.display().to_string()),
            message: None,
            actions: vec![
                "Run volumeleaders-agent doctor --live to verify the credentials and create or refresh the cached session.".to_string(),
            ],
        },
        Some(Err(err)) => {
            let config_path = default_config_path().ok().map(|path| path.display().to_string());
            CredentialReport {
                credentials_set: false,
                source: None,
                config_path: config_path.clone(),
                message: Some(err.to_string()),
                actions: auth_setup_actions(config_path.as_deref()),
            }
        }
        None => CredentialReport {
            credentials_set: false,
            source: None,
            config_path: None,
            message: None,
            actions: Vec::new(),
        },
    }
}

fn auth_setup_actions(config_path: Option<&str>) -> Vec<String> {
    let config_action = match config_path {
        Some(path) => format!(
            "Or create {path} containing {{\"username\":\"YOUR_EMAIL\",\"password\":\"YOUR_PASSWORD\"}}."
        ),
        None => "Or create the XDG config file volumeleaders-agent/config.json containing {\"username\":\"YOUR_EMAIL\",\"password\":\"YOUR_PASSWORD\"}.".to_string(),
    };

    vec![
        format!(
            "Set {ENV_USERNAME} and {ENV_PASSWORD} in the process environment with the VolumeLeaders login email and password."
        ),
        config_action,
        format!(
            "Do not set only one auth environment variable; if either {ENV_USERNAME} or {ENV_PASSWORD} is present, both must be non-empty and config fallback is skipped."
        ),
        "Run volumeleaders-agent doctor, then volumeleaders-agent doctor --live to verify login and create the cached session.".to_string(),
    ]
}

fn has_cookie(cookies: &[crate::session::Cookie], name: &str) -> bool {
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

    use crate::session::{COOKIE_DOMAIN, Cookie, FORMS_AUTH_COOKIE_NAME, SESSION_COOKIE_NAME};
    use crate::{
        ClientError, ClientError as Error, CredentialSource, Credentials, ResolvedCredentials,
    };

    use super::{
        AuthReport, AuthStatus, DoctorReport, LIVE_CONNECTIVITY_FAILURE_REASON,
        LiveConnectivityReport, LiveConnectivityStatus, auth_report_from_cache, doctor_exit_code,
        finish_doctor_output, has_cookie, live_connectivity_from_error,
    };

    fn valid_session() -> crate::session::Session {
        crate::session::Session::new(
            vec![
                Cookie::new(SESSION_COOKIE_NAME, "session-123", COOKIE_DOMAIN),
                Cookie::new(FORMS_AUTH_COOKIE_NAME, "auth-456", COOKIE_DOMAIN),
            ],
            "xsrf-789",
        )
    }

    #[test]
    fn auth_report_from_valid_cached_session() {
        let session = Some(valid_session());
        let report = auth_report_from_cache(&session, None, None);

        assert_eq!(report.kind, "credentials");
        assert!(!report.credentials_set);
        assert_eq!(report.source, Some("xdg_cache"));
        assert!(report.cached_session);
        assert!(report.session_cookie_found);
        assert!(report.forms_auth_cookie_found);
        assert!(report.xsrf_token_found);
        assert!(matches!(report.status, AuthStatus::Ok));
        assert!(report.message.is_none());
        assert!(report.actions.is_empty());
    }

    #[test]
    fn auth_report_with_no_cache_and_no_credentials() {
        let resolution = Err(ClientError::SessionValidation {
            message: "set VL_USERNAME and VL_PASSWORD".to_string(),
        });
        let report = auth_report_from_cache(&None, Some(&resolution), None);

        assert_eq!(report.kind, "credentials");
        assert!(!report.credentials_set);
        assert!(!report.cached_session);
        assert!(matches!(report.status, AuthStatus::Missing));
        assert!(report.message.unwrap().contains("VL_USERNAME"));
        assert!(
            report
                .actions
                .iter()
                .any(|action| action.contains("config"))
        );
    }

    #[test]
    fn auth_report_with_no_cache_but_credentials_set() {
        let resolution = Ok(ResolvedCredentials::new(
            Credentials::new("user@example.com", "password"),
            CredentialSource::Environment,
        ));
        let report = auth_report_from_cache(&None, Some(&resolution), None);

        assert!(report.credentials_set);
        assert_eq!(report.source, Some("environment"));
        assert!(!report.cached_session);
        assert!(matches!(report.status, AuthStatus::Ok));
        assert!(report.message.unwrap().contains("first live command"));
    }

    #[test]
    fn auth_report_from_invalid_cached_session() {
        let bad_session = Some(crate::session::Session::new(vec![], ""));
        let resolution = Err(ClientError::SessionValidation {
            message: "set VL_USERNAME and VL_PASSWORD".to_string(),
        });
        let report = auth_report_from_cache(&bad_session, Some(&resolution), None);

        assert!(report.cached_session);
        assert!(!report.xsrf_token_found);
        assert!(matches!(report.status, AuthStatus::Invalid));
        assert!(
            report
                .actions
                .iter()
                .any(|action| action.contains("doctor --live"))
        );
    }

    #[test]
    fn auth_report_from_live_failed_cached_session_includes_recovery_actions() {
        let session = Some(valid_session());
        let resolution = Err(ClientError::SessionValidation {
            message: "set VL_USERNAME and VL_PASSWORD".to_string(),
        });
        let report = auth_report_from_cache(
            &session,
            Some(&resolution),
            Some("cached session expired server-side"),
        );

        assert!(report.cached_session);
        assert!(!report.credentials_set);
        assert!(matches!(report.status, AuthStatus::Invalid));
        assert!(
            report
                .message
                .unwrap()
                .contains("cached session expired server-side")
        );
        assert!(
            report
                .actions
                .iter()
                .any(|action| action.contains("VL_USERNAME"))
        );
    }

    #[test]
    fn auth_report_from_live_failed_cached_session_uses_fallback_credentials() {
        let session = Some(valid_session());
        let resolution = Ok(ResolvedCredentials::new(
            Credentials::new("user@example.com", "password"),
            CredentialSource::Environment,
        ));
        let report = auth_report_from_cache(
            &session,
            Some(&resolution),
            Some("cached session expired server-side"),
        );

        assert!(report.cached_session);
        assert!(report.credentials_set);
        assert_eq!(report.source, Some("environment"));
        assert!(matches!(report.status, AuthStatus::Ok));
        assert!(
            report
                .message
                .unwrap()
                .contains("credentials are available")
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
                kind: "credentials",
                credentials_set: true,
                source: Some("xdg_cache"),
                credential_config_path: None,
                cached_session: true,
                session_cookie_found: true,
                forms_auth_cookie_found: true,
                xsrf_token_found: true,
                status: AuthStatus::Ok,
                message: None,
                actions: Vec::new(),
            },
            live_connectivity: LiveConnectivityReport {
                checked: true,
                status,
                reason: LIVE_CONNECTIVITY_FAILURE_REASON,
                message: None,
            },
        }
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
        let login = Error::LoginFailed {
            reason: "bad password".to_string(),
        };

        let auth_report: Value = serde_json::to_value(live_connectivity_from_error(&auth)).unwrap();
        let api_report: Value = serde_json::to_value(live_connectivity_from_error(&api)).unwrap();
        let json_report: Value = serde_json::to_value(live_connectivity_from_error(&json)).unwrap();
        let login_report: Value =
            serde_json::to_value(live_connectivity_from_error(&login)).unwrap();

        assert_eq!(auth_report["status"], "auth_error");
        assert_eq!(api_report["status"], "api_error");
        assert_eq!(json_report["status"], "json_error");
        assert_eq!(login_report["status"], "auth_error");
    }

    #[test]
    fn live_connectivity_reports_api_error_from_status_200() {
        let report: Value =
            serde_json::to_value(super::live_connectivity_from_error(&ClientError::Status {
                code: 200,
                url: "test".into(),
                body: "ok".into(),
            }))
            .unwrap();

        assert_eq!(report["status"], "api_error");
    }
}
