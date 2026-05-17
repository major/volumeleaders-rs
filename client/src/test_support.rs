//! Test utilities for the VolumeLeaders client crate.
//!
//! Provides helpers for loading golden fixture files used in integration
//! and unit tests. Gated behind `#[cfg(test)]` or the `test-support`
//! feature so these helpers are never compiled into release builds.

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
