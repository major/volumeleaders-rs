use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;

use serde_json::Value;

#[test]
fn documented_fields_examples_use_discoverable_field_names() {
    let schema = run_json(&["schema"]);
    let commands = schema["commands"].as_array().unwrap();
    let mut field_names_by_path = BTreeMap::new();
    let mut failures = Vec::new();

    for command in commands {
        let preferred_path = command["preferred_path"].as_str().unwrap();
        let Some(long_about) = command["long_about"].as_str() else {
            continue;
        };

        for example in fields_examples(long_about) {
            validate_example_fields(
                preferred_path,
                example,
                &mut field_names_by_path,
                &mut failures,
            );
        }
    }

    let help_examples = run_text(&["help", "examples"]);
    for example in fields_examples(&help_examples) {
        let Some(preferred_path) = command_path_for_example(example, commands) else {
            failures.push(format!(
                "could not resolve command path for help example: {example}"
            ));
            continue;
        };
        validate_example_fields(
            preferred_path,
            example,
            &mut field_names_by_path,
            &mut failures,
        );
    }

    assert!(
        failures.is_empty(),
        "documented --fields examples reference invalid fields:\n{}",
        failures.join("\n")
    );
}

fn validate_example_fields(
    preferred_path: &str,
    example: &str,
    field_names_by_path: &mut BTreeMap<String, BTreeSet<String>>,
    failures: &mut Vec<String>,
) {
    let Some(fields) = fields_argument(example) else {
        return;
    };
    let valid_fields = field_names_by_path
        .entry(preferred_path.to_string())
        .or_insert_with(|| discover_field_names(preferred_path));

    for field in fields.split(',').filter(|field| !field.is_empty()) {
        if !valid_fields.contains(field) {
            failures.push(format!(
                "{preferred_path}: `{field}` is not discoverable in `{example}`"
            ));
        }
    }
}

fn fields_examples(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .map(str::trim)
        .filter(|line| line.starts_with("volumeleaders-agent ") && line.contains("--fields"))
}

fn fields_argument(example: &str) -> Option<&str> {
    let mut tokens = example.split_whitespace();
    while let Some(token) = tokens.next() {
        if token == "--fields" {
            return tokens.next();
        }
        if let Some(fields) = token.strip_prefix("--fields=") {
            return Some(fields);
        }
    }

    None
}

fn command_path_for_example<'a>(example: &str, commands: &'a [Value]) -> Option<&'a str> {
    commands
        .iter()
        .filter_map(|command| command["preferred_path"].as_str())
        .filter(|preferred_path| example_contains_path(example, preferred_path))
        .max_by_key(|preferred_path| preferred_path.len())
}

fn example_contains_path(example: &str, preferred_path: &str) -> bool {
    let example_tokens = example.split_whitespace().collect::<Vec<_>>();
    let path_tokens = preferred_path.split_whitespace().collect::<Vec<_>>();

    example_tokens
        .windows(path_tokens.len())
        .any(|window| window == path_tokens.as_slice())
}

fn discover_field_names(preferred_path: &str) -> BTreeSet<String> {
    let mut args = vec!["fields".to_string()];
    args.extend(preferred_path.split_whitespace().map(ToString::to_string));
    let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    let discovery = run_json(&arg_refs);

    discovery["fields"]
        .as_array()
        .unwrap_or_else(|| panic!("missing fields array for {preferred_path}"))
        .iter()
        .map(|field| field["name"].as_str().unwrap().to_string())
        .collect()
}

fn run_json(args: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .args(args)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "command failed: volumeleaders-agent {}\nstderr: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    serde_json::from_slice(&output.stdout).unwrap()
}

fn run_text(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .args(args)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "command failed: volumeleaders-agent {}\nstderr: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    String::from_utf8(output.stdout).unwrap()
}
