use std::process::Command;

#[test]
fn commands_outputs_flat_sorted_leaf_paths() {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .arg("commands")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<_> = stdout.lines().collect();
    let mut sorted = lines.clone();
    sorted.sort_unstable();

    assert_eq!(lines, sorted);
    assert!(lines.contains(&"commands"));
    assert!(lines.contains(&"dashboard"));
    assert!(lines.contains(&"doctor"));
    assert!(lines.contains(&"help"));
    assert!(lines.contains(&"levels"));
    assert!(lines.contains(&"schema"));
    assert!(lines.contains(&"trade list"));
    assert!(lines.contains(&"trades"));
    assert!(lines.contains(&"volume institutional"));
}

#[test]
fn commands_grouped_outputs_groups_and_descriptions() {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .args(["commands", "--grouped"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("trade\n"));
    assert!(
        stdout
            .lines()
            .any(|line| line.starts_with("  list  ") && line.contains("trades"))
    );
    assert!(stdout.contains("volume\n"));
    assert!(
        stdout
            .lines()
            .any(|line| line.starts_with("  institutional  ") && line.contains("volume"))
    );
    assert!(stdout.contains("help\n"));
    assert!(
        stdout
            .lines()
            .any(|line| line.starts_with("  help  ") && line.contains("operational help topics"))
    );
    assert!(stdout.contains("trades\n"));
    assert!(
        stdout
            .lines()
            .any(|line| line.starts_with("  trades  ") && line.contains("Alias for `trade list`"))
    );
}
