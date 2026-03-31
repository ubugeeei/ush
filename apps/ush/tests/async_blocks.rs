use std::{fs, process::Command};

use tempfile::tempdir;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

#[test]
fn ush_script_supports_async_blocks() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("async_block.ush");
    fs::write(
        &script,
        r#"
        print "main"
        let task = async {
          let prefix = "work"
          prefix + "er"
        }
        print "after"
        let result = task.await
        print result
        "#,
    )
    .expect("write script");

    let output = ush().arg(&script).output().expect("run ush");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "main\nafter\nworker\n"
    );
}
