use std::process::Command;

use serde_json::Value;

#[test]
fn doctor_command_emits_machine_readable_readiness_report() {
    let output = Command::new(env!("CARGO_BIN_EXE_volumeleaders-agent"))
        .arg("doctor")
        .output()
        .unwrap();

    assert!(output.stderr.is_empty());

    let report: Value = serde_json::from_slice(&output.stdout).unwrap();

    let expected_code = if report["ok"] == true { 0 } else { 3 };
    assert_eq!(output.status.code(), Some(expected_code));

    assert_eq!(report["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(report["auth"]["kind"], "browser_cookies");
    assert_eq!(report["live_connectivity"]["checked"], false);
    assert_eq!(report["live_connectivity"]["status"], "skipped");
}
