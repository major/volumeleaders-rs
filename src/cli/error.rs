//! Structured CLI error rendering and semantic exit-code mapping.

use std::io::{self, Write};

use serde::Serialize;

use crate::ClientError;

/// Exit code used by clap for invalid flags, arguments, or command shape.
pub const EXIT_USAGE_ERROR: i32 = 2;
/// Exit code for missing, expired, or unusable authentication.
pub const EXIT_AUTH_ERROR: i32 = 3;
/// Exit code for transport-level HTTP failures.
pub const EXIT_HTTP_ERROR: i32 = 4;
/// Exit code for VolumeLeaders API error responses.
pub const EXIT_API_ERROR: i32 = 5;
/// Exit code for JSON parsing, serialization, or transformation failures.
pub const EXIT_JSON_ERROR: i32 = 6;
/// Exit code reserved for strict empty result handling.
pub const EXIT_EMPTY_RESULT: i32 = 7;

/// Stable machine-readable CLI runtime error kinds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CliErrorKind {
    /// Invalid flags, arguments, or command shape beyond clap's parser checks.
    UsageError,
    /// Missing, expired, or unusable authentication.
    AuthError,
    /// Network, DNS, timeout, TLS, or HTTP-client failure.
    HttpError,
    /// VolumeLeaders returned an error response.
    ApiError,
    /// Response or output could not be parsed, serialized, or transformed.
    JsonError,
    /// A command returned no rows while strict empty handling was requested.
    EmptyResult,
}

impl CliErrorKind {
    /// Returns the stable JSON `error.kind` string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UsageError => "usage_error",
            Self::AuthError => "auth_error",
            Self::HttpError => "http_error",
            Self::ApiError => "api_error",
            Self::JsonError => "json_error",
            Self::EmptyResult => "empty_result",
        }
    }

    /// Returns the semantic process exit code for this error kind.
    #[must_use]
    pub const fn exit_code(self) -> i32 {
        match self {
            Self::UsageError => EXIT_USAGE_ERROR,
            Self::AuthError => EXIT_AUTH_ERROR,
            Self::HttpError => EXIT_HTTP_ERROR,
            Self::ApiError => EXIT_API_ERROR,
            Self::JsonError => EXIT_JSON_ERROR,
            Self::EmptyResult => EXIT_EMPTY_RESULT,
        }
    }
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope<'a> {
    ok: bool,
    error: ErrorBody<'a>,
}

#[derive(Debug, Serialize)]
struct ErrorBody<'a> {
    kind: &'static str,
    message: &'a str,
}

/// Writes a structured JSON runtime error to stderr and returns its exit code.
pub fn render_error(kind: CliErrorKind, message: impl AsRef<str>) -> i32 {
    let message = message.as_ref();
    let mut stderr = io::stderr().lock();

    if let Err(err) = write_error(&mut stderr, kind, message) {
        let fallback = serde_json::json!({
            "ok": false,
            "error": {
                "kind": "json_error",
                "message": format!("failed to render CLI error: {err}"),
            }
        });
        let _ = writeln!(stderr, "{fallback}");
    }

    kind.exit_code()
}

/// Writes a structured usage error and returns exit code 2.
pub fn usage_error(message: impl AsRef<str>) -> i32 {
    render_error(CliErrorKind::UsageError, message)
}

/// Writes a structured JSON error and returns exit code 6.
pub fn json_error(message: impl AsRef<str>) -> i32 {
    render_error(CliErrorKind::JsonError, message)
}

/// Writes a structured empty-result error and returns exit code 7.
pub fn empty_result(message: impl AsRef<str>) -> i32 {
    render_error(CliErrorKind::EmptyResult, message)
}

/// Writes a structured client error and returns its semantic exit code.
pub fn client_error(err: &ClientError) -> i32 {
    render_error(client_error_kind(err), err.to_string())
}

/// Classifies a client error into the CLI's stable runtime error kinds.
#[must_use]
pub fn client_error_kind(err: &ClientError) -> CliErrorKind {
    match err {
        ClientError::SessionExpired { .. }
        | ClientError::SessionValidation { .. }
        | ClientError::LoginFailed { .. } => CliErrorKind::AuthError,
        ClientError::Http(_) => CliErrorKind::HttpError,
        ClientError::Status { .. }
        | ClientError::RateLimit { .. }
        | ClientError::CrossOriginRedirect { .. } => CliErrorKind::ApiError,
        ClientError::BodyLimit { .. }
        | ClientError::UnexpectedContent { .. }
        | ClientError::Json(_)
        | ClientError::Io(_) => CliErrorKind::JsonError,
        ClientError::Cache(_) => CliErrorKind::AuthError,
    }
}

fn write_error<W: Write>(mut writer: W, kind: CliErrorKind, message: &str) -> io::Result<()> {
    let envelope = ErrorEnvelope {
        ok: false,
        error: ErrorBody {
            kind: kind.as_str(),
            message,
        },
    };
    serde_json::to_writer(&mut writer, &envelope)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    writer.write_all(b"\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_structured_error_envelope() {
        let mut buffer = Vec::new();

        write_error(
            &mut buffer,
            CliErrorKind::AuthError,
            "login credentials are missing",
        )
        .unwrap();

        let value: serde_json::Value = serde_json::from_slice(&buffer).unwrap();
        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["kind"], "auth_error");
        assert_eq!(value["error"]["message"], "login credentials are missing");
    }

    #[test]
    fn error_kinds_have_semantic_exit_codes() {
        assert_eq!(CliErrorKind::UsageError.exit_code(), 2);
        assert_eq!(CliErrorKind::AuthError.exit_code(), 3);
        assert_eq!(CliErrorKind::HttpError.exit_code(), 4);
        assert_eq!(CliErrorKind::ApiError.exit_code(), 5);
        assert_eq!(CliErrorKind::JsonError.exit_code(), 6);
        assert_eq!(CliErrorKind::EmptyResult.exit_code(), 7);
    }

    #[test]
    fn client_errors_are_classified_for_cli_contract() {
        assert_eq!(
            client_error_kind(&ClientError::SessionValidation {
                message: "missing cookies".into()
            }),
            CliErrorKind::AuthError
        );
        assert_eq!(
            client_error_kind(&ClientError::Status {
                code: 500,
                url: "https://example.com".into(),
                body: "server error".into()
            }),
            CliErrorKind::ApiError
        );
        assert_eq!(
            client_error_kind(&ClientError::Json(
                serde_json::from_str::<serde_json::Value>("not json").unwrap_err()
            )),
            CliErrorKind::JsonError
        );
        assert_eq!(
            client_error_kind(&ClientError::RateLimit {
                retry_after: Some(30)
            }),
            CliErrorKind::ApiError
        );
        assert_eq!(
            client_error_kind(&ClientError::CrossOriginRedirect {
                from: "https://www.volumeleaders.com".into(),
                to: "https://example.com".into()
            }),
            CliErrorKind::ApiError
        );
        assert_eq!(
            client_error_kind(&ClientError::BodyLimit {
                limit: 10,
                actual: 11
            }),
            CliErrorKind::JsonError
        );
        assert_eq!(
            client_error_kind(&ClientError::UnexpectedContent {
                expected: "JSON".into(),
                actual: "HTML login page".into(),
                url: "https://www.volumeleaders.com".into()
            }),
            CliErrorKind::JsonError
        );
        assert_eq!(
            client_error_kind(&ClientError::Io(std::io::Error::other("disk error"))),
            CliErrorKind::JsonError
        );
        assert_eq!(
            client_error_kind(&ClientError::LoginFailed {
                reason: "bad credentials".into()
            }),
            CliErrorKind::AuthError
        );
        assert_eq!(
            client_error_kind(&ClientError::Cache("unreadable".into())),
            CliErrorKind::AuthError
        );
    }
}
