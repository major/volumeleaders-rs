//! Session cookie cache in the XDG cache directory.
//!
//! Persists authenticated [`Session`] data to
//! `~/.cache/volumeleaders-agent/cookies.json` so that subsequent CLI
//! invocations can reuse a valid session without re-authenticating.
//!
//! Sessions are serialized as JSON. On load, cache entries are validated
//! (required cookies present, XSRF token non-empty). Expired sessions are
//! cleared automatically when used via the client's session-expiry detection.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use tracing::{debug, warn};

use crate::error::{ClientError, Result};
use crate::session::Session;

/// Cache directory name within the XDG user cache dir.
const CACHE_DIR: &str = "volumeleaders-agent";

/// File name for cached session cookies.
const CACHE_FILE: &str = "cookies.json";

/// Loads a cached [`Session`] from the XDG cache directory.
///
/// Returns `None` if no cache file exists, the file is unreadable, or
/// the stored session fails validation.
pub fn load_cached_session() -> Option<Session> {
    let base = match default_cache_base_dir() {
        Ok(d) => d,
        Err(_) => return None,
    };
    load_cached_session_at(&cache_path_from_base(&base))
}

/// Loads a cached [`Session`] from a specific cache file path.
///
/// Returns `None` if the file doesn't exist, can't be read, can't be
/// deserialized, or fails validation.
fn load_cached_session_at(path: &Path) -> Option<Session> {
    if !path.exists() {
        debug!(?path, "no cached session file");
        return None;
    }

    let data = match fs::read_to_string(path) {
        Ok(d) => d,
        Err(err) => {
            warn!(?path, %err, "failed to read cached session");
            return None;
        }
    };

    let session: Session = match serde_json::from_str(&data) {
        Ok(s) => s,
        Err(err) => {
            warn!(?path, %err, "failed to deserialize cached session");
            return None;
        }
    };

    match session.validate() {
        Ok(()) => {
            debug!(?path, "loaded valid cached session");
            Some(session)
        }
        Err(err) => {
            warn!(?path, %err, "cached session failed validation");
            None
        }
    }
}

/// Saves a [`Session`] to the XDG cache directory.
///
/// Creates the cache directory if it doesn't exist. Overwrites any
/// existing cache file.
pub fn save_session(session: &Session) -> Result<()> {
    let base = default_cache_base_dir()?;
    save_session_at(session, &base)
}

/// Saves a [`Session`] to a specific cache file path.
fn save_session_at(session: &Session, base_dir: &Path) -> Result<()> {
    let path = cache_path_from_base(base_dir);

    fs::create_dir_all(path.parent().expect("cache path has parent")).map_err(|err| {
        ClientError::Cache(format!(
            "failed to create cache directory {}: {err}",
            path.parent().unwrap().display(),
        ))
    })?;

    let data = serde_json::to_string(session)?;

    // Write atomically via unique temp file then rename.
    // Use NamedTempFile so temp files are auto-cleaned on failure
    // and avoid racing concurrent writers.
    let parent = path.parent().unwrap();
    let mut tmp = tempfile::NamedTempFile::new_in(parent)
        .map_err(|err| ClientError::Cache(format!("failed to create temp session file: {err}")))?;
    tmp.write_all(data.as_bytes())
        .map_err(|err| ClientError::Cache(format!("failed to write session cache: {err}")))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(tmp.path(), fs::Permissions::from_mode(0o600))
            .map_err(|err| ClientError::Cache(format!("failed to secure session cache: {err}")))?;
    }
    tmp.persist(&path)
        .map_err(|err| ClientError::Cache(format!("failed to persist session cache: {err}")))?;

    debug!(?path, "saved session to cache");
    Ok(())
}

/// Clears the cached session file.
pub fn clear_cache() {
    match default_cache_base_dir() {
        Ok(base) => clear_cache_at(&base),
        Err(err) => {
            warn!(%err, "failed to resolve cache path for clearing");
        }
    }
}

/// Clears the cached session file at a specific base directory.
fn clear_cache_at(base_dir: &Path) {
    let path = cache_path_from_base(base_dir);
    if path.exists() {
        if let Err(err) = fs::remove_file(&path) {
            warn!(?path, %err, "failed to clear session cache");
        } else {
            debug!(?path, "cleared session cache");
        }
    }
}

/// Returns the XDG cache base directory.
fn default_cache_base_dir() -> Result<PathBuf> {
    dirs::cache_dir()
        .ok_or_else(|| ClientError::Cache("XDG cache directory not available".to_string()))
}

/// Builds the full cache file path from a base directory.
fn cache_path_from_base(base: &Path) -> PathBuf {
    base.join(CACHE_DIR).join(CACHE_FILE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{COOKIE_DOMAIN, Cookie, FORMS_AUTH_COOKIE_NAME, SESSION_COOKIE_NAME};
    use tempfile::TempDir;

    /// Returns the cache file path within a TempDir for testing.
    fn cache_path_for_temp(dir: &TempDir) -> PathBuf {
        cache_path_from_base(dir.path())
    }

    /// Builds a minimal valid session for tests.
    fn valid_session() -> Session {
        Session::new(
            vec![
                Cookie::new(SESSION_COOKIE_NAME, "session-123", COOKIE_DOMAIN),
                Cookie::new(FORMS_AUTH_COOKIE_NAME, "auth-456", COOKIE_DOMAIN),
            ],
            "xsrf-789",
        )
    }

    #[test]
    fn save_and_load_session_roundtrip() {
        let tmp = TempDir::new().expect("temp dir");
        let base = tmp.path();

        let session = valid_session();
        save_session_at(&session, base).expect("save should succeed");

        let path = cache_path_for_temp(&tmp);
        let loaded = load_cached_session_at(&path).expect("load should succeed");
        assert_eq!(loaded.xsrf_token(), "xsrf-789");
        assert_eq!(loaded.cookies().len(), 2);
        assert!(loaded.validate().is_ok());
    }

    #[test]
    fn load_returns_none_when_no_cache() {
        let tmp = TempDir::new().expect("temp dir");
        let path = tmp.path().join("nonexistent").join(CACHE_FILE);

        let result = load_cached_session_at(&path);
        assert!(result.is_none());
    }

    #[test]
    fn clear_cache_removes_file() {
        let tmp = TempDir::new().expect("temp dir");
        let base = tmp.path();

        save_session_at(&valid_session(), base).expect("save should succeed");
        let path = cache_path_for_temp(&tmp);
        assert!(load_cached_session_at(&path).is_some());

        clear_cache_at(base);
        assert!(load_cached_session_at(&path).is_none());
    }

    #[test]
    fn load_rejects_invalid_session() {
        let tmp = TempDir::new().expect("temp dir");
        let base = tmp.path();

        // Save a session with no cookies — validation will fail.
        let bad_session = Session::new(vec![], "");
        save_session_at(&bad_session, base).expect("save should succeed");

        // Should return None because validation fails.
        let path = cache_path_for_temp(&tmp);
        let result = load_cached_session_at(&path);
        assert!(result.is_none());
    }

    #[test]
    fn load_returns_none_for_corrupt_json() {
        let tmp = TempDir::new().expect("temp dir");
        let path = cache_path_for_temp(&tmp);

        // Write invalid JSON directly to the cache path.
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "not valid json").unwrap();

        let result = load_cached_session_at(&path);
        assert!(result.is_none());
    }

    #[test]
    fn clear_cache_at_non_existent_dir_handles_gracefully() {
        let tmp = TempDir::new().expect("temp dir");
        // Should not panic when nothing to clear.
        clear_cache_at(tmp.path());
    }

    #[test]
    fn load_cached_session_at_read_error_returns_none() {
        let tmp = TempDir::new().expect("temp dir");
        let path = cache_path_for_temp(&tmp);

        // Create a directory where the file should be — reading a dir fails.
        fs::create_dir_all(&path).unwrap();

        let result = load_cached_session_at(&path);
        assert!(result.is_none());
    }
}
