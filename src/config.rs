//! Credential configuration loading for VolumeLeaders login.
//!
//! Credentials are resolved from environment variables first, then from the
//! XDG config file at `~/.config/volumeleaders-agent/config.json` when neither
//! credential environment variable is set. Credential values are treated as
//! secrets and are redacted from [`Debug`] output.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tracing::{debug, warn};

use crate::error::{ClientError, Result};

/// Environment variable for the VolumeLeaders username or email address.
pub const ENV_USERNAME: &str = "VL_USERNAME";

/// Environment variable for the VolumeLeaders password.
pub const ENV_PASSWORD: &str = "VL_PASSWORD";

/// Config directory name within the XDG user config dir.
pub const CONFIG_DIR: &str = "volumeleaders-agent";

/// File name for credential configuration.
pub const CONFIG_FILE: &str = "config.json";

/// Username and password credentials for VolumeLeaders login.
#[derive(Clone, Deserialize, PartialEq, Eq)]
pub struct Credentials {
    username: String,
    password: String,
}

impl Credentials {
    /// Creates credentials from username and password values.
    #[must_use]
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }

    /// Returns the username or email address.
    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns the password.
    #[must_use]
    pub fn password(&self) -> &str {
        &self.password
    }

    fn validate(&self, source: &str) -> Result<()> {
        let mut missing = Vec::new();
        if self.username.is_empty() {
            missing.push("username");
        }
        if self.password.is_empty() {
            missing.push("password");
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(ClientError::SessionValidation {
                message: format!(
                    "credential {source} has empty required field(s): {}",
                    missing.join(", ")
                ),
            })
        }
    }
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("username", &"[REDACTED]")
            .field("password", &"[REDACTED]")
            .finish()
    }
}

/// Source used to resolve VolumeLeaders credentials.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CredentialSource {
    /// Credentials came from `VL_USERNAME` and `VL_PASSWORD`.
    Environment,
    /// Credentials came from an XDG config file.
    ConfigFile(PathBuf),
}

impl CredentialSource {
    /// Returns a stable source kind for diagnostics.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Environment => "environment",
            Self::ConfigFile(_) => "xdg_config",
        }
    }

    /// Returns the config file path when this source is an XDG config file.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Environment => None,
            Self::ConfigFile(path) => Some(path),
        }
    }
}

/// Credentials plus the source that supplied them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedCredentials {
    credentials: Credentials,
    source: CredentialSource,
}

impl ResolvedCredentials {
    /// Creates resolved credentials with a source marker.
    #[must_use]
    pub fn new(credentials: Credentials, source: CredentialSource) -> Self {
        Self {
            credentials,
            source,
        }
    }

    /// Returns the resolved credentials.
    #[must_use]
    pub fn credentials(&self) -> &Credentials {
        &self.credentials
    }

    /// Returns the source that supplied the credentials.
    #[must_use]
    pub fn source(&self) -> &CredentialSource {
        &self.source
    }
}

/// Resolve credentials from environment variables or the XDG config file.
///
/// Environment variables take precedence. If either `VL_USERNAME` or
/// `VL_PASSWORD` is present, both must be present and non-empty. When neither
/// environment variable is set, the resolver falls back to
/// `~/.config/volumeleaders-agent/config.json`.
///
/// # Errors
///
/// Returns [`ClientError::SessionValidation`] when credentials are missing,
/// incomplete, or the config file cannot be read or parsed. Returns
/// [`ClientError::Cache`] when the XDG config directory is unavailable and no
/// environment credentials were supplied.
pub fn resolve_credentials() -> Result<ResolvedCredentials> {
    match credentials_from_env()? {
        Some(credentials) => Ok(ResolvedCredentials::new(
            credentials,
            CredentialSource::Environment,
        )),
        None => {
            let path = default_config_path()?;
            load_credentials_at(&path)
        }
    }
}

/// Returns `true` when complete credentials are available from env or config.
#[must_use]
pub fn credentials_available() -> bool {
    resolve_credentials().is_ok()
}

/// Returns the default XDG credential config file path.
///
/// # Errors
///
/// Returns [`ClientError::Cache`] when the XDG config directory is unavailable.
pub fn default_config_path() -> Result<PathBuf> {
    dirs::config_dir()
        .map(|base| config_path_from_base(&base))
        .ok_or_else(|| ClientError::Cache("XDG config directory not available".to_string()))
}

/// Builds the full credential config file path from a base config directory.
#[must_use]
pub fn config_path_from_base(base: &Path) -> PathBuf {
    base.join(CONFIG_DIR).join(CONFIG_FILE)
}

#[cfg(test)]
fn resolve_credentials_with_path(path: &Path) -> Result<ResolvedCredentials> {
    match credentials_from_env()? {
        Some(credentials) => Ok(ResolvedCredentials::new(
            credentials,
            CredentialSource::Environment,
        )),
        None => load_credentials_at(path),
    }
}

fn credentials_from_env() -> Result<Option<Credentials>> {
    let username = env::var(ENV_USERNAME).ok();
    let password = env::var(ENV_PASSWORD).ok();

    match (username, password) {
        (Some(username), Some(password)) => {
            let credentials = Credentials::new(username, password);
            credentials.validate("environment variables")?;
            Ok(Some(credentials))
        }
        (Some(_), None) => Err(ClientError::SessionValidation {
            message: format!("{ENV_PASSWORD} environment variable not set"),
        }),
        (None, Some(_)) => Err(ClientError::SessionValidation {
            message: format!("{ENV_USERNAME} environment variable not set"),
        }),
        (None, None) => Ok(None),
    }
}

fn load_credentials_at(path: &Path) -> Result<ResolvedCredentials> {
    if !path.exists() {
        return Err(ClientError::SessionValidation {
            message: format!(
                "set {ENV_USERNAME} and {ENV_PASSWORD} environment variables or create {} with username and password fields",
                path.display()
            ),
        });
    }

    let data = fs::read_to_string(path).map_err(|err| {
        warn!(?path, %err, "failed to read credential config");
        ClientError::SessionValidation {
            message: format!("failed to read credential config {}: {err}", path.display()),
        }
    })?;
    let credentials: Credentials = serde_json::from_str(&data).map_err(|err| {
        warn!(?path, %err, "failed to deserialize credential config");
        ClientError::SessionValidation {
            message: format!(
                "failed to parse credential config {}: {err}",
                path.display()
            ),
        }
    })?;
    credentials.validate(&format!("config file {}", path.display()))?;

    debug!(?path, "loaded credential config");
    Ok(ResolvedCredentials::new(
        credentials,
        CredentialSource::ConfigFile(path.to_path_buf()),
    ))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Mutex;

    use tempfile::TempDir;

    use super::*;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn config_path_for_temp(dir: &TempDir) -> PathBuf {
        config_path_from_base(dir.path())
    }

    fn with_clean_env(test: impl FnOnce()) {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let old_username = env::var(ENV_USERNAME).ok();
        let old_password = env::var(ENV_PASSWORD).ok();
        unsafe {
            env::remove_var(ENV_USERNAME);
            env::remove_var(ENV_PASSWORD);
        }

        test();

        unsafe {
            match old_username {
                Some(value) => env::set_var(ENV_USERNAME, value),
                None => env::remove_var(ENV_USERNAME),
            }
            match old_password {
                Some(value) => env::set_var(ENV_PASSWORD, value),
                None => env::remove_var(ENV_PASSWORD),
            }
        }
    }

    #[test]
    fn resolves_environment_credentials_first() {
        with_clean_env(|| {
            let tmp = TempDir::new().expect("temp dir");
            let path = config_path_for_temp(&tmp);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(
                &path,
                r#"{"username":"config@example.com","password":"config-pass"}"#,
            )
            .unwrap();
            unsafe {
                env::set_var(ENV_USERNAME, "env@example.com");
                env::set_var(ENV_PASSWORD, "env-pass");
            }

            let resolved = resolve_credentials_with_path(&path).expect("credentials");

            assert_eq!(resolved.credentials().username(), "env@example.com");
            assert_eq!(resolved.credentials().password(), "env-pass");
            assert_eq!(resolved.source(), &CredentialSource::Environment);
        });
    }

    #[test]
    fn resolves_config_credentials_when_env_is_absent() {
        with_clean_env(|| {
            let tmp = TempDir::new().expect("temp dir");
            let path = config_path_for_temp(&tmp);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(
                &path,
                r#"{"username":"config@example.com","password":"config-pass"}"#,
            )
            .unwrap();

            let resolved = resolve_credentials_with_path(&path).expect("credentials");

            assert_eq!(resolved.credentials().username(), "config@example.com");
            assert_eq!(resolved.credentials().password(), "config-pass");
            assert_eq!(
                resolved.source(),
                &CredentialSource::ConfigFile(path.to_path_buf())
            );
            assert_eq!(resolved.source().kind(), "xdg_config");
        });
    }

    #[test]
    fn partial_environment_credentials_do_not_fall_back_to_config() {
        with_clean_env(|| {
            let tmp = TempDir::new().expect("temp dir");
            let path = config_path_for_temp(&tmp);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(
                &path,
                r#"{"username":"config@example.com","password":"config-pass"}"#,
            )
            .unwrap();
            unsafe {
                env::set_var(ENV_USERNAME, "env@example.com");
            }

            let err = resolve_credentials_with_path(&path).unwrap_err();

            assert!(err.to_string().contains(ENV_PASSWORD));
        });
    }

    #[test]
    fn missing_config_reports_env_and_config_recovery() {
        with_clean_env(|| {
            let tmp = TempDir::new().expect("temp dir");
            let path = config_path_for_temp(&tmp);

            let err = resolve_credentials_with_path(&path).unwrap_err();
            let message = err.to_string();

            assert!(message.contains(ENV_USERNAME));
            assert!(message.contains(ENV_PASSWORD));
            assert!(message.contains(path.to_string_lossy().as_ref()));
        });
    }

    #[test]
    fn config_requires_non_empty_fields() {
        with_clean_env(|| {
            let tmp = TempDir::new().expect("temp dir");
            let path = config_path_for_temp(&tmp);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, r#"{"username":"","password":""}"#).unwrap();

            let err = resolve_credentials_with_path(&path).unwrap_err();
            let message = err.to_string();

            assert!(message.contains("username"));
            assert!(message.contains("password"));
        });
    }

    #[test]
    fn config_rejects_invalid_json() {
        with_clean_env(|| {
            let tmp = TempDir::new().expect("temp dir");
            let path = config_path_for_temp(&tmp);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, "not json").unwrap();

            let err = resolve_credentials_with_path(&path).unwrap_err();

            assert!(matches!(err, ClientError::SessionValidation { .. }));
            assert!(err.to_string().contains("failed to parse"));
        });
    }

    #[test]
    fn debug_redacts_credentials() {
        let credentials = Credentials::new("user@example.com", "secret");

        let debug = format!("{credentials:?}");

        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("user@example.com"));
        assert!(!debug.contains("secret"));
    }
}
