use std::{fs, process::Command};

use tempfile::tempdir;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("profile\n"));
    assert!(stdout.contains("ll is aliased to"));
    assert!(stdout.contains("profile-loaded"));
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
    assert_eq!(String::from_utf8_lossy(&output.stdout), "loaded\n");
}
