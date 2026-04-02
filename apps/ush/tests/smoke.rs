use std::{fs, process::Command};

use tempfile::tempdir;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r#"'\''"#))
}

#[test]
fn helper_pipeline_counts_lines() {
    let output = ush()
        .args(["-c", "printf 'a\nb\n' | len"])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "2\n");
}

#[test]
fn helper_lambdas_support_backslash_and_block_syntax() {
    let output = ush()
        .args(["-c", r#"printf 'ush\n' | map(\line -> { upper(line) })"#])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "USH\n");
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
fn source_builtin_runs_multiline_posix_blocks() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("source.sh");
    fs::write(&script, "if true; then\n  echo sourced\nfi\n").expect("write source");

    let output = ush()
        .args([
            "-c",
            &format!(
                "source {}",
                shell_quote(script.to_str().expect("utf8 path"))
            ),
        ])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "sourced\n");
}

#[test]
fn source_builtin_returns_the_last_command_status() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("source.sh");
    fs::write(&script, "false\n").expect("write source");

    let output = ush()
        .args([
            "-c",
            &format!(
                "source {}",
                shell_quote(script.to_str().expect("utf8 path"))
            ),
        ])
        .output()
        .expect("run ush");

    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn source_builtin_applies_fallback_state_changes_to_following_commands() {
    let dir = tempdir().expect("tempdir");
    let target = dir.path().join("workspace");
    fs::create_dir_all(&target).expect("mkdir");
    let script = dir.path().join("source.sh");
    fs::write(
        &script,
        format!(
            "cd {} && true\npwd\nexport FOO=bar && true\necho $FOO\nalias ll='ls -la' && true\ntype ll\n",
            shell_quote(target.to_str().expect("utf8 path"))
        ),
    )
    .expect("write source");

    let output = ush()
        .args([
            "-c",
            &format!(
                "source {}",
                shell_quote(script.to_str().expect("utf8 path"))
            ),
        ])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(&format!("{}\n", target.display())));
    assert!(stdout.contains("bar\n"));
    assert!(stdout.contains("ll is aliased to"));
    assert!(stdout.contains("ls -la"));
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
          message
        }
        print "main"
        let task = async worker "worker"
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

#[test]
fn ush_script_supports_functional_calls() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("functional.ush");
    fs::write(
        &script,
        r#"
        fn greet(name: String) -> String {
          "hi " + name
        }
        fn wrap(message: String) -> String {
          "<" + message + ">"
        }
        fn label() -> String {
          "ush"
        }
        print $ wrap $ greet (label ())
        "#,
    )
    .expect("write script");

    let output = ush().arg(&script).output().expect("run ush");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "<hi ush>\n");
}

#[test]
fn ush_script_supports_unit_and_trait_declarations() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("traits.ush");
    fs::write(
        &script,
        r#"
        trait Named {}
        impl Eq for () {}
        impl Add for Int {}
        fn noop() -> () {
          ()
        }
        let value = noop ()
        print value == ()
        print 1 + 2
        "#,
    )
    .expect("write script");

    let output = ush().arg(&script).output().expect("run ush");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "true\n3\n");
}

#[test]
fn ush_script_exposes_generated_docs() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("docs.ush");
    fs::write(
        &script,
        r#"
        #| Demo script.
        #| @usage docs.ush --man greet
        #| Greet a user.
        #| @param name target user
        #| @return greeting text
        fn greet(name: String) -> String {
          "hi " + name
        }
        print $ greet "ush"
        "#,
    )
    .expect("write script");

    let help = ush()
        .args([script.to_str().unwrap(), "--help"])
        .output()
        .expect("run help");
    let man = ush()
        .args([script.to_str().unwrap(), "--man", "greet"])
        .output()
        .expect("run man");
    let complete = ush()
        .args([script.to_str().unwrap(), "--complete", "gr"])
        .output()
        .expect("run completion");

    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("Documented items:"));
    assert!(man.status.success());
    assert!(String::from_utf8_lossy(&man.stdout).contains("PARAMETERS"));
    assert!(complete.status.success());
    assert_eq!(String::from_utf8_lossy(&complete.stdout), "greet\n");
}
