use std::{fs, process::Command};

mod support;

use support::assert_snapshot;
use tempfile::tempdir;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

fn normalize_path(text: &str, path: &std::path::Path, marker: &str) -> String {
    text.replace(&path.display().to_string(), marker)
}

fn assert_only_locale_warnings(stderr: &[u8]) {
    let stderr = String::from_utf8_lossy(stderr);
    let unexpected = stderr
        .lines()
        .filter(|line| !line.contains("setlocale: LC_ALL: cannot change locale"))
        .collect::<Vec<_>>();
    assert!(unexpected.is_empty(), "unexpected stderr: {stderr}");
}

#[test]
fn login_flag_loads_explicit_profile_before_running_a_command() {
    let dir = tempdir().expect("tempdir");
    let profile = dir.path().join("profile.sh");
    fs::write(
        &profile,
        "export PROFILE_VALUE=profile\nalias ll='echo profile-loaded'\n",
    )
    .expect("write profile");

    let output = ush()
        .args([
            "--login",
            "--profile-file",
            profile.to_str().expect("utf8 path"),
            "-c",
            "echo $PROFILE_VALUE; type ll",
        ])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert_only_locale_warnings(&output.stderr);
    let stdout = normalize_path(
        &String::from_utf8_lossy(&output.stdout),
        &profile,
        "<PROFILE_SH>",
    );
    assert_snapshot("startup/login_profile.stdout", &stdout);
}

#[test]
fn rc_file_loads_before_running_a_command() {
    let dir = tempdir().expect("tempdir");
    let rc = dir.path().join("rc.sh");
    fs::write(&rc, "export RC_VALUE=loaded\n").expect("write rc");

    let output = ush()
        .args([
            "--rc-file",
            rc.to_str().expect("utf8 path"),
            "-c",
            "echo $RC_VALUE",
        ])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert_only_locale_warnings(&output.stderr);
    let stdout = normalize_path(&String::from_utf8_lossy(&output.stdout), &rc, "<RC_SH>");
    assert_snapshot("startup/rc.stdout", &stdout);
}
