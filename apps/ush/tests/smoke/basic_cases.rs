use super::*;

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
    let stderr = String::from_utf8_lossy(&output.stderr);
    let unexpected_stderr = stderr
        .lines()
        .filter(|line| !line.contains("setlocale: LC_ALL: cannot change locale"))
        .collect::<Vec<_>>();
    assert!(unexpected_stderr.is_empty(), "unexpected stderr: {stderr}");
    let stdout = normalize_path(&stdout, &target, "<WORKSPACE>");
    assert_snapshot(&fixture("source_state"), &stdout);
}

#[test]
fn stylish_pwd_emits_ansi() {
    let output = ush().args(["-s", "-c", "pwd"]).output().expect("run ush");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = normalize_path(
        &String::from_utf8_lossy(&output.stdout),
        &std::env::current_dir().expect("cwd"),
        "<CWD>",
    );
    assert_snapshot(&fixture("stylish_pwd"), &stdout);
}
