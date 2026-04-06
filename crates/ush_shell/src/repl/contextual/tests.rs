mod extra;

use std::{fs, path::Path, process::Command};

use rustyline::{Context, completion::Completer, history::DefaultHistory};
use tempfile::tempdir;

use crate::repl::UshHelper;

fn helper(cwd: &Path) -> UshHelper {
    UshHelper::new(
        vec![
            "bun".to_string(),
            "cargo".to_string(),
            "claude".to_string(),
            "codex".to_string(),
            "git".to_string(),
            "go".to_string(),
            "just".to_string(),
            "make".to_string(),
            "mise".to_string(),
            "moon".to_string(),
            "nix".to_string(),
            "node".to_string(),
            "npm".to_string(),
            "pnpm".to_string(),
            "vp".to_string(),
            "yarn".to_string(),
            "zig".to_string(),
        ],
        vec!["PATH".to_string()],
        cwd.to_path_buf(),
    )
}

#[test]
fn completes_git_subcommands_and_refs() {
    let dir = tempdir().expect("tempdir");
    run(dir.path(), &["init", "-q"]);
    run(dir.path(), &["config", "user.name", "ush"]);
    run(dir.path(), &["config", "user.email", "ush@example.com"]);
    fs::write(dir.path().join("tracked.txt"), "hello\n").expect("write file");
    run(dir.path(), &["add", "tracked.txt"]);
    run(dir.path(), &["commit", "-q", "-m", "init"]);
    run(dir.path(), &["branch", "feature/ui"]);
    run(dir.path(), &["tag", "v1.0.0"]);

    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    let helper = helper(dir.path());

    let (_, commands) = helper.complete("git ch", 6, &ctx).expect("git commands");
    let (_, refs) = helper
        .complete("git checkout fe", 15, &ctx)
        .expect("git refs");

    assert!(commands.iter().any(|pair| pair.replacement == "checkout"));
    assert!(refs.iter().any(|pair| pair.replacement == "feature/ui"));
}

#[test]
fn completes_make_targets() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("Makefile"),
        ".PHONY: build test\nbuild:\n\t@echo build\ntest:\n\t@echo test\n",
    )
    .expect("write makefile");
    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    let helper = helper(dir.path());

    let (_, pairs) = helper.complete("make b", 6, &ctx).expect("make targets");
    assert!(pairs.iter().any(|pair| pair.replacement == "build"));
}

#[test]
fn completes_just_recipes() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("justfile"),
        "set shell := [\"bash\", \"-cu\"]\nbuild:\n  echo build\ntest unit:\n  echo test\n",
    )
    .expect("write justfile");
    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    let helper = helper(dir.path());

    let (_, pairs) = helper.complete("just te", 7, &ctx).expect("just recipes");
    assert!(pairs.iter().any(|pair| pair.replacement == "test"));
}

#[test]
fn completes_mise_tasks() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("mise.toml"),
        "[tasks.build]\nrun = \"npm run build\"\n",
    )
    .expect("write mise toml");
    let task_dir = dir.path().join(".mise/tasks");
    fs::create_dir_all(&task_dir).expect("mkdir");
    fs::write(task_dir.join("lint"), "#!/usr/bin/env bash\necho lint\n").expect("write task");
    let nested_dir = task_dir.join("frontend");
    fs::create_dir_all(&nested_dir).expect("mkdir nested");
    fs::write(nested_dir.join("dev"), "#!/usr/bin/env bash\necho dev\n")
        .expect("write nested task");
    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    let helper = helper(dir.path());

    let (_, run_pairs) = helper.complete("mise run b", 10, &ctx).expect("mise run");
    let (_, top_level_pairs) = helper.complete("mise li", 7, &ctx).expect("mise top-level");
    let (_, nested_pairs) = helper
        .complete("mise run frontend/", 18, &ctx)
        .expect("mise nested");

    assert!(run_pairs.iter().any(|pair| pair.replacement == "build"));
    assert!(
        top_level_pairs
            .iter()
            .any(|pair| pair.replacement == "lint")
    );
    assert!(
        nested_pairs
            .iter()
            .any(|pair| pair.replacement == "frontend/dev")
    );
}

#[test]
fn completes_npm_scripts() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("package.json"),
        r#"{"scripts":{"build":"vite build","test:unit":"vitest"}}"#,
    )
    .expect("write package");
    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    let helper = helper(dir.path());

    let (_, pairs) = helper
        .complete("npm run bu", 10, &ctx)
        .expect("npm scripts");
    assert!(pairs.iter().any(|pair| pair.replacement == "build"));
}

#[test]
fn completes_vite_commands() {
    let dir = tempdir().expect("tempdir");
    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    let helper = helper(dir.path());

    let (_, pairs) = helper.complete("vp bu", 5, &ctx).expect("vp commands");
    assert!(pairs.iter().any(|pair| pair.replacement == "build"));
}

fn run(dir: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
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
