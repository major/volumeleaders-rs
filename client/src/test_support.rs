//! Test utilities for the VolumeLeaders client crate.
//!
//! Provides helpers for loading golden fixture files used in integration
//! and unit tests. Gated behind `#[cfg(test)]` or the `test-support`
//! feature so these helpers are never compiled into release builds.

use serde::Serialize;

use crate::datatables::DataTablesResponse;
use crate::session::{COOKIE_DOMAIN, Cookie, FORMS_AUTH_COOKIE_NAME, SESSION_COOKIE_NAME, Session};

/// Load a fixture file from `tests/fixtures/` relative to the crate root.
///
/// # Panics
///
/// Panics if the file does not exist or cannot be read. This is intentional
/// for test helpers: missing fixtures should be immediately obvious failures.
pub fn read_fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);

    std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to load fixture {}: {err}", path.display()))
}

/// Build a [`Session`] with fake but structurally valid cookie values.
pub fn test_session() -> Session {
    Session::new(
        vec![
            Cookie::new(SESSION_COOKIE_NAME, "session-123", COOKIE_DOMAIN),
            Cookie::new(FORMS_AUTH_COOKIE_NAME, "auth-456", COOKIE_DOMAIN),
        ],
        "xsrf-789",
    )
}

/// Build a [`Client`] pointed at a local mockito server.
///
/// Gated behind `#[cfg(test)]` because it depends on `mockito`, which is
/// only a dev-dependency and would fail to compile for downstream crates
/// enabling the `test-support` feature.
#[cfg(test)]
pub fn test_client(server: &mockito::Server) -> crate::client::Client {
    use crate::client::{Client, ClientConfig};

    Client::with_config(
        test_session(),
        ClientConfig {
            base_url: server.url(),
            ..ClientConfig::default()
        },
    )
    .unwrap()
}

/// Wrap data in a [`DataTablesResponse`] JSON body for mock server responses.
pub fn datatables_body<T: Serialize>(data: Vec<T>) -> String {
    serde_json::to_string(&DataTablesResponse {
        draw: 1,
        records_total: data.len() as i32,
        records_filtered: data.len() as i32,
        data,
        error: None,
    })
    .unwrap()
}
