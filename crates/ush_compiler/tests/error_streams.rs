use std::{fs, process::Command};

use tempfile::tempdir;
use ush_compiler::UshCompiler;

fn compile_program(source: &str) -> String {
    UshCompiler::default()
        .compile_source(source)
        .expect("compile ush program")
        .to_string()
}

fn compile_error(source: &str) -> String {
    UshCompiler::default()
        .compile_source(source)
        .expect_err("program should fail to compile")
        .to_string()
}

fn run_program(source: &str) -> std::process::Output {
    let compiled = compile_program(source);
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("program.sh");
    fs::write(&script, compiled).expect("write script");
    Command::new("/bin/sh")
        .arg(&script)
        .output()
        .expect("run compiled script")
}

#[test]
fn raise_requires_an_error_adt() {
    let error = compile_error(
        r#"
        fn fail() {
          raise "oops"
        }
    "#,
    );

    assert!(error.contains("raise expects an error ADT value"));
}

#[test]
fn function_error_streams_are_inferred_and_composed() {
    let compiled = compile_program(
        r#"
        enum Problem {
          Nope,
        }

        fn leaf() -> String {
          raise Problem::Nope
        }

        fn wrap(message: String) -> String {
          return "<" + message + ">"
        }

        fn mixed() -> String {
          $ false
          return $ wrap $ leaf ()
        }

        fn awaited() -> String {
          let task = async leaf ()
          let value = task.await
          return value
        }
    "#,
    );

    assert!(compiled.contains("# raises: Problem\nush_fn_leaf()"));
    assert!(compiled.contains("# raises: Problem | unknown\nush_fn_mixed()"));
    assert!(compiled.contains("# raises: Problem\nush_fn_awaited()"));
}

#[test]
fn raise_aborts_runtime_with_typed_message() {
    let output = run_program(
        r#"
        enum Problem {
          Nope,
        }

        fn fail() -> String {
          raise Problem::Nope
        }

        print $ fail ()
    "#,
    );

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("ush raise: Problem"));
}

#[test]
fn piped_raise_aborts_runtime_with_typed_message() {
    let output = run_program(
        r#"
        enum Problem {
          Nope,
        }

        fn fail() -> String {
          raise Problem::Nope
        }

        fn wrap(message: String) -> String {
          return message
        }

        print $ wrap $ fail ()
    "#,
    );

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("ush raise: Problem"));
}

#[test]
fn try_operator_returns_from_the_current_function() {
    let compiled = compile_program(
        r#"
        enum Problem {
          Nope,
        }

        fn fail() -> String {
          raise Problem::Nope
        }

        fn outer() -> String {
          let value = fail()?
          return value
        }
    "#,
    );

    assert!(compiled.contains("# raises: Problem\nush_fn_outer()"));
    assert!(compiled.contains("$(__ush_capture_return='1' ush_fn_fail)\" || return \"$?\""));
}

#[test]
fn try_statements_propagate_call_and_shell_failures() {
    let compiled = compile_program(
        r#"
        fn outer() {
          let command = "false"
          shell command?
          helper()?
        }

        fn helper() {
          $ false?
        }
    "#,
    );

    assert!(compiled.contains("eval \"${command}\" || return \"$?\""));
    assert!(compiled.contains("ush_fn_helper || return \"$?\""));
    assert!(compiled.contains("false || return \"$?\""));
}
