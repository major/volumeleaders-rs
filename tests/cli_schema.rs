use std::process::Command;

use serde_json::Value;

#[test]
fn schema_command_emits_machine_readable_contract() {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .arg("schema")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let schema: Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(schema["schema_version"], 1);
    assert_eq!(schema["binary"], "volumeleaders-agent");
    assert!(
        schema["commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|command| {
                command["preferred_path"] == "trade list" && command["auth_required"] == true
            })
    );
}
