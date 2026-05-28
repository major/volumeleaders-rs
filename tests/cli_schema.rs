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
    assert!(
        schema["commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|command| {
                command["preferred_path"] == "doctor" && command["auth_required"] == false
            })
    );
    assert!(
        schema["commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|command| {
                command["preferred_path"] == "commands" && command["auth_required"] == false
            })
    );
    assert!(
        schema["commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|command| {
                command["preferred_path"] == "fields" && command["auth_required"] == false
            })
    );
    assert!(
        schema["commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|command| {
                command["preferred_path"] == "help" && command["auth_required"] == false
            })
    );
    let trade_list = schema["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|command| command["path"] == serde_json::json!(["trade", "list"]))
        .unwrap();
    let trades_alias = schema["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|command| command["path"] == serde_json::json!(["trades"]))
        .unwrap();
    assert_eq!(
        trades_alias["alias_for"],
        serde_json::json!(["trade", "list"])
    );
    assert_eq!(trades_alias["preferred_path"], "trade list");
    assert!(trade_list["args"].as_array().unwrap().iter().any(|arg| {
        arg["long"] == "strict-empty" && arg["kind"] == "flag" && arg["parser"] == "enum"
    }));
    assert!(trade_list["args"].as_array().unwrap().iter().any(|arg| {
        arg["long"] == "verbose"
            && arg["short"] == "v"
            && arg["kind"] == "flag"
            && arg["parser"] == "count"
    }));
    let fields = schema["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|command| command["path"] == serde_json::json!(["fields"]))
        .unwrap();
    assert!(fields["args"].as_array().unwrap().iter().any(|arg| {
        arg["kind"] == "positional"
            && arg["value_name"] == "COMMAND_PATH"
            && arg["multi_value"] == true
    }));
}
