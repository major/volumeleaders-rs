//! Local environment and auth readiness diagnostics.

use std::io;

use serde::Serialize;

use crate::cli::common::auth::VL_DOMAIN;
use crate::cli::error::EXIT_AUTH_ERROR;
use crate::cli::output::{finish_output, print_json};
use crate::error::Result;
use crate::session::{
    Cookie, FORMS_AUTH_COOKIE_NAME, REQUEST_VERIFICATION_COOKIE_NAME, SESSION_COOKIE_NAME, Session,
};

const LIVE_CONNECTIVITY_SKIP_REASON: &str = "doctor performs local checks by default";

/// Emit local readiness diagnostics as compact JSON.
pub fn handle() -> i32 {
    let report = build_report();
    finish_doctor_output(print_json(&report), report.ok)
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
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum LiveConnectivityStatus {
    Skipped,
}

fn build_report() -> DoctorReport {
    build_report_from_cookies(crate::extract_browser_cookies(VL_DOMAIN))
}

fn build_report_from_cookies(cookies: Result<Vec<Cookie>>) -> DoctorReport {
    let auth = match cookies {
        Ok(cookies) => auth_report_from_cookies(cookies),
        Err(err) => AuthReport {
            kind: "browser_cookies",
            cookie_source: "chrome_or_firefox",
            cookies_found: false,
            session_cookie_found: false,
            forms_auth_cookie_found: false,
            xsrf_token_found: false,
            status: AuthStatus::Missing,
            message: Some(err.to_string()),
        },
    };
    let ok = matches!(auth.status, AuthStatus::Ok);

    DoctorReport {
        ok,
        version: env!("CARGO_PKG_VERSION"),
        auth,
        live_connectivity: LiveConnectivityReport {
            checked: false,
            status: LiveConnectivityStatus::Skipped,
            reason: LIVE_CONNECTIVITY_SKIP_REASON,
        },
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

fn finish_doctor_output(result: io::Result<()>, ok: bool) -> i32 {
    match result {
        Ok(()) if ok => 0,
        Ok(()) => EXIT_AUTH_ERROR,
        Err(err) => finish_output(Err(err)),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::ClientError;
    use crate::session::{
        COOKIE_DOMAIN, Cookie, FORMS_AUTH_COOKIE_NAME, REQUEST_VERIFICATION_COOKIE_NAME,
        SESSION_COOKIE_NAME,
    };

    use super::{build_report_from_cookies, finish_doctor_output, has_cookie};

    #[test]
    fn doctor_report_includes_static_contract() {
        let report: Value = serde_json::to_value(build_report_from_cookies(Err(
            ClientError::SessionValidation {
                message: "no cookies found".to_string(),
            },
        )))
        .unwrap();

        assert_eq!(report["ok"], false);
        assert_eq!(report["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(report["auth"]["kind"], "browser_cookies");
        assert_eq!(report["auth"]["cookies_found"], false);
        assert_eq!(report["auth"]["status"], "missing");
        assert_eq!(report["live_connectivity"]["checked"], false);
        assert_eq!(report["live_connectivity"]["status"], "skipped");
    }

    #[test]
    fn doctor_report_marks_complete_cookie_set_ok() {
        let report: Value = serde_json::to_value(build_report_from_cookies(Ok(vec![
            Cookie::new(SESSION_COOKIE_NAME, "session-123", COOKIE_DOMAIN),
            Cookie::new(FORMS_AUTH_COOKIE_NAME, "auth-456", COOKIE_DOMAIN),
            Cookie::new(REQUEST_VERIFICATION_COOKIE_NAME, "xsrf-789", COOKIE_DOMAIN),
        ])))
        .unwrap();

        assert_eq!(report["ok"], true);
        assert_eq!(report["auth"]["cookies_found"], true);
        assert_eq!(report["auth"]["session_cookie_found"], true);
        assert_eq!(report["auth"]["forms_auth_cookie_found"], true);
        assert_eq!(report["auth"]["xsrf_token_found"], true);
        assert_eq!(report["auth"]["status"], "ok");
        assert_eq!(report["auth"]["message"], Value::Null);
    }

    #[test]
    fn doctor_report_marks_partial_cookie_set_invalid() {
        let report: Value = serde_json::to_value(build_report_from_cookies(Ok(vec![
            Cookie::new(SESSION_COOKIE_NAME, "session-123", COOKIE_DOMAIN),
            Cookie::new(REQUEST_VERIFICATION_COOKIE_NAME, "xsrf-789", COOKIE_DOMAIN),
        ])))
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
        assert_eq!(finish_doctor_output(Ok(()), true), 0);
        assert_eq!(finish_doctor_output(Ok(()), false), 3);
        assert_eq!(
            finish_doctor_output(Err(std::io::Error::other("stdout closed")), true),
            6
        );
    }
}
