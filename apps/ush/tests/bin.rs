use std::{fs, process::Command};

mod support;

use support::assert_snapshot;
use tempfile::tempdir;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

fn fixture(name: &str) -> String {
    format!("bin/{name}.stdout")
}

#[test]
fn bin_script_maps_named_flags_defaults_and_bool_switches() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("bin.ush");
    fs::write(
        &script,
        r#"
        fn bin(#[alias("n")] name: String, #[default(2)] count: Int, verbose: Bool) {
          print name + ":" + count
          print verbose
        }
        "#,
    )
    .expect("write script");

    let output = ush()
        .args([
            script.to_str().unwrap(),
            "--name",
            "ush",
            "--count=4",
            "--verbose",
        ])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "ush:4\ntrue\n");
}

#[test]
fn bin_script_supports_short_aliases_and_defaults() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("bin.ush");
    fs::write(
        &script,
        r#"
        fn bin(#[alias("n")] name: String, #[default(3)] count: Int, verbose: Bool) {
          print name + ":" + count
          print verbose
        }
        "#,
    )
    .expect("write script");

    let output = ush()
        .args([script.to_str().unwrap(), "-n", "mini"])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "mini:3\nfalse\n");
}

#[test]
fn bin_script_completion_includes_generated_flags() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("bin.ush");
    fs::write(
        &script,
        r#"
        #| Demo CLI.
        fn bin(#[alias("n")] name: String, #[default(2)] count: Int, verbose: Bool) {
          print name + ":" + count
        }
        "#,
    )
    .expect("write script");

    let output = ush()
        .args([script.to_str().unwrap(), "--complete", "--"])
        .output()
        .expect("run completion");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot(&fixture("completion"), &stdout);
}
