use super::*;

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

#[test]
fn stylish_history_renders_numbered_entries() {
    let home = tempdir().expect("tempdir");
    write_history(
        home.path(),
        &["echo hello", "git status", "cargo test -p ush"],
    );

    let output = ush()
        .args(["-s", "-c", "history"])
        .env("HOME", home.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot(&fixture("stylish_history"), &stdout);
}

#[test]
fn stylish_history_limit_shows_latest_entries() {
    let home = tempdir().expect("tempdir");
    write_history(
        home.path(),
        &["echo hello", "git status", "cargo test -p ush", "history 2"],
    );

    let output = ush()
        .args(["-s", "-c", "history 2"])
        .env("HOME", home.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot(&fixture("stylish_history_limit"), &stdout);
}
