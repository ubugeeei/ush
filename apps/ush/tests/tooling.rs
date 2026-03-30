use std::{fs, process::Command};

use tempfile::tempdir;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

#[test]
fn format_check_detects_unformatted_script() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("format.ush");
    fs::write(&script, "fn greet(name: String)->String {\nprint name\n}\n").expect("write");

    let output = ush()
        .args(["format", script.to_str().unwrap(), "--check"])
        .output()
        .expect("run format check");

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("formatting required"));
}

#[test]
fn format_stdout_prints_formatted_source() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("format.ush");
    fs::write(&script, "let value=1\n").expect("write");

    let output = ush()
        .args(["format", script.to_str().unwrap(), "--stdout"])
        .output()
        .expect("run format stdout");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "let value = 1\n");
}

#[test]
fn check_reports_type_errors() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("check.ush");
    fs::write(&script, "let value = missing.await\n").expect("write");

    let output = ush()
        .args(["check", script.to_str().unwrap()])
        .output()
        .expect("run check");

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing"));
}
