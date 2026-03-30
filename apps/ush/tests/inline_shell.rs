use std::{fs, process::Command};

use tempfile::tempdir;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

#[test]
fn ush_script_supports_dollar_prefixed_inline_shell() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("inline.ush");
    fs::write(
        &script,
        "let greeting = \"hello\"\n$ printf '%s\\n' \"$greeting\"\n",
    )
    .expect("write script");

    let output = ush().arg(&script).output().expect("run ush");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "hello\n");
}

#[test]
fn ush_script_allows_inline_shell_in_match_arms() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("match-inline.ush");
    fs::write(
        &script,
        "let greeting = \"hello\"\nmatch greeting {\n  \"hello\" => $ printf '%s\\n' matched\n  _ => print \"fallback\"\n}\n",
    )
    .expect("write script");

    let output = ush().arg(&script).output().expect("run ush");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "matched\n");
}
