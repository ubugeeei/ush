use std::{
    fs,
    io::Write,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

mod support;

use support::assert_snapshot;
use tempfile::tempdir;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

fn fixture(name: &str) -> String {
    format!("builtins/{name}.stdout")
}

fn run_with_stdin(args: &[&str], stdin: &str) -> std::process::Output {
    run_with_stdin_in_dir(args, stdin, None)
}

fn run_with_stdin_in_dir(args: &[&str], stdin: &str, dir: Option<&Path>) -> std::process::Output {
    let mut child = ush()
        .args(args)
        .current_dir(dir.unwrap_or_else(|| Path::new(".")))
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

fn normalize_command_paths(text: &str, names: &[&str]) -> String {
    let mut out = text.to_string();
    for name in names {
        let output = Command::new("/bin/sh")
            .args(["-c", &format!("command -v {name}")])
            .output()
            .expect("resolve command path");
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            out = out.replace(&path, &format!("<{}_PATH>", name.to_ascii_uppercase()));
        }
    }
    out
}

fn normalize_git_hashes(text: &str) -> String {
    let mut out = String::new();
    let mut token = String::new();

    for ch in text.chars() {
        if ch.is_ascii_hexdigit() {
            token.push(ch);
            continue;
        }
        flush_token(&mut out, &mut token);
        out.push(ch);
    }
    flush_token(&mut out, &mut token);
    out
}

fn flush_token(out: &mut String, token: &mut String) {
    if token.len() >= 7 && token.chars().all(|ch| ch.is_ascii_hexdigit()) {
        out.push_str("<SHA>");
    } else {
        out.push_str(token);
    }
    token.clear();
}

fn normalize_iso_dates(text: &str) -> String {
    let mut out = String::new();
    let chars = text.chars().collect::<Vec<_>>();
    let mut index = 0usize;

    while index < chars.len() {
        if index + 10 <= chars.len()
            && chars[index..index + 10]
                .iter()
                .enumerate()
                .all(|(offset, ch)| match offset {
                    4 | 7 => *ch == '-',
                    _ => ch.is_ascii_digit(),
                })
        {
            out.push_str("<DATE>");
            index += 10;
            continue;
        }

        out.push(chars[index]);
        index += 1;
    }

    out
}

fn normalize_ls_output(text: &str) -> String {
    let normalized = text
        .lines()
        .map(|line| {
            let mut line = line.to_string();
            if let Some(index) = line.find(" items, updated ") {
                let mut start = index;
                while start > 0 && line.as_bytes()[start - 1].is_ascii_digit() {
                    start -= 1;
                }
                line.replace_range(start..index, "<ITEMS>");
            }
            if let Some(index) = line.find("updated ") {
                line.replace_range(index + "updated ".len()..line.len(), "<TIMESTAMP>");
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("{normalized}\n")
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
    assert_snapshot(&fixture("stylish_env_sorted"), &stdout);
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
    assert_snapshot(&fixture("stylish_env_cleared"), &stdout);
}

#[test]
fn glob_builtin_expands_and_sorts_matches() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("b.txt"), "b\n").expect("write b");
    fs::write(dir.path().join("a.txt"), "a\n").expect("write a");
    fs::write(dir.path().join("note.md"), "md\n").expect("write md");

    let output = ush()
        .args(["-c", "glob '*.txt'"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "a.txt\nb.txt\n");
}

#[test]
fn glob_builtin_reads_patterns_from_stdin() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("a.txt"), "a\n").expect("write a");
    fs::write(dir.path().join("note.md"), "md\n").expect("write md");

    let mut child = ush()
        .args(["-c", "glob"])
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn ush");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(b"*.txt\n*.md\n")
        .expect("write stdin");
    let output = child.wait_with_output().expect("wait ush");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "a.txt\nnote.md\n");
}

#[test]
fn glob_builtin_returns_one_when_nothing_matches() {
    let dir = tempdir().expect("tempdir");

    let output = ush()
        .args(["-c", "glob '*.txt'"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).is_empty());
}

#[test]
fn bracket_test_builtin_returns_zero_for_true_expression() {
    let output = ush().args(["-c", "[ -d . ]"]).output().expect("run ush");
    assert!(output.status.success());
}

#[test]
fn rm_guard_rejects_split_recursive_short_flags_without_yes() {
    let dir = tempdir().expect("tempdir");
    let target = dir.path().join("target");
    fs::create_dir_all(target.join("nested")).expect("mkdir target");
    fs::write(target.join("nested/file.txt"), "keep\n").expect("write target");

    let output = run_with_stdin_in_dir(&["-c", "rm -r -f target"], "n\n", Some(dir.path()));

    assert_eq!(output.status.code(), Some(130));
    assert!(target.exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_snapshot("builtins/rm_guard_reject.stderr", &stderr);
}

#[test]
fn rm_guard_allows_recursive_delete_with_yes() {
    let dir = tempdir().expect("tempdir");
    let target = dir.path().join("target");
    fs::create_dir_all(target.join("nested")).expect("mkdir target");
    fs::write(target.join("nested/file.txt"), "remove\n").expect("write target");

    let output = ush()
        .args(["-c", "rm --yes -r -f target"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert!(!target.exists());
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
    let stdout = normalize_ls_output(&String::from_utf8_lossy(&output.stdout));
    assert!(output.stderr.is_empty());
    assert_snapshot(&fixture("stylish_ls_all"), &stdout);
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
    let stdout = normalize_ls_output(&String::from_utf8_lossy(&output.stdout));
    assert!(output.stderr.is_empty());
    assert_snapshot(&fixture("stylish_ls_lah"), &stdout);
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
    let stdout = normalize_ls_output(&String::from_utf8_lossy(&output.stdout));
    assert!(output.stderr.is_empty());
    assert_snapshot(&fixture("stylish_ls_almost_all"), &stdout);
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
    let stdout = normalize_ls_output(&String::from_utf8_lossy(&output.stdout));
    assert!(output.stderr.is_empty());
    assert_snapshot(&fixture("stylish_ls_hidden_target"), &stdout);
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
    let stdout = normalize_ls_output(&String::from_utf8_lossy(&output.stdout));
    assert!(output.stderr.is_empty());
    assert_snapshot(&fixture("stylish_ls_broken_symlink"), &stdout);
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
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot(&fixture("stylish_diff"), &stdout);
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
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot(&fixture("stylish_diff_u"), &stdout);
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
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot(&fixture("stylish_grep_file"), &stdout);
}

#[test]
fn stylish_grep_reads_pipeline_input() {
    let output = ush()
        .args(["-s", "-c", "printf 'alpha\nfoo\nbeta\n' | grep foo"])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot(&fixture("stylish_grep_stdin"), &stdout);
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
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot(&fixture("stylish_grep_no_matches"), &stdout);
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

#[test]
fn tasks_lists_discovered_workspace_tasks() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("Makefile"),
        ".PHONY: build test\nbuild:\n\t@echo build\ntest:\n\t@echo test\n",
    )
    .expect("write makefile");
    fs::write(dir.path().join("justfile"), "fmt:\n  echo fmt\n").expect("write justfile");
    fs::write(
        dir.path().join("mise.toml"),
        "[tasks.lint]\nrun = \"cargo clippy\"\n",
    )
    .expect("write mise toml");
    fs::create_dir_all(dir.path().join(".mise/tasks/frontend")).expect("mkdir mise tasks");
    fs::write(
        dir.path().join(".mise/tasks/frontend/dev"),
        "#!/usr/bin/env bash\necho dev\n",
    )
    .expect("write task script");
    fs::write(
        dir.path().join("package.json"),
        r#"{"scripts":{"build":"vite build","test:unit":"vitest"},"devDependencies":{"vite":"^7.0.0"}}"#,
    )
    .expect("write package");

    let output = ush()
        .args(["-c", "tasks"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "make build\nmake test\njust fmt\nmise run frontend/dev\nmise run lint\nnpm run build\nnpm run test:unit\nvp build\nvp dev\nvp optimize\nvp preview\nvp serve\n"
    );
}

#[test]
fn stylish_tasks_group_by_source() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("Makefile"),
        ".PHONY: build\nbuild:\n\t@echo build\n",
    )
    .expect("write makefile");
    fs::write(dir.path().join("justfile"), "fmt:\n  echo fmt\n").expect("write justfile");
    fs::write(
        dir.path().join("mise.toml"),
        "[tasks.lint]\nrun = \"cargo clippy\"\n",
    )
    .expect("write mise toml");
    fs::write(
        dir.path().join("package.json"),
        r#"{"scripts":{"build":"vite build"},"devDependencies":{"vite":"^7.0.0"}}"#,
    )
    .expect("write package");

    let output = ush()
        .args(["-s", "-c", "tasks"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot(&fixture("stylish_tasks"), &stdout);
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
    assert!(output.stderr.is_empty());
    let stdout = normalize_command_paths(&String::from_utf8_lossy(&output.stdout), &["sh"]);
    assert_snapshot(&fixture("stylish_which_multi"), &stdout);
}

#[test]
fn which_lists_all_matches_with_current_first() {
    let dir = tempdir().expect("tempdir");
    let config_path = dir.path().join("config.json");
    fs::write(
        &config_path,
        r#"{
  "aliases": {
    "echo": "printf"
  }
}
"#,
    )
    .expect("write config");

    let output = ush()
        .args([
            "--config",
            config_path.to_str().expect("utf8 path"),
            "-c",
            "which echo",
        ])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = normalize_command_paths(&String::from_utf8_lossy(&output.stdout), &["echo"]);
    assert_snapshot(&fixture("which_echo_plain"), &stdout);
}

#[test]
fn stylish_which_highlights_current_match_while_showing_all_candidates() {
    let dir = tempdir().expect("tempdir");
    let config_path = dir.path().join("config.json");
    fs::write(
        &config_path,
        r#"{
  "aliases": {
    "echo": "printf"
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
            "which echo",
        ])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = normalize_command_paths(&String::from_utf8_lossy(&output.stdout), &["echo"]);
    assert_eq!(stdout.matches("[current]").count(), 1);
    assert_snapshot(&fixture("stylish_which_echo"), &stdout);
}

#[test]
fn stylish_which_marks_missing_targets_and_preserves_exit_code() {
    let output = ush()
        .args(["-s", "-c", "which definitely-not-a-real-command"])
        .output()
        .expect("run ush");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot(&fixture("stylish_which_missing"), &stdout);
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
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot(&fixture("stylish_git_status"), &stdout);
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
    assert!(output.stderr.is_empty());
    let stdout = normalize_iso_dates(&normalize_git_hashes(&String::from_utf8_lossy(
        &output.stdout,
    )));
    assert_snapshot(&fixture("stylish_git_branch"), &stdout);
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
    assert!(output.stderr.is_empty());
    let stdout = normalize_iso_dates(&normalize_git_hashes(&String::from_utf8_lossy(
        &output.stdout,
    )));
    assert_snapshot(&fixture("stylish_git_log"), &stdout);
}
