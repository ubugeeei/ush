use std::{
    fs,
    io::Write,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use tempfile::tempdir;

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

fn run_git(dir: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {:?} failed\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn init_git_repo(dir: &Path) {
    run_git(dir, &["init", "-q"]);
    run_git(dir, &["config", "user.name", "ush"]);
    run_git(dir, &["config", "user.email", "ush@example.com"]);
    fs::write(dir.join("tracked.txt"), "hello\n").expect("write tracked");
    run_git(dir, &["add", "tracked.txt"]);
    run_git(dir, &["commit", "-q", "-m", "initial commit"]);
}

fn history_file(home: &Path) -> PathBuf {
    home.join("Library/Caches/dev.ubugeeei.ush/history.txt")
}

fn write_history(home: &Path, entries: &[&str]) {
    let path = history_file(home);
    fs::create_dir_all(path.parent().expect("history dir")).expect("create history dir");
    let body = if entries.is_empty() {
        String::new()
    } else {
        format!("{}\n", entries.join("\n"))
    };
    fs::write(path, body).expect("write history");
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("command -v"));
    assert!(stdout.contains("3 targets"));
    assert!(stdout.contains("[alias]"));
    assert!(stdout.contains("[builtin]"));
    assert!(stdout.contains("[external]"));
    assert!(!stdout.contains("┌"));
    assert!(!stdout.contains("│"));
}

#[test]
fn stylish_type_marks_missing_targets() {
    let output = ush()
        .args(["-s", "-c", "type echo definitely-not-a-real-command"])
        .output()
        .expect("run ush");

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("type"));
    assert!(stdout.contains("2 targets"));
    assert!(stdout.contains("echo"));
    assert!(stdout.contains("[builtin]"));
    assert!(stdout.contains("[not found]"));
    assert!(stdout.contains("definitely-not-a-real-command"));
    assert!(!stdout.contains("┌"));
    assert!(!stdout.contains("│"));
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

#[test]
fn stylish_ls_a_includes_dot_entries_and_hidden_files() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join(".hidden"), "secret\n").expect("write");
    fs::write(dir.path().join("visible"), "open\n").expect("write");

    let output = ush()
        .args(["-s", "-c", "ls -a"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ls"));
    assert!(stdout.contains("4 entries"));
    assert!(stdout.contains("./"));
    assert!(stdout.contains("../"));
    assert!(stdout.contains(".hidden"));
    assert!(stdout.contains("visible"));
    assert!(!stdout.contains("┌"));
    assert!(!stdout.contains("│"));
}

#[test]
fn stylish_ls_combined_short_flags_keep_rich_output() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join(".hidden"), "secret\n").expect("write");
    fs::write(dir.path().join("visible"), "open\n").expect("write");

    let output = ush()
        .args(["-s", "-c", "ls -lah"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ls"));
    assert!(stdout.contains("4 entries"));
    assert!(stdout.contains("./"));
    assert!(stdout.contains("../"));
    assert!(stdout.contains(".hidden"));
    assert!(stdout.contains("visible"));
    assert!(!stdout.contains("┌"));
    assert!(!stdout.contains("│"));
}

#[test]
fn stylish_ls_almost_all_shows_hidden_without_dot_entries() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join(".hidden"), "secret\n").expect("write");
    fs::write(dir.path().join("visible"), "open\n").expect("write");

    let output = ush()
        .args(["-s", "-c", "ls -A"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ls"));
    assert!(stdout.contains("2 entries"));
    assert!(stdout.contains(".hidden"));
    assert!(stdout.contains("visible"));
    assert!(!stdout.contains("./"));
    assert!(!stdout.contains("../"));
    assert!(!stdout.contains("┌"));
    assert!(!stdout.contains("│"));
}

#[test]
fn stylish_ls_keeps_explicit_hidden_targets_visible_without_all_flag() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join(".env"), "TOKEN=ush\n").expect("write");

    let output = ush()
        .args(["-s", "-c", "ls .env"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ls"));
    assert!(stdout.contains(".env"));
    assert!(stdout.contains("[file]"));
    assert!(!stdout.contains("│"));
}

#[test]
fn stylish_ls_handles_broken_symlinks() {
    let dir = tempdir().expect("tempdir");
    symlink("missing-target", dir.path().join("broken-link")).expect("symlink");

    let output = ush()
        .args(["-s", "-c", "ls"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("broken-link"));
    assert!(stdout.contains("[link]"));
    assert!(stdout.contains("-> missing-target"));
}

#[test]
fn stylish_diff_renders_hunks_and_preserves_exit_code() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("before.txt"), "alpha\nbeta\n").expect("write before");
    fs::write(dir.path().join("after.txt"), "alpha\ngamma\n").expect("write after");

    let output = ush()
        .args(["-s", "-c", "diff before.txt after.txt"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("diff"));
    assert!(stdout.contains("before.txt"));
    assert!(stdout.contains("after.txt"));
    assert!(stdout.contains("@@ -1,2 +1,2 @@"));
    assert!(stdout.contains("-beta"));
    assert!(stdout.contains("+gamma"));
    assert!(!stdout.contains("2c2"));
    assert!(!stdout.contains("┌"));
    assert!(!stdout.contains("│"));
}

#[test]
fn stylish_diff_unified_flag_keeps_rich_output() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("before.txt"), "alpha\nbeta\n").expect("write before");
    fs::write(dir.path().join("after.txt"), "alpha\ngamma\n").expect("write after");

    let output = ush()
        .args(["-s", "-c", "diff -u before.txt after.txt"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("diff"));
    assert!(stdout.contains("[1 hunk]"));
    assert!(stdout.contains("[+1]"));
    assert!(stdout.contains("[-1]"));
    assert!(stdout.contains("@@ -1,2 +1,2 @@"));
    assert!(!stdout.contains("┌"));
    assert!(!stdout.contains("│"));
}

#[test]
fn stylish_grep_groups_file_matches() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("sample.txt"),
        "alpha\nfoo first\nbeta\nfoo second\n",
    )
    .expect("write sample");

    let output = ush()
        .args(["-s", "-c", "grep foo sample.txt"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("grep"));
    assert!(stdout.contains("foo"));
    assert!(stdout.contains("sample.txt"));
    assert!(stdout.contains("[2 matches]"));
    assert!(stdout.contains("[line 2]"));
    assert!(stdout.contains("foo first"));
    assert!(stdout.contains("[line 4]"));
    assert!(stdout.contains("foo second"));
    assert!(!stdout.contains("┌"));
    assert!(!stdout.contains("│"));
}

#[test]
fn stylish_grep_reads_pipeline_input() {
    let output = ush()
        .args(["-s", "-c", "printf 'alpha\nfoo\nbeta\n' | grep foo"])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("grep"));
    assert!(stdout.contains("stdin"));
    assert!(stdout.contains("[1 match]"));
    assert!(stdout.contains("[line 2]"));
    assert!(stdout.contains("foo"));
    assert!(!stdout.contains("┌"));
    assert!(!stdout.contains("│"));
}

#[test]
fn stylish_grep_no_matches_preserves_exit_code() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("sample.txt"), "alpha\nbeta\n").expect("write sample");

    let output = ush()
        .args(["-s", "-c", "grep foo sample.txt"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("grep"));
    assert!(stdout.contains("foo"));
    assert!(stdout.contains("[no matches]"));
    assert!(!stdout.contains("┌"));
    assert!(!stdout.contains("│"));
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("history"));
    assert!(stdout.contains("3 entries"));
    assert!(stdout.contains("[1]"));
    assert!(stdout.contains("echo hello"));
    assert!(stdout.contains("[3]"));
    assert!(stdout.contains("cargo test -p ush"));
    assert!(!stdout.contains("┌"));
    assert!(!stdout.contains("│"));
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("history"));
    assert!(stdout.contains("showing latest 2"));
    assert!(stdout.contains("[3]"));
    assert!(stdout.contains("cargo test -p ush"));
    assert!(stdout.contains("[4]"));
    assert!(stdout.contains("history 2"));
    assert!(!stdout.contains("[1]"));
    assert!(!stdout.contains("echo hello"));
    assert!(!stdout.contains("┌"));
    assert!(!stdout.contains("│"));
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("alias"));
    assert!(stdout.contains("2 aliases"));
    assert!(stdout.contains("shell shortcuts expanded before command lookup"));
    assert!(stdout.contains("ll"));
    assert!(stdout.contains("ls -lah"));
    assert!(stdout.contains("gs"));
    assert!(stdout.contains("git status"));
    assert!(!stdout.contains("┌"));
    assert!(!stdout.contains("│"));
}

#[test]
fn stylish_which_renders_alias_builtin_and_external_targets() {
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
            "which ll echo sh",
        ])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("which"));
    assert!(stdout.contains("3 targets"));
    assert!(stdout.contains("[alias]"));
    assert!(stdout.contains("ll"));
    assert!(stdout.contains("ls -lah"));
    assert!(stdout.contains("[builtin]"));
    assert!(stdout.contains("echo"));
    assert!(stdout.contains("shell builtin"));
    assert!(stdout.contains("[external]"));
    assert!(stdout.contains("sh"));
    assert!(!stdout.contains("┌"));
    assert!(!stdout.contains("│"));
}

#[test]
fn stylish_which_marks_missing_targets_and_preserves_exit_code() {
    let output = ush()
        .args(["-s", "-c", "which definitely-not-a-real-command"])
        .output()
        .expect("run ush");

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("which"));
    assert!(stdout.contains("1 target"));
    assert!(stdout.contains("[not found]"));
    assert!(stdout.contains("definitely-not-a-real-command"));
    assert!(stdout.contains("unavailable on PATH"));
    assert!(!stdout.contains("┌"));
    assert!(!stdout.contains("│"));
}

#[test]
fn stylish_git_status_renders_rich_output() {
    let dir = tempdir().expect("tempdir");
    init_git_repo(dir.path());
    run_git(dir.path(), &["checkout", "-q", "-b", "stylish-status"]);
    fs::write(dir.path().join("tracked.txt"), "hello\nworld\n").expect("update tracked");
    fs::write(dir.path().join("fresh.txt"), "new\n").expect("write untracked");

    let output = ush()
        .args(["-s", "-c", "git status"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("git"));
    assert!(stdout.contains("status"));
    assert!(stdout.contains("stylish-status"));
    assert!(stdout.contains("tracked.txt"));
    assert!(stdout.contains("[unstaged modified]"));
    assert!(stdout.contains("fresh.txt"));
    assert!(stdout.contains("[untracked]"));
    assert!(!stdout.contains("┌"));
    assert!(!stdout.contains("│"));
}

#[test]
fn stylish_git_branch_renders_current_branch_without_tables() {
    let dir = tempdir().expect("tempdir");
    init_git_repo(dir.path());
    run_git(dir.path(), &["checkout", "-q", "-b", "stylish-branch"]);
    run_git(dir.path(), &["branch", "feature/ui"]);

    let output = ush()
        .args(["-s", "-c", "git branch"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("git"));
    assert!(stdout.contains("branch"));
    assert!(stdout.contains("stylish-branch"));
    assert!(stdout.contains("feature/ui"));
    assert!(stdout.contains("[current]"));
    assert!(!stdout.contains("┌"));
    assert!(!stdout.contains("│"));
}

#[test]
fn stylish_git_log_renders_recent_commits() {
    let dir = tempdir().expect("tempdir");
    init_git_repo(dir.path());
    run_git(dir.path(), &["checkout", "-q", "-b", "stylish-log"]);
    fs::write(dir.path().join("tracked.txt"), "hello\nworld\n").expect("update tracked");
    run_git(dir.path(), &["commit", "-qam", "second pass"]);

    let output = ush()
        .args(["-s", "-c", "git log -2"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("git"));
    assert!(stdout.contains("log"));
    assert!(stdout.contains("second pass"));
    assert!(stdout.contains("initial commit"));
    assert!(stdout.contains("HEAD -> stylish-log"));
    assert!(!stdout.contains("┌"));
    assert!(!stdout.contains("│"));
}
