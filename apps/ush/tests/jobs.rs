use std::{fs, process::Command};

mod support;

use support::assert_snapshot;
use tempfile::tempdir;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r#"'\''"#))
}

fn normalize_job_output(text: &str) -> String {
    let normalized = text
        .lines()
        .map(|line| {
            let mut parts = line.split_whitespace().collect::<Vec<_>>();
            if parts.len() >= 2 && parts[0].starts_with('[') && parts[1].chars().all(|ch| ch.is_ascii_digit()) {
                parts[1] = "<PID>";
                parts.join(" ")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("{normalized}\n")
}

#[test]
fn background_jobs_can_be_listed_and_waited() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("jobs.sh");
    fs::write(
        &script,
        "sleep 1 </dev/null >/dev/null 2>&1 &\njobs\nwait\n",
    )
    .expect("write script");

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
    assert!(output.stderr.is_empty());
    let stdout = normalize_job_output(&String::from_utf8_lossy(&output.stdout));
    assert_snapshot("jobs/background_jobs.stdout", &stdout);
}

#[test]
fn fg_waits_for_the_most_recent_background_job() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("fg.sh");
    fs::write(&script, "sleep 1 </dev/null >/dev/null 2>&1 &\nfg %1\n").expect("write script");

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
    assert!(output.stderr.is_empty());
    let stdout = normalize_job_output(&String::from_utf8_lossy(&output.stdout));
    assert_snapshot("jobs/fg.stdout", &stdout);
}

#[test]
fn disown_removes_jobs_from_follow_up_listings() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("disown.sh");
    fs::write(
        &script,
        "sleep 1 </dev/null >/dev/null 2>&1 &\ndisown %1\njobs\n",
    )
    .expect("write script");

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
    assert!(output.stderr.is_empty());
    let stdout = normalize_job_output(&String::from_utf8_lossy(&output.stdout));
    assert_snapshot("jobs/disown.stdout", &stdout);
}
