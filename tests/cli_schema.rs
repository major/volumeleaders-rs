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
    assert_eq!(trades_alias["is_alias"], true);
    assert_eq!(trades_alias["preferred_path"], "trade list");
    assert_eq!(trade_list["is_alias"], false);
    assert_eq!(trade_list["aliases"], serde_json::json!(["trades"]));
    assert!(trade_list["alias_for"].is_null());
    assert!(trade_list["args"].as_array().unwrap().iter().any(|arg| {
        arg["name"] == "strict-empty"
            && arg["long"] == "strict-empty"
            && arg["kind"] == "flag"
            && arg["parser"] == "enum"
    }));
    assert!(trade_list["args"].as_array().unwrap().iter().any(|arg| {
        arg["name"] == "verbose"
            && arg["long"] == "verbose"
            && arg["short"] == "v"
            && arg["kind"] == "flag"
            && arg["parser"] == "count"
    }));
    assert!(trade_list["args"].as_array().unwrap().iter().any(|arg| {
        arg["name"] == "tickers"
            && arg["kind"] == "positional"
            && arg["semantic_type"] == "ticker-list"
            && arg["separators"] == serde_json::json!([",", " "])
            && arg["value_name"] == "TICKERS"
            && arg["required"] == false
            && arg["multi_value"] == true
    }));
    let fields = schema["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|command| command["path"] == serde_json::json!(["fields"]))
        .unwrap();
    assert!(fields["args"].as_array().unwrap().iter().any(|arg| {
        arg["name"] == "command_path"
            && arg["kind"] == "positional"
            && arg["value_name"] == "COMMAND_PATH"
            && arg["multi_value"] == true
    }));
}

#[test]
fn schema_emits_semantic_argument_metadata() {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .arg("schema")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let schema: Value = serde_json::from_slice(&output.stdout).unwrap();
    let commands = schema["commands"].as_array().unwrap();

    assert_semantic(
        commands,
        &["trade", "list"],
        "start-date",
        "date",
        Some("YYYY-MM-DD"),
        None,
    );
    assert_semantic(
        commands,
        &["trade", "list"],
        "tickers",
        "ticker-list",
        None,
        Some(serde_json::json!([",", " "])),
    );
    assert_semantic(commands, &["trade", "list"], "limit", "integer", None, None);
    assert_semantic(
        commands,
        &["trade", "list"],
        "min-dollars",
        "money",
        None,
        None,
    );
    assert_semantic(
        commands,
        &["trade", "list"],
        "dark-pools",
        "boolean-filter",
        None,
        None,
    );
    assert_semantic(
        commands,
        &["volume", "institutional"],
        "date",
        "date",
        Some("YYYY-MM-DD"),
        None,
    );
    assert_semantic(
        commands,
        &["volume", "institutional"],
        "tickers",
        "ticker-list",
        None,
        Some(serde_json::json!([",", " "])),
    );
    assert_semantic(
        commands,
        &["watchlist", "create"],
        "tickers",
        "ticker-list",
        None,
        Some(serde_json::json!([",", " "])),
    );
    assert_semantic(
        commands,
        &["watchlist", "create"],
        "normal-prints",
        "boolean-filter",
        None,
        None,
    );
    assert_semantic(commands, &["completions"], "shell", "enum", None, None);
}

#[test]
fn schema_emits_structured_command_examples() {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .arg("schema")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let schema: Value = serde_json::from_slice(&output.stdout).unwrap();
    let commands = schema["commands"].as_array().unwrap();
    let trade_list = commands
        .iter()
        .find(|command| command["path"] == serde_json::json!(["trade", "list"]))
        .unwrap();
    let examples = trade_list["examples"].as_array().unwrap();

    assert!(examples.iter().any(|example| {
        example["description"] == "List recent trades for a ticker"
            && example["command"] == "volumeleaders-agent trade list NVDA"
    }));
    assert!(examples.iter().any(|example| {
        example["command"]
            .as_str()
            .is_some_and(|command| command.contains("--fields Ticker,DateTime,Price,Dollars"))
    }));

    let mut failures = Vec::new();
    for command in commands {
        let preferred_path = command["preferred_path"].as_str().unwrap();
        for example in command["examples"].as_array().unwrap() {
            let Some(command_text) = example["command"].as_str() else {
                failures.push(format!(
                    "{preferred_path}: example command is missing or not text"
                ));
                continue;
            };
            if !command_text.starts_with("volumeleaders-agent ") {
                failures.push(format!(
                    "{preferred_path}: example does not start with volumeleaders-agent: {command_text}"
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "schema examples must be copy-pasteable commands:\n{}",
        failures.join("\n")
    );
}

fn assert_semantic(
    commands: &[Value],
    path: &[&str],
    name: &str,
    semantic_type: &str,
    format: Option<&str>,
    separators: Option<Value>,
) {
    let command_path = path.iter().map(|part| part.to_string()).collect::<Vec<_>>();
    let command = commands
        .iter()
        .find(|command| command["path"] == serde_json::json!(command_path))
        .unwrap_or_else(|| panic!("missing command path {path:?}"));
    let arg = command["args"]
        .as_array()
        .unwrap()
        .iter()
        .find(|arg| arg["name"] == name)
        .unwrap_or_else(|| panic!("missing arg {name} for {path:?}"));

    assert_eq!(arg["semantic_type"], semantic_type);
    if let Some(format) = format {
        assert_eq!(arg["format"], format);
    }
    if let Some(separators) = separators {
        assert_eq!(arg["separators"], separators);
    }
}
