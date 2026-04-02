use std::{fs, process::Command};

use tempfile::tempdir;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

#[test]
fn test_command_runs_explicit_directory_and_reports_failures() {
    let dir = tempdir().expect("tempdir");
    let tests = dir.path().join("suite");
    fs::create_dir_all(&tests).expect("create test dir");
    fs::write(tests.join("pass.ush"), "print \"ok\"\n").expect("write pass test");
    fs::write(tests.join("fail.ush"), "shell \"false\"\n").expect("write fail test");

    let output = ush()
        .args(["test", tests.to_str().unwrap()])
        .current_dir(dir.path())
        .output()
        .expect("run ush test");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(1));
    assert!(stdout.contains("ok   suite/pass.ush"));
    assert!(stdout.contains("fail suite/fail.ush"));
    assert!(stdout.contains("stderr: ush runtime map:"));
    assert!(stdout.contains("stderr:   section: user-code"));
    assert!(stdout.contains("stderr:   source : shell \"false\""));
    assert!(stdout.contains("stderr:   mapped : G"));
    assert!(stdout.contains("1 passed; 1 failed"));
}

#[test]
fn test_command_defaults_to_tests_directory() {
    let dir = tempdir().expect("tempdir");
    let tests = dir.path().join("tests");
    fs::create_dir_all(&tests).expect("create tests dir");
    fs::write(tests.join("sample.ush"), "print \"ok\"\n").expect("write test");

    let output = ush()
        .arg("test")
        .current_dir(dir.path())
        .output()
        .expect("run ush test");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("ok   tests/sample.ush"));
    assert!(stdout.contains("1 passed; 0 failed"));
}
