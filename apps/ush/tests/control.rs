use std::{fs, process::Command};

use tempfile::tempdir;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

#[test]
fn ush_scripts_support_control_flow_and_iterables() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("control.ush");
    fs::write(
        &script,
        r#"
        let items = [1, 2]
        for item in items {
          print item
        }
        let pair = (3, 4)
        for item in pair {
          print item
        }
        for item in 4..6 {
          print item
        }
        "#,
    )
    .expect("write script");

    let output = ush().arg(&script).output().expect("run ush");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "1\n2\n3\n4\n4\n5\n"
    );
}

#[test]
fn ush_scripts_support_if_let_and_loop_breaks() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("if-let.ush");
    fs::write(
        &script,
        r#"
        enum Option {
          None,
          Some(Int),
        }
        let maybe = Option::Some(7)
        if let Option::Some(it) = maybe && it == 7 {
          print it
        }
        loop {
          print 9
          break
        }
        "#,
    )
    .expect("write script");

    let output = ush().arg(&script).output().expect("run ush");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "7\n9\n");
}
