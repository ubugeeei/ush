use std::{fs, process::Command};

use tempfile::tempdir;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

#[test]
fn helper_pipeline_counts_lines() {
    let output = ush()
        .args(["-c", "printf 'a\nb\n' | length"])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "2\n");
}

#[test]
fn pipe_to_sh_executes_script_from_stdin() {
    let output = ush()
        .args(["-c", "printf '%s\\n' 'printf \"ok\\n\"' | sh"])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "ok\n");
}

#[test]
fn posix_control_flow_uses_sh_fallback() {
    let output = ush()
        .args(["-c", "if true; then printf 'ok\\n'; fi"])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "ok\n");
}

#[test]
fn stylish_pwd_emits_ansi() {
    let output = ush().args(["-s", "-c", "pwd"]).output().expect("run ush");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("\u{1b}[1;34m"));
}

#[test]
fn ush_script_executes_via_sh() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("hello.ush");
    fs::write(
        &script,
        r#"
        let greeting = "hello"
        print greeting + " world"
        "#,
    )
    .expect("write script");

    let output = ush().arg(&script).output().expect("run ush");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "hello world\n");
}

#[test]
fn ush_script_supports_nested_enums() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("result.ush");
    fs::write(
        &script,
        r#"
        enum Result {
          Ok(String),
          Err(String),
        }
        enum Envelope {
          Wrap(Result),
          Missing,
        }
        let payload = Envelope::Wrap(Result::Ok("done"))
        match payload {
          Envelope::Wrap(Result::Ok(message)) => print message
          _ => print "fallback"
        }
        "#,
    )
    .expect("write script");

    let output = ush().arg(&script).output().expect("run ush");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "done\n");
}

#[test]
fn ush_script_supports_async_functions() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("async.ush");
    fs::write(
        &script,
        r#"
        fn worker(message: String) -> String {
          return message
        }
        print "main"
        let task = async worker("worker")
        print "after"
        let result = await task
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
