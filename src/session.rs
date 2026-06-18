//! Browser session carrying authentication cookies and XSRF token.
//!
//! A [`Session`] holds the browser cookies and anti-forgery token that
//! VolumeLeaders requires for authenticated API calls. Cookie values and
//! the XSRF token are treated as secrets: the [`Debug`] implementation
//! shows cookie names but replaces all values with `[REDACTED]`.

use serde::{Deserialize, Serialize};

use crate::error::{ClientError, Result};

/// Cookie domain used by VolumeLeaders browser sessions.
pub const COOKIE_DOMAIN: &str = "volumeleaders.com";

/// ASP.NET session cookie name required by VolumeLeaders.
pub const SESSION_COOKIE_NAME: &str = "ASP.NET_SessionId";

/// ASP.NET forms authentication cookie name required by VolumeLeaders.
pub const FORMS_AUTH_COOKIE_NAME: &str = ".ASPXAUTH";

/// Request verification cookie name that carries the XSRF token.
pub const REQUEST_VERIFICATION_COOKIE_NAME: &str = "__RequestVerificationToken";

/// Required cookie names checked during session validation.
const REQUIRED_COOKIE_NAMES: &[&str] = &[SESSION_COOKIE_NAME, FORMS_AUTH_COOKIE_NAME];

/// A single browser cookie with name, value, and domain.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cookie {
    name: String,
    value: String,
    domain: String,
}

impl Cookie {
    /// Creates a new cookie.
    pub fn new(
        name: impl Into<String>,
        value: impl Into<String>,
        domain: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            domain: domain.into(),
        }
    }

    /// Returns the cookie name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the cookie value.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Returns the cookie domain.
    pub fn domain(&self) -> &str {
        &self.domain
    }
}

impl std::fmt::Debug for Cookie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cookie")
            .field("name", &self.name)
            .field("value", &"[REDACTED]")
            .field("domain", &self.domain)
            .finish()
    }
}

/// Browser session carrying authentication material for VolumeLeaders.
///
/// Use [`Session::new`] when you already have the XSRF token, or
/// [`Session::from_cookies`] to extract it from a
/// `__RequestVerificationToken` cookie automatically.
///
/// Call [`Session::validate`] before making API requests to catch
/// missing authentication material early with a descriptive error
/// listing ALL missing fields.
#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    cookies: Vec<Cookie>,
    xsrf_token: String,
}

impl Session {
    /// Creates a session from cookies and an explicit XSRF token.
    ///
    /// The cookies are defensively cloned so mutations to the original
    /// `Vec` do not affect this session.
    pub fn new(cookies: Vec<Cookie>, xsrf_token: impl Into<String>) -> Self {
        Self {
            cookies: clone_cookies(&cookies),
            xsrf_token: xsrf_token.into(),
        }
    }

    /// Creates a session by extracting the XSRF token from a
    /// `__RequestVerificationToken` cookie.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError::SessionValidation`] if no cookie named
    /// `__RequestVerificationToken` is found or its value is empty.
    pub fn from_cookies(cookies: Vec<Cookie>) -> Result<Self> {
        let xsrf_token = cookies
            .iter()
            .find(|c| c.name == REQUEST_VERIFICATION_COOKIE_NAME)
            .map(|c| c.value.clone())
            .unwrap_or_default();

        if xsrf_token.is_empty() {
            return Err(ClientError::SessionValidation {
                message: format!(
                    "missing {REQUEST_VERIFICATION_COOKIE_NAME} cookie for XSRF token"
                ),
            });
        }

        Ok(Self::new(cookies, xsrf_token))
    }

    /// Validates that all required authentication material is present.
    ///
    /// Checks for [`SESSION_COOKIE_NAME`] and [`FORMS_AUTH_COOKIE_NAME`]
    /// cookies, plus a non-empty XSRF token. All missing fields are
    /// collected and reported in a single error.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError::SessionValidation`] listing every missing
    /// field when any required material is absent.
    pub fn validate(&self) -> Result<()> {
        let missing = self.missing_fields();
        if missing.is_empty() {
            return Ok(());
        }
        Err(ClientError::SessionValidation {
            message: format!("missing fields: {}", missing.join(", ")),
        })
    }

    /// Returns the session cookies.
    pub fn cookies(&self) -> &[Cookie] {
        &self.cookies
    }

    /// Returns the XSRF token.
    pub fn xsrf_token(&self) -> &str {
        &self.xsrf_token
    }

    /// Returns names of required authentication fields that are missing
    /// or empty.
    fn missing_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        for &name in REQUIRED_COOKIE_NAMES {
            let found = self
                .cookies
                .iter()
                .any(|c| c.name == name && !c.value.is_empty());
            if !found {
                missing.push(name);
            }
        }
        if self.xsrf_token.is_empty() {
            missing.push("xsrf_token");
        }
        missing
    }
}

impl Clone for Session {
    fn clone(&self) -> Self {
        Self {
            cookies: clone_cookies(&self.cookies),
            xsrf_token: self.xsrf_token.clone(),
        }
    }
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cookie_names: Vec<&str> = self.cookies.iter().map(|c| c.name.as_str()).collect();
        f.debug_struct("Session")
            .field("cookies", &cookie_names)
            .field("xsrf_token", &"[REDACTED]")
            .finish()
    }
}

/// Deep-copies a slice of cookies so the caller retains no shared references.
fn clone_cookies(cookies: &[Cookie]) -> Vec<Cookie> {
    cookies
        .iter()
        .map(|c| Cookie {
            name: c.name.clone(),
            value: c.value.clone(),
            domain: c.domain.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::test_session;

    #[test]
    fn valid_session_passes_validation() {
        let session = test_session();
        assert!(session.validate().is_ok());
    }

    #[test]
    fn missing_session_cookie_fails_validation() {
        let session = Session::new(
            vec![Cookie::new(
                FORMS_AUTH_COOKIE_NAME,
                "auth-456",
                COOKIE_DOMAIN,
            )],
            "xsrf-789",
        );

        let err = session.validate().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains(SESSION_COOKIE_NAME),
            "error should mention {SESSION_COOKIE_NAME}, got: {msg}"
        );
    }

    #[test]
    fn missing_auth_cookie_fails_validation() {
        let session = Session::new(
            vec![Cookie::new(
                SESSION_COOKIE_NAME,
                "session-123",
                COOKIE_DOMAIN,
            )],
            "xsrf-789",
        );

        let err = session.validate().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains(FORMS_AUTH_COOKIE_NAME),
            "error should mention {FORMS_AUTH_COOKIE_NAME}, got: {msg}"
        );
    }

    #[test]
    fn empty_xsrf_token_fails_validation() {
        let session = Session::new(
            vec![
                Cookie::new(SESSION_COOKIE_NAME, "session-123", COOKIE_DOMAIN),
                Cookie::new(FORMS_AUTH_COOKIE_NAME, "auth-456", COOKIE_DOMAIN),
            ],
            "",
        );

        let err = session.validate().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("xsrf_token"),
            "error should mention xsrf_token, got: {msg}"
        );
    }

    #[test]
    fn multiple_missing_fields_all_reported() {
        let session = Session::new(vec![], "");

        let err = session.validate().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains(SESSION_COOKIE_NAME),
            "error should mention {SESSION_COOKIE_NAME}, got: {msg}"
        );
        assert!(
            msg.contains(FORMS_AUTH_COOKIE_NAME),
            "error should mention {FORMS_AUTH_COOKIE_NAME}, got: {msg}"
        );
        assert!(
            msg.contains("xsrf_token"),
            "error should mention xsrf_token, got: {msg}"
        );
    }

    #[test]
    fn from_cookies_extracts_xsrf_token() {
        let cookies = vec![
            Cookie::new(SESSION_COOKIE_NAME, "session-123", COOKIE_DOMAIN),
            Cookie::new(FORMS_AUTH_COOKIE_NAME, "auth-456", COOKIE_DOMAIN),
            Cookie::new(
                REQUEST_VERIFICATION_COOKIE_NAME,
                "xsrf-from-cookie",
                COOKIE_DOMAIN,
            ),
        ];

        let session = Session::from_cookies(cookies).unwrap();
        assert_eq!(session.xsrf_token(), "xsrf-from-cookie");
        assert!(session.validate().is_ok());
    }

    #[test]
    fn from_cookies_rejects_missing_verification_cookie() {
        let cookies = vec![
            Cookie::new(SESSION_COOKIE_NAME, "session-123", COOKIE_DOMAIN),
            Cookie::new(FORMS_AUTH_COOKIE_NAME, "auth-456", COOKIE_DOMAIN),
        ];

        let err = Session::from_cookies(cookies).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains(REQUEST_VERIFICATION_COOKIE_NAME),
            "error should mention {REQUEST_VERIFICATION_COOKIE_NAME}, got: {msg}"
        );
    }

    #[test]
    fn debug_shows_cookie_names_but_redacts_values() {
        let session = test_session();
        let debug = format!("{session:?}");

        // Cookie names should appear
        assert!(
            debug.contains(SESSION_COOKIE_NAME),
            "Debug should show cookie name {SESSION_COOKIE_NAME}, got: {debug}"
        );
        assert!(
            debug.contains(FORMS_AUTH_COOKIE_NAME),
            "Debug should show cookie name {FORMS_AUTH_COOKIE_NAME}, got: {debug}"
        );

        // Values must NOT appear
        assert!(
            !debug.contains("session-123"),
            "Debug must not contain cookie value 'session-123'"
        );
        assert!(
            !debug.contains("auth-456"),
            "Debug must not contain cookie value 'auth-456'"
        );
        assert!(
            !debug.contains("xsrf-789"),
            "Debug must not contain xsrf_token value 'xsrf-789'"
        );

        // Redaction markers should appear
        assert!(
            debug.contains("[REDACTED]"),
            "Debug should contain [REDACTED], got: {debug}"
        );
    }

    #[test]
    fn clone_produces_independent_copy() {
        let original = test_session();
        let cloned = original.clone();

        assert_eq!(original.xsrf_token(), cloned.xsrf_token());
        assert_eq!(original.cookies().len(), cloned.cookies().len());

        // Verify the cloned cookies are structurally equal but independent
        for (orig, copy) in original.cookies().iter().zip(cloned.cookies().iter()) {
            assert_eq!(orig.name(), copy.name());
            assert_eq!(orig.value(), copy.value());
            assert_eq!(orig.domain(), copy.domain());
            // Pointers must differ (independent allocations)
            assert!(
                !std::ptr::eq(orig, copy),
                "cloned cookies must not share pointers"
            );
        }
    }

    #[test]
    fn cookie_with_empty_value_treated_as_missing() {
        let session = Session::new(
            vec![
                Cookie::new(SESSION_COOKIE_NAME, "", COOKIE_DOMAIN),
                Cookie::new(FORMS_AUTH_COOKIE_NAME, "auth-456", COOKIE_DOMAIN),
            ],
            "xsrf-789",
        );

        let err = session.validate().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains(SESSION_COOKIE_NAME),
            "empty cookie value should be treated as missing, got: {msg}"
        );
    }

    #[test]
    fn validation_error_is_auth_error() {
        let session = Session::new(vec![], "");
        let err = session.validate().unwrap_err();
        assert!(
            err.is_auth_error(),
            "SessionValidation should be classified as auth error"
        );
    }

    #[test]
    fn getters_return_correct_values() {
        let session = test_session();
        assert_eq!(session.xsrf_token(), "xsrf-789");
        assert_eq!(session.cookies().len(), 2);
        assert_eq!(session.cookies()[0].name(), SESSION_COOKIE_NAME);
        assert_eq!(session.cookies()[0].value(), "session-123");
        assert_eq!(session.cookies()[0].domain(), COOKIE_DOMAIN);
    }
}
