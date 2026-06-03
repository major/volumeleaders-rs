use std::process::Command;

#[test]
fn help_with_valid_topic_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .args(["help", "auth"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("cached session"));
    assert!(stdout.contains("VL_USERNAME"));
    assert!(stdout.contains("auth.actions"));
    assert!(stdout.contains("doctor"));
}

#[test]
fn help_agent_topic_guides_automation() {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .args(["help", "agent"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("non-interactive automation"));
    assert!(stdout.contains("volumeleaders-agent doctor"));
    assert!(stdout.contains("volumeleaders-agent commands --grouped"));
    assert!(stdout.contains("volumeleaders-agent schema | jq"));
    assert!(stdout.contains("stdout"));
    assert!(stdout.contains("stderr"));
    assert!(stdout.contains("--strict-empty"));
    assert!(stdout.contains("mutating alert and watchlist commands"));
}

#[test]
fn help_help_lists_agent_topic() {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .args(["help", "--help"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("agent"));
    assert!(stdout.contains("Automation guidance for non-interactive agents"));
    assert!(stdout.contains("workflows"));
    assert!(stdout.contains("Workflow-oriented guidance for common agent tasks"));
}

#[test]
fn help_workflows_topic_guides_common_agent_tasks() {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .args(["help", "workflows"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Recommended first command"));
    assert!(stdout.contains("Start with defaults first"));
    assert!(stdout.contains("Institutional prints show unusual activity"));
    assert!(stdout.contains("fields <command path>"));
    assert!(stdout.contains("volumeleaders-agent doctor"));
    assert!(stdout.contains("volumeleaders-agent report top-100-rank"));
    assert!(stdout.contains("volumeleaders-agent trade list NVDA"));
    assert!(stdout.contains("volumeleaders-agent trade dashboard NVDA"));
    assert!(stdout.contains("volumeleaders-agent trade levels NVDA"));
    assert!(stdout.contains("volumeleaders-agent trade cluster-bombs NVDA"));
    assert!(stdout.contains("volumeleaders-agent trade clusters NVDA"));
    assert!(stdout.contains("volumeleaders-agent trade sentiment"));
    assert!(stdout.contains("volumeleaders-agent market earnings"));
    assert!(stdout.contains("volumeleaders-agent volume institutional"));
    assert!(stdout.contains("volumeleaders-agent alert configs"));
    assert!(stdout.contains("volumeleaders-agent watchlist configs"));
    assert!(stdout.contains("--dry-run"));
}

#[test]
fn help_exit_codes_topic_lists_semantic_codes() {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .args(["help", "exit-codes"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).unwrap();
    for code in ["0", "2", "3", "4", "5", "6", "7"] {
        assert!(
            stdout.contains(code),
            "exit-codes topic must mention {code}"
        );
    }
}

#[test]
fn all_help_topics_succeed() {
    for topic in [
        "agent",
        "auth",
        "environment",
        "exit-codes",
        "schema",
        "examples",
        "workflows",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
            .args(["help", topic])
            .output()
            .unwrap();

        assert!(output.status.success(), "help {topic} should succeed");
        assert!(
            output.stderr.is_empty(),
            "help {topic} should not use stderr"
        );
        assert!(
            !output.stdout.is_empty(),
            "help {topic} should write stdout"
        );
    }
}

#[test]
fn help_with_invalid_topic_fails_with_usage_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .args(["help", "missing-topic"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn help_without_topic_shows_usage_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .arg("help")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}
