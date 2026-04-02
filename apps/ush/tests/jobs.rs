use std::{fs, process::Command};

use tempfile::tempdir;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r#"'\''"#))
}

#[test]
fn background_jobs_can_be_listed_and_waited() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("jobs.sh");
    fs::write(&script, "sleep 1 &\njobs\nwait\n").expect("write script");

    let output = ush()
        .args([
            "-c",
            &format!(
                "source {}",
                shell_quote(script.to_str().expect("utf8 path"))
            ),
        ])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[1]"));
    assert!(stdout.contains("Running"));
    assert!(stdout.contains("sleep 1"));
}

#[test]
fn fg_waits_for_the_most_recent_background_job() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("fg.sh");
    fs::write(&script, "sleep 1 &\nfg %1\n").expect("write script");

    let output = ush()
        .args([
            "-c",
            &format!(
                "source {}",
                shell_quote(script.to_str().expect("utf8 path"))
            ),
        ])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("[1]"));
}

#[test]
fn disown_removes_jobs_from_follow_up_listings() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("disown.sh");
    fs::write(&script, "sleep 1 &\ndisown %1\njobs\n").expect("write script");

    let output = ush()
        .args([
            "-c",
            &format!(
                "source {}",
                shell_quote(script.to_str().expect("utf8 path"))
            ),
        ])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[1]"));
    assert!(!stdout.contains("Running"));
}
