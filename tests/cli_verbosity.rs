use std::process::Command;

use serde_json::Value;

#[test]
fn verbose_schema_keeps_stdout_machine_readable() {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .args(["-vv", "schema"])
        .output()
        .unwrap();

    assert!(output.status.success());

    let schema: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(schema["binary"], "volumeleaders-agent");
}

#[test]
fn verbose_flag_is_available_after_subcommands() {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .args(["schema", "-vvv"])
        .output()
        .unwrap();

    assert!(output.status.success());

    let schema: Value = serde_json::from_slice(&output.stdout).unwrap();
    let trade_list = schema["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|command| command["preferred_path"] == "trade list")
        .unwrap();

    assert!(trade_list["args"].as_array().unwrap().iter().any(|arg| {
        arg["long"] == "verbose" && arg["short"] == "v" && arg["parser"] == "count"
    }));
}
