use std::{fs, process::Command};

use tempfile::tempdir;
use ush_compiler::UshCompiler;

fn run_program(source: &str) -> String {
    let compiled = UshCompiler::default()
        .compile_source(source)
        .expect("compile ush program");
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("program.sh");
    fs::write(&script, compiled).expect("write script");

    let output = Command::new("/bin/sh")
        .arg(&script)
        .output()
        .expect("run compiled script");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn await_returns_async_block_result() {
    let output = run_program(
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
    );

    assert_eq!(output, "main\nafter\nworker\n");
}

#[test]
fn async_blocks_support_explicit_return() {
    let output = run_program(
        r#"
        let task = async {
          if true {
            return 40
          }
          0
        }
        let result = task.await
        print result + 2
    "#,
    );

    assert_eq!(output, "42\n");
}
