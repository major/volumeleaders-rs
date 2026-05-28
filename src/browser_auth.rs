//! Browser-based authentication using locally stored cookies.
//!
//! Extracts VolumeLeaders authentication cookies from Chrome and Firefox
//! browser stores via the [`rookie`] crate, then builds a [`Session`]
//! from the extracted cookies.
//!
//! # Security
//!
//! Cookie values are never included in log output or error messages.
//! Only cookie names appear in tracing spans and diagnostics.

use crate::error::{ClientError, Result};
use crate::session::{
    Cookie, FORMS_AUTH_COOKIE_NAME, REQUEST_VERIFICATION_COOKIE_NAME, SESSION_COOKIE_NAME, Session,
};
use tracing::debug;

/// Cookie names selected from browser stores for VolumeLeaders authentication.
const VL_COOKIE_NAMES: &[&str] = &[
    SESSION_COOKIE_NAME,
    FORMS_AUTH_COOKIE_NAME,
    REQUEST_VERIFICATION_COOKIE_NAME,
];

/// Extracts VolumeLeaders authentication cookies from local browsers.
///
/// Tries Chrome first, then Firefox. Cookies from both browsers are
/// combined, with Chrome taking priority for duplicate cookie names.
/// Returns an error only when both browsers fail to provide any
/// matching cookies.
///
/// # Errors
///
/// Returns [`ClientError::SessionValidation`] when no VolumeLeaders
/// cookies can be extracted from either browser.
pub fn extract_browser_cookies(domain: &str) -> Result<Vec<Cookie>> {
    let domains: Option<Vec<String>> = Some(vec![domain.to_owned()]);
    let mut cookies = Vec::new();

    // Try Chrome first.
    match rookie::chrome(domains.clone()) {
        Ok(raw) => {
            let filtered = filter_vl_cookies(&raw, domain);
            debug!(
                browser = "chrome",
                cookie_names = ?cookie_names(&filtered),
                "extracted cookies"
            );
            cookies.extend(filtered);
        }
        Err(_) => {
            debug!("Chrome cookie extraction unavailable, trying Firefox");
        }
    }

    // Try Firefox.
    match rookie::firefox(domains) {
        Ok(raw) => {
            let filtered = filter_vl_cookies(&raw, domain);
            debug!(
                browser = "firefox",
                cookie_names = ?cookie_names(&filtered),
                "extracted cookies"
            );
            // Only add cookies not already found in Chrome.
            for cookie in filtered {
                if !cookies.iter().any(|c| c.name() == cookie.name()) {
                    cookies.push(cookie);
                }
            }
        }
        Err(_) => {
            debug!("Firefox cookie extraction unavailable");
        }
    }

    if cookies.is_empty() {
        return Err(ClientError::SessionValidation {
            message: "no VolumeLeaders cookies found in Chrome or Firefox; \
                      please log in at volumeleaders.com in your browser"
                .into(),
        });
    }

    Ok(cookies)
}

/// Builds a [`Session`] from cookies extracted from local browsers.
///
/// Calls [`extract_browser_cookies`] to get the raw cookies, then
/// passes them to [`Session::from_cookies`] which extracts the XSRF
/// token from the `__RequestVerificationToken` cookie.
///
/// # Errors
///
/// Returns [`ClientError::SessionValidation`] if no browser cookies
/// are found, or if the required `__RequestVerificationToken` cookie
/// is missing.
pub fn session_from_browser(domain: &str) -> Result<Session> {
    let cookies = extract_browser_cookies(domain)?;
    Session::from_cookies(cookies)
}

/// Converts rookie cookies to our [`Cookie`] type, keeping only
/// VolumeLeaders authentication cookie names.
fn filter_vl_cookies(raw: &[rookie::common::enums::Cookie], domain: &str) -> Vec<Cookie> {
    raw.iter()
        .filter(|c| VL_COOKIE_NAMES.contains(&c.name.as_str()))
        .map(|c| Cookie::new(&c.name, &c.value, domain))
        .collect()
}

/// Extracts cookie names for safe logging (no values).
fn cookie_names(cookies: &[Cookie]) -> Vec<&str> {
    cookies.iter().map(Cookie::name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::COOKIE_DOMAIN;

    /// Builds a rookie cookie with the given name and value.
    fn rookie_cookie(name: &str, value: &str) -> rookie::common::enums::Cookie {
        rookie::common::enums::Cookie {
            domain: COOKIE_DOMAIN.into(),
            path: "/".into(),
            secure: true,
            expires: None,
            name: name.into(),
            value: value.into(),
            http_only: true,
            same_site: 0,
        }
    }

    #[test]
    fn filter_keeps_only_vl_cookies() {
        let raw = vec![
            rookie_cookie(SESSION_COOKIE_NAME, "sess-val"),
            rookie_cookie(FORMS_AUTH_COOKIE_NAME, "auth-val"),
            rookie_cookie(REQUEST_VERIFICATION_COOKIE_NAME, "xsrf-val"),
            rookie_cookie("_ga", "tracking-id"),
            rookie_cookie("random_cookie", "junk"),
        ];

        let filtered = filter_vl_cookies(&raw, COOKIE_DOMAIN);

        assert_eq!(filtered.len(), 3);
        let names: Vec<&str> = filtered.iter().map(Cookie::name).collect();
        assert!(names.contains(&SESSION_COOKIE_NAME));
        assert!(names.contains(&FORMS_AUTH_COOKIE_NAME));
        assert!(names.contains(&REQUEST_VERIFICATION_COOKIE_NAME));
    }

    #[test]
    fn filter_returns_empty_for_no_vl_cookies() {
        let raw = vec![
            rookie_cookie("_ga", "tracking-id"),
            rookie_cookie("_gid", "other-tracking"),
        ];

        let filtered = filter_vl_cookies(&raw, COOKIE_DOMAIN);
        assert!(filtered.is_empty());
    }

    #[test]
    fn filter_sets_domain_from_parameter() {
        let raw = vec![rookie_cookie(SESSION_COOKIE_NAME, "sess-val")];
        let custom_domain = "test.example.com";

        let filtered = filter_vl_cookies(&raw, custom_domain);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].domain(), custom_domain);
    }

    #[test]
    fn cookie_names_extracts_names_only() {
        let cookies = vec![
            Cookie::new(SESSION_COOKIE_NAME, "secret-session-value", COOKIE_DOMAIN),
            Cookie::new(FORMS_AUTH_COOKIE_NAME, "secret-auth-value", COOKIE_DOMAIN),
        ];

        let names = cookie_names(&cookies);

        assert_eq!(names, vec![SESSION_COOKIE_NAME, FORMS_AUTH_COOKIE_NAME]);
        // Values must not appear anywhere in the output.
        let names_str = format!("{names:?}");
        assert!(
            !names_str.contains("secret"),
            "cookie_names must not expose values, got: {names_str}"
        );
    }

    #[test]
    fn filter_preserves_cookie_values() {
        let raw = vec![rookie_cookie(SESSION_COOKIE_NAME, "my-session-id")];

        let filtered = filter_vl_cookies(&raw, COOKIE_DOMAIN);

        assert_eq!(filtered[0].value(), "my-session-id");
    }

    #[test]
    fn filter_handles_empty_input() {
        let filtered = filter_vl_cookies(&[], COOKIE_DOMAIN);
        assert!(filtered.is_empty());
    }
}
