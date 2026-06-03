use std::env;

use tracing::{debug, warn};

use crate::cli::error::client_error;
use crate::{Client, ClientError, Session};

/// Cookie domain used for VolumeLeaders authentication.
pub const VL_DOMAIN: &str = "volumeleaders.com";

/// Environment variable for the VolumeLeaders username (email).
const ENV_USERNAME: &str = "VL_USERNAME";

/// Environment variable for the VolumeLeaders password.
const ENV_PASSWORD: &str = "VL_PASSWORD";

/// Build a VolumeLeaders client from environment-variable credentials.
///
/// Flow:
/// 1. Reads `VL_USERNAME` and `VL_PASSWORD` from the environment.
/// 2. Tries to load a cached session from `~/.cache/volumeleaders-agent/`.
/// 3. If the cached session is valid and current, refreshes its XSRF token.
/// 4. If no valid cached session exists, logs in with credentials and
///    saves the new session to the cache.
/// 5. Returns an authenticated [`Client`] ready for API calls.
pub async fn make_client_from_env() -> Result<Client, i32> {
    let username = match env::var(ENV_USERNAME) {
        Ok(u) if !u.is_empty() => u,
        _ => {
            return Err(client_error(&ClientError::SessionValidation {
                message: format!("{ENV_USERNAME} environment variable not set or empty"),
            }));
        }
    };

    let password = match env::var(ENV_PASSWORD) {
        Ok(p) if !p.is_empty() => p,
        _ => {
            return Err(client_error(&ClientError::SessionValidation {
                message: format!("{ENV_PASSWORD} environment variable not set or empty"),
            }));
        }
    };

    make_client_with_creds(&username, &password).await
}

/// Build a VolumeLeaders client from explicit credentials.
async fn make_client_with_creds(username: &str, password: &str) -> Result<Client, i32> {
    // Try cached session first.
    if let Some(session) = crate::load_cached_session() {
        debug!("using cached session");
        match build_client_from_session(session).await {
            Ok(client) => return Ok(client),
            Err(err) => {
                warn!(%err, "cached session invalid, will re-login");
                crate::clear_cache();
            }
        }
    }

    // Login with credentials.
    debug!("logging in with credentials");
    let session = match crate::login(username, password).await {
        Ok(s) => s,
        Err(err) => return Err(client_error(&err)),
    };

    // Save session to cache for future invocations.
    if let Err(err) = crate::save_session(&session) {
        warn!(%err, "failed to cache session");
    }

    build_client_from_session(session)
        .await
        .map_err(|err| client_error(&err))
}

/// Build a VolumeLeaders client from environment-variable credentials.
///
/// Kept for backward compatibility in tests; prefer [`make_client_from_env`].
pub async fn make_client() -> Result<Client, i32> {
    make_client_from_env().await
}

/// Convert API errors into CLI exit codes and messages.
pub fn handle_api_error(err: ClientError) -> i32 {
    client_error(&err)
}

/// Build an authenticated client from a session, refreshing the XSRF token.
async fn build_client_from_session(session: Session) -> Result<Client, ClientError> {
    let bootstrap_client = Client::new(session.clone())?;

    let xsrf_token = crate::extract_xsrf_token(&bootstrap_client).await?;

    let refreshed_session = Session::new(session.cookies().to_vec(), xsrf_token);
    Client::new(refreshed_session)
}

#[cfg(test)]
mod tests {
    use crate::ClientError;

    use super::*;

    #[test]
    fn handle_api_error_maps_auth_error() {
        let code = handle_api_error(ClientError::SessionValidation {
            message: "test".into(),
        });
        assert_eq!(code, 3);
    }

    #[test]
    fn handle_api_error_maps_http_error() {
        let code = handle_api_error(ClientError::Status {
            code: 500,
            url: "https://example.com".into(),
            body: "error".into(),
        });
        assert_eq!(code, 5);
    }

    #[tokio::test]
    async fn make_client_from_env_missing_username() {
        unsafe {
            env::remove_var(ENV_USERNAME);
            env::remove_var(ENV_PASSWORD);
        }
        let result = make_client_from_env().await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), 3);
    }

    #[tokio::test]
    async fn make_client_from_env_empty_username() {
        unsafe {
            env::set_var(ENV_USERNAME, "");
            env::remove_var(ENV_PASSWORD);
        }
        let result = make_client_from_env().await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), 3);
    }

    #[tokio::test]
    async fn make_client_from_env_username_set_but_no_password() {
        unsafe {
            env::set_var(ENV_USERNAME, "user@example.com");
            env::remove_var(ENV_PASSWORD);
        }
        let result = make_client_from_env().await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), 3);
    }

    #[tokio::test]
    async fn make_client_from_env_empty_password() {
        unsafe {
            env::set_var(ENV_USERNAME, "user@example.com");
            env::set_var(ENV_PASSWORD, "");
        }
        let result = make_client_from_env().await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), 3);
    }
}
