use std::process::Command;

use serde_json::Value;

#[test]
fn fields_trade_list_emits_machine_readable_metadata() {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .args(["fields", "trade", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let discovery: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(discovery["command_path"], "trade list");
    assert_eq!(discovery["preferred_path"], "trade list");

    let fields = discovery["fields"].as_array().unwrap();
    assert_field(fields, "FullTimeString24", "string");
    assert_field(fields, "DollarsMultiplier", "number");
    assert_field(fields, "Dollars", "number");
    assert!(fields.iter().all(|field| field["description"].is_string()));
}

#[test]
fn fields_volume_institutional_emits_non_trade_metadata() {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .args(["fields", "volume", "institutional"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let discovery: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(discovery["command_path"], "volume institutional");

    let fields = discovery["fields"].as_array().unwrap();
    assert_field(fields, "Ticker", "string");
    assert_field(fields, "LatePrint", "boolean");
}

#[test]
fn fields_trade_dashboard_emits_nested_section_metadata() {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .args(["fields", "trade", "dashboard"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let discovery: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(discovery["command_path"], "trade dashboard");
    assert_eq!(discovery["preferred_path"], "trade dashboard");

    let fields = discovery["fields"].as_array().unwrap();
    assert_field(fields, "trades.TradeRank", "number");
    assert_field(fields, "clusters.TradeClusterRank", "number");
    assert_field(fields, "clusters.MinFullTimeString24", "string");
    assert_field(fields, "levels.TradeLevelRank", "number");
    assert_field(fields, "cluster_bombs.TradeCount", "number");
    assert!(fields.iter().all(|field| {
        field["name"]
            .as_str()
            .is_some_and(|name| name.contains('.'))
    }));
}

#[test]
fn fields_unknown_command_returns_structured_usage_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .args(["fields", "trade", "unknown"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());

    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["ok"], false);
    assert_eq!(error["error"]["kind"], "usage_error");
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("unknown command path")
    );
}

fn assert_field(fields: &[Value], name: &str, type_hint: &str) {
    assert!(fields.iter().any(|field| {
        field["name"] == name && field["type"] == type_hint && field["description"].is_string()
    }));
}
