use super::*;

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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with('/'));
    assert!(stdout.trim_end().ends_with("/sh"));
}

#[test]
fn stylish_command_v_renders_lookup_categories() {
    let dir = tempdir().expect("tempdir");
    let config_path = dir.path().join("config.json");
    fs::write(
        &config_path,
        r#"{
  "aliases": {
    "ll": "ls -lah"
  }
}
"#,
    )
    .expect("write config");

    let output = ush()
        .args([
            "--config",
            config_path.to_str().expect("utf8 path"),
            "-s",
            "-c",
            "command -v ll echo sh",
        ])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = normalize_command_paths(&String::from_utf8_lossy(&output.stdout), &["sh"]);
    assert_snapshot(&fixture("stylish_command_v"), &stdout);
}

#[test]
fn stylish_type_marks_missing_targets() {
    let output = ush()
        .args(["-s", "-c", "type echo definitely-not-a-real-command"])
        .output()
        .expect("run ush");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot(&fixture("stylish_type_missing"), &stdout);
}

#[test]
fn stylish_env_renders_sorted_variables_without_tables() {
    let output = ush()
        .env_clear()
        .args(["-s", "-c", "env HELLO=ush FOO=bar EMPTY="])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let empty_index = stdout.find("EMPTY").expect("EMPTY");
    let foo_index = stdout.find("FOO").expect("FOO");
    let hello_index = stdout.find("HELLO").expect("HELLO");
    assert!(empty_index < foo_index);
    assert!(foo_index < hello_index);
    assert!(output.stderr.is_empty());
    assert!(stdout.contains("UBSHELL_INTERACTION"));
    assert!(stdout.contains("UBSHELL_STYLISH"));
    assert!(stdout.contains("USH_INTERACTION"));
    assert!(stdout.contains("USH_STYLISH"));
}

#[test]
fn stylish_env_handles_cleared_process_environment() {
    let output = ush()
        .env_clear()
        .args(["-s", "-c", "env"])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.stderr.is_empty());
    assert!(stdout.contains("UBSHELL_INTERACTION"));
    assert!(stdout.contains("UBSHELL_STYLISH"));
    assert!(stdout.contains("USH_INTERACTION"));
    assert!(stdout.contains("USH_STYLISH"));
    assert!(!stdout.contains("PATH"));
    assert!(!stdout.contains("HOME"));
}

#[test]
fn stylish_alias_renders_named_expansions() {
    let dir = tempdir().expect("tempdir");
    let config_path = dir.path().join("config.json");
    fs::write(
        &config_path,
        r#"{
  "aliases": {
    "ll": "ls -lah",
    "gs": "git status"
  }
}
"#,
    )
    .expect("write config");

    let output = ush()
        .args([
            "--config",
            config_path.to_str().expect("utf8 path"),
            "-s",
            "-c",
            "alias",
        ])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot(&fixture("stylish_alias"), &stdout);
}
