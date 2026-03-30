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
fn tail_expressions_return_without_explicit_return() {
    let output = run_program(
        r#"
        fn greet(name: String) -> String {
          "hi " + name
        }
        print $ greet "ush"
    "#,
    );

    assert_eq!(output, "hi ush\n");
}

#[test]
fn semicolon_keeps_statement_calls_distinct_from_tail_returns() {
    let output = run_program(
        r#"
        fn trace() -> String {
          print "trace"
          "value"
        }

        fn run() -> String {
          trace();
          "after"
        }

        print $ run ()
    "#,
    );

    assert_eq!(output, "trace\nafter\n");
}

#[test]
fn match_can_be_used_as_a_tail_expression_or_a_statement() {
    let output = run_program(
        r#"
        fn choose(flag: Bool) -> String {
          match flag {
            true => "yes",
            _ => "no",
          }
        }

        fn run() -> String {
          match false {
            true => "ignored",
            _ => "ignored",
          };
          choose true
        }

        print $ run ()
    "#,
    );

    assert_eq!(output, "yes\n");
}
