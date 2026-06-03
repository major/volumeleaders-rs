use std::process::Command;

use serde_json::Value;

#[test]
fn doctor_command_emits_machine_readable_readiness_report() {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .arg("doctor")
        .output()
        .unwrap();

    let report: Value = serde_json::from_slice(&output.stdout).unwrap();

    let ok = report
        .get("ok")
        .and_then(Value::as_bool)
        .expect("doctor report must include a boolean ok field");
    let expected_code = if ok { 0 } else { 3 };
    assert_eq!(output.status.code(), Some(expected_code));

    assert_eq!(report["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(report["auth"]["kind"], "credentials");
    assert_eq!(report["live_connectivity"]["checked"], false);
    assert_eq!(report["live_connectivity"]["status"], "skipped");
}

#[test]
fn doctor_help_documents_live_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .args(["doctor", "--help"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--live"));
    assert!(stdout.contains("low-cost authenticated live connectivity check"));
    assert!(stdout.contains("doctor --live"));
}
