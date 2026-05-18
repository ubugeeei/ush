//! CLI smoke tests: the kind of thing a production deployment relies
//! on (the binary actually runs, `--version` prints something, `-c`
//! gets a script through the compiler and to /bin/sh, an unknown flag
//! is rejected). Each test runs the real release-built binary via
//! `CARGO_BIN_EXE_ush` so any future refactor of the CLI surface is
//! caught here before it ships.

use std::process::Command;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

#[test]
fn prints_version_to_stdout() {
    let out = ush().arg("--version").output().expect("spawn ush");
    assert!(out.status.success(), "ush --version exited non-zero");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("ush "),
        "expected `ush <version>` on stdout, got: {stdout}",
    );
}

#[test]
fn help_includes_known_subcommands() {
    let out = ush().arg("--help").output().expect("spawn ush");
    assert!(out.status.success(), "ush --help exited non-zero");
    let stdout = String::from_utf8_lossy(&out.stdout);
    for keyword in ["format", "check"] {
        assert!(
            stdout.contains(keyword),
            "ush --help should mention `{keyword}`, full output: {stdout}",
        );
    }
}

#[test]
fn unknown_flag_is_rejected() {
    let out = ush()
        .arg("--definitely-not-a-real-flag")
        .output()
        .expect("spawn ush");
    assert!(
        !out.status.success(),
        "unknown flag must cause a non-zero exit",
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.is_empty(),
        "unknown flag must produce a stderr message",
    );
}

#[test]
fn dash_c_runs_a_one_liner_through_sh() {
    let out = ush()
        .args(["-c", "printf '%s' production-ready"])
        .output()
        .expect("spawn ush");
    assert!(out.status.success(), "ush -c exited non-zero: {out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.trim(), "production-ready");
}
