use std::process::Command;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
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
