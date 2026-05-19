use super::*;

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
    assert!(stdout.contains("which"));
    assert!(stdout.contains("ll"));
    assert!(stdout.contains("[alias]"));
    assert!(stdout.contains("echo"));
    assert!(stdout.contains("[builtin]"));
    assert!(stdout.contains("shell builtin"));
    assert!(stdout.contains("sh"));
    assert!(stdout.contains("[external]"));
    assert!(stdout.contains("<SH_PATH>"));
    assert_eq!(stdout.matches("[current]").count(), 3);
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines = stdout.lines().collect::<Vec<_>>();
    assert!(!lines.is_empty());
    assert!(lines[0].starts_with("=> alias echo='printf'"));
    assert!(lines.contains(&"   echo"));
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("   /") && line.ends_with("/echo"))
    );
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.matches("[current]").count(), 1);
    assert!(stdout.contains("[alias]"));
    assert!(stdout.contains("[builtin]"));
    assert!(stdout.contains("[external]"));
    assert!(stdout.contains("printf"));
    assert!(stdout.contains("/echo"));
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
