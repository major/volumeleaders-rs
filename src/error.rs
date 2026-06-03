//! Error types and result alias for the VolumeLeaders client.

/// Convenient result type used by this crate.
pub type Result<T> = std::result::Result<T, ClientError>;

/// Errors returned by the VolumeLeaders client.
#[derive(thiserror::Error)]
pub enum ClientError {
    /// VolumeLeaders returned a non-success HTTP status.
    ///
    /// The `body` field is preserved for programmatic inspection but is
    /// intentionally excluded from both `Display` and `Debug` output to
    /// prevent authenticated response content from leaking into logs.
    #[error("VolumeLeaders API returned HTTP {code} for {url}")]
    Status {
        /// The HTTP status code.
        code: u16,
        /// The request URL that produced the error.
        url: String,
        /// The response body (never shown in Display/Debug).
        body: String,
    },

    /// The response body exceeded the configured size limit.
    #[error("response body exceeded {limit} byte limit (got {actual} bytes)")]
    BodyLimit {
        /// The configured maximum body size in bytes.
        limit: usize,
        /// The actual body size in bytes.
        actual: usize,
    },

    /// VolumeLeaders returned content that could not be decoded as the
    /// expected response type.
    ///
    /// The `actual` field may contain partial authenticated response content
    /// and is redacted from `Debug` output.
    #[error("unexpected content from {url}: expected {expected}")]
    UnexpectedContent {
        /// What we expected to receive.
        expected: String,
        /// What we actually received (redacted in Debug).
        actual: String,
        /// The request URL.
        url: String,
    },

    /// The browser session cookies have expired or been invalidated.
    #[error("session expired (redirected from {url})")]
    SessionExpired {
        /// The URL that triggered the session-expired redirect.
        url: String,
    },

    /// The session is missing required authentication material.
    #[error("invalid session: {message}")]
    SessionValidation {
        /// Description of what is missing or invalid.
        message: String,
    },

    /// VolumeLeaders returned HTTP 429 with an optional retry delay.
    #[error("rate limited{}", retry_after.map(|s| format!(" (retry after {s}s)")).unwrap_or_default())]
    RateLimit {
        /// Seconds to wait before retrying, if the server provided one.
        retry_after: Option<u64>,
    },

    /// A redirect attempted to leave the original request origin.
    ///
    /// The `to` field may contain session tokens in query parameters and
    /// is redacted from `Debug` output.
    #[error("cross-origin redirect blocked from {from}")]
    CrossOriginRedirect {
        /// The original request URL.
        from: String,
        /// The redirect target (redacted in Debug).
        to: String,
    },

    /// An HTTP transport error occurred.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization or deserialization failed.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// A filesystem or I/O operation failed.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// VolumeLeaders login failed (bad credentials, account locked, etc.).
    #[error("login failed: {reason}")]
    LoginFailed {
        /// Human-readable reason for the login failure.
        reason: String,
    },

    /// Failed to read from or write to the session cookie cache.
    #[error("session cache error: {0}")]
    Cache(String),
}

// Manual Debug impl to redact sensitive fields that may contain
// authenticated response content, session tokens, or redirect URLs
// with query-string credentials.
impl std::fmt::Debug for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Status { code, url, .. } => f
                .debug_struct("Status")
                .field("code", code)
                .field("url", url)
                .field("body", &"[REDACTED BODY]")
                .finish(),

            Self::BodyLimit { limit, actual } => f
                .debug_struct("BodyLimit")
                .field("limit", limit)
                .field("actual", actual)
                .finish(),

            Self::UnexpectedContent { expected, url, .. } => f
                .debug_struct("UnexpectedContent")
                .field("expected", expected)
                .field("actual", &"[REDACTED]")
                .field("url", url)
                .finish(),

            Self::SessionExpired { url } => {
                f.debug_struct("SessionExpired").field("url", url).finish()
            }

            Self::SessionValidation { message } => f
                .debug_struct("SessionValidation")
                .field("message", message)
                .finish(),

            Self::RateLimit { retry_after } => f
                .debug_struct("RateLimit")
                .field("retry_after", retry_after)
                .finish(),

            Self::CrossOriginRedirect { from, .. } => f
                .debug_struct("CrossOriginRedirect")
                .field("from", from)
                .field("to", &"[REDACTED REDIRECT]")
                .finish(),

            Self::Http(err) => f.debug_tuple("Http").field(err).finish(),

            Self::Json(err) => f.debug_tuple("Json").field(err).finish(),

            Self::Io(err) => f.debug_tuple("Io").field(err).finish(),

            Self::LoginFailed { reason } => f
                .debug_struct("LoginFailed")
                .field("reason", reason)
                .finish(),

            Self::Cache(reason) => f.debug_struct("Cache").field("reason", reason).finish(),
        }
    }
}

impl ClientError {
    /// Returns `true` if this error indicates missing, invalid, or expired
    /// authentication material.
    pub fn is_auth_error(&self) -> bool {
        matches!(
            self,
            Self::SessionExpired { .. } | Self::SessionValidation { .. }
        )
    }

    /// Returns `true` if this error is an HTTP 429 rate limit.
    pub fn is_rate_limit(&self) -> bool {
        matches!(self, Self::RateLimit { .. })
    }

    /// Returns `true` if this error is likely transient and the request
    /// could succeed on retry.
    ///
    /// Rate limits and HTTP transport errors (timeouts, connection resets)
    /// are considered retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::RateLimit { .. } | Self::Http(_))
    }

    /// Returns `true` if this error specifically indicates an expired session.
    pub fn is_session_expired(&self) -> bool {
        matches!(self, Self::SessionExpired { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_debug_redacts_body() {
        let err = ClientError::Status {
            code: 403,
            url: "https://example.com/api".into(),
            body: "secret account data with tokens and credentials".into(),
        };

        let debug = format!("{err:?}");
        assert!(
            debug.contains("[REDACTED BODY]"),
            "Debug should contain [REDACTED BODY], got: {debug}"
        );
        assert!(
            !debug.contains("secret account data"),
            "Debug must not contain the actual body content"
        );
    }

    #[test]
    fn status_display_omits_body() {
        let err = ClientError::Status {
            code: 401,
            url: "https://example.com/login".into(),
            body: "sensitive auth response".into(),
        };

        let display = err.to_string();
        assert_eq!(
            display,
            "VolumeLeaders API returned HTTP 401 for https://example.com/login"
        );
        assert!(!display.contains("sensitive auth response"));
    }

    #[test]
    fn unexpected_content_debug_redacts_actual() {
        let err = ClientError::UnexpectedContent {
            expected: "application/json".into(),
            actual: "partial auth token response data".into(),
            url: "https://example.com/data".into(),
        };

        let debug = format!("{err:?}");
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("partial auth token"));
    }

    #[test]
    fn cross_origin_redirect_debug_redacts_to() {
        let err = ClientError::CrossOriginRedirect {
            from: "https://volumeleaders.com/api".into(),
            to: "https://evil.com/steal?session=abc123".into(),
        };

        let debug = format!("{err:?}");
        assert!(debug.contains("[REDACTED REDIRECT]"));
        assert!(!debug.contains("evil.com"));
        assert!(!debug.contains("session=abc123"));
    }

    #[test]
    fn is_auth_error_positive_cases() {
        assert!(
            ClientError::SessionExpired {
                url: "https://example.com".into()
            }
            .is_auth_error()
        );

        assert!(
            ClientError::SessionValidation {
                message: "missing cookies".into()
            }
            .is_auth_error()
        );
    }

    #[test]
    fn is_auth_error_negative_cases() {
        assert!(!ClientError::RateLimit { retry_after: None }.is_auth_error());
        assert!(
            !ClientError::BodyLimit {
                limit: 1024,
                actual: 2048
            }
            .is_auth_error()
        );
    }

    #[test]
    fn is_rate_limit_positive() {
        assert!(
            ClientError::RateLimit {
                retry_after: Some(30)
            }
            .is_rate_limit()
        );

        assert!(ClientError::RateLimit { retry_after: None }.is_rate_limit());
    }

    #[test]
    fn is_rate_limit_negative() {
        assert!(
            !ClientError::SessionExpired {
                url: "https://example.com".into()
            }
            .is_rate_limit()
        );
    }

    #[test]
    fn is_retryable_covers_rate_limit_and_http() {
        assert!(
            ClientError::RateLimit {
                retry_after: Some(5)
            }
            .is_retryable()
        );

        // Io errors are not retryable
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        assert!(!ClientError::Io(io_err).is_retryable());
    }

    #[test]
    fn is_session_expired_positive() {
        assert!(
            ClientError::SessionExpired {
                url: "https://example.com".into()
            }
            .is_session_expired()
        );
    }

    #[test]
    fn is_session_expired_negative() {
        assert!(
            !ClientError::SessionValidation {
                message: "missing".into()
            }
            .is_session_expired()
        );
    }

    #[test]
    fn debug_impl_covers_all_variants() {
        let serde_err = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");

        let variants: Vec<ClientError> = vec![
            ClientError::Status {
                code: 500,
                url: "https://example.com".into(),
                body: "secret".into(),
            },
            ClientError::BodyLimit {
                limit: 1024,
                actual: 2048,
            },
            ClientError::UnexpectedContent {
                expected: "json".into(),
                actual: "html".into(),
                url: "https://example.com".into(),
            },
            ClientError::SessionExpired {
                url: "https://example.com".into(),
            },
            ClientError::SessionValidation {
                message: "missing cookies".into(),
            },
            ClientError::RateLimit {
                retry_after: Some(30),
            },
            ClientError::CrossOriginRedirect {
                from: "https://a.com".into(),
                to: "https://b.com".into(),
            },
            ClientError::Json(serde_err),
            ClientError::Io(io_err),
        ];

        // Every variant must produce non-empty Debug output without panicking
        for variant in &variants {
            let debug = format!("{variant:?}");
            assert!(!debug.is_empty(), "Debug output must not be empty");
        }

        // Redaction checks on specific variants
        let status_debug = format!("{:?}", variants[0]);
        assert!(status_debug.contains("[REDACTED BODY]"));
        assert!(!status_debug.contains("secret"));

        let content_debug = format!("{:?}", variants[2]);
        assert!(content_debug.contains("[REDACTED]"));
        assert!(!content_debug.contains("html"));

        let redirect_debug = format!("{:?}", variants[6]);
        assert!(redirect_debug.contains("[REDACTED REDIRECT]"));
        assert!(!redirect_debug.contains("b.com"));
    }
}
