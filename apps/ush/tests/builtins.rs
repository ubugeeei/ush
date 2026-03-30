use std::{
    io::Write,
    process::{Command, Stdio},
};

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

fn run_with_stdin(args: &[&str], stdin: &str) -> std::process::Output {
    let mut child = ush()
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn ush");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(stdin.as_bytes())
        .expect("write stdin");
    child.wait_with_output().expect("wait ush")
}

#[test]
fn echo_is_available_as_builtin() {
    let output = ush()
        .args(["-c", "echo hello ush"])
        .output()
        .expect("run ush");
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "hello ush\n");
}

#[test]
fn command_v_reports_existing_commands() {
    let output = ush()
        .args(["-c", "command -v sh"])
        .output()
        .expect("run ush");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("/"));
}

#[test]
fn bracket_test_builtin_returns_zero_for_true_expression() {
    let output = ush().args(["-c", "[ -d . ]"]).output().expect("run ush");
    assert!(output.status.success());
}

#[test]
fn confirm_builtin_accepts_yes_from_stdin() {
    let output = run_with_stdin(&["-c", "confirm proceed?"], "y\n");
    assert!(output.status.success());
}

#[test]
fn input_builtin_returns_typed_value() {
    let output = run_with_stdin(&["-c", "input your-name?"], "ush\n");
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "ush\n");
}

#[test]
fn select_builtin_uses_pipe_as_option_source() {
    let output = run_with_stdin(
        &["-c", "printf 'red\nblue\n' | select --prompt color"],
        "2\n",
    );
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "blue\n");
}

#[test]
fn interactive_builtins_honor_defaults_without_interaction() {
    let confirm = ush()
        .args(["--no-interaction", "-c", "confirm --default yes proceed?"])
        .output()
        .expect("run confirm");
    assert!(confirm.status.success());

    let input = ush()
        .args(["--no-interaction", "-c", "input --default guest name?"])
        .output()
        .expect("run input");
    assert!(input.status.success());
    assert_eq!(String::from_utf8_lossy(&input.stdout), "guest\n");

    let select = ush()
        .args([
            "--no-interaction",
            "-c",
            "printf 'red\nblue\n' | select --default blue",
        ])
        .output()
        .expect("run select");
    assert!(select.status.success());
    assert_eq!(String::from_utf8_lossy(&select.stdout), "blue\n");
}
