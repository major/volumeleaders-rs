use std::process::Command;

use serde_json::Value;

#[test]
fn aliases_route_to_canonical_help_pages() {
    for (alias, usage) in [
        ("trades", "Usage: volumeleaders-agent trade list"),
        ("dashboard", "Usage: volumeleaders-agent trade dashboard"),
        ("levels", "Usage: volumeleaders-agent trade levels"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
            .args([alias, "--help"])
            .output()
            .unwrap();

        assert!(output.status.success(), "{alias} --help should succeed");
        assert!(output.stderr.is_empty());

        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains(usage));
        assert!(stdout.contains("Examples:"));
    }
}

#[test]
fn schema_reports_aliases_with_canonical_preferred_paths() {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .arg("schema")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let schema: Value = serde_json::from_slice(&output.stdout).unwrap();
    let commands = schema["commands"].as_array().unwrap();

    for (alias, canonical) in [
        ("trades", ["trade", "list"]),
        ("dashboard", ["trade", "dashboard"]),
        ("levels", ["trade", "levels"]),
    ] {
        let command = commands
            .iter()
            .find(|command| command["path"] == serde_json::json!([alias]))
            .unwrap();

        assert_eq!(command["alias_for"], serde_json::json!(canonical));
        assert_eq!(command["preferred_path"], canonical.join(" "));
        assert_eq!(command["auth_required"], true);
        assert_eq!(command["is_alias"], true);
        assert_eq!(command["aliases"], serde_json::json!([]));

        let canonical_command = commands
            .iter()
            .find(|command| command["path"] == serde_json::json!(canonical))
            .unwrap();

        assert_eq!(canonical_command["is_alias"], false);
        assert_eq!(canonical_command["aliases"], serde_json::json!([alias]));
        assert!(canonical_command["alias_for"].is_null());
    }
}
