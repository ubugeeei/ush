use std::{fs, process::Command};

use tempfile::tempdir;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

#[test]
fn ush_script_raise_exits_with_typed_error() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("raise.ush");
    fs::write(
        &script,
        concat!(
            "enum Problem {\n",
            "  Nope,\n",
            "}\n",
            "fn fail() -> Problem!String {\n",
            "  raise Problem::Nope\n",
            "}\n",
            "fn wrap(message: String) -> String {\n",
            "  return message\n",
            "}\n",
            "print $ wrap $ fail ()\n",
        ),
    )
    .expect("write script");

    let output = ush().arg(&script).output().expect("run ush");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("ush raise: Problem"));
}

#[test]
fn ush_script_try_operator_propagates_function_failure() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("try.ush");
    fs::write(
        &script,
        concat!(
            "enum Problem {\n",
            "  Nope,\n",
            "}\n",
            "fn fail() -> Problem!() {\n",
            "  raise Problem::Nope\n",
            "}\n",
            "fn outer() -> Problem!() {\n",
            "  fail()?\n",
            "  print \"unreachable\"\n",
            "}\n",
            "outer ()\n",
        ),
    )
    .expect("write script");

    let output = ush().arg(&script).output().expect("run ush");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("ush raise: Problem"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("unreachable"));
}
