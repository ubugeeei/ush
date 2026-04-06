use std::{
    path::{Path, PathBuf},
    process::Command,
};

use rustyline::{Context, completion::Completer, hint::Hinter, history::DefaultHistory};
use tempfile::tempdir;

use crate::repl::UshHelper;

fn helper(cwd: PathBuf) -> UshHelper {
    UshHelper::new(vec!["git".to_string()], vec!["PATH".to_string()], cwd)
}

fn git(cwd: &Path, args: &[&str]) {
    let status = Command::new("git")
        .current_dir(cwd)
        .args(args)
        .status()
        .expect("run git");
    assert!(status.success(), "git {:?} failed", args);
}

fn seed_repo() -> tempfile::TempDir {
    let dir = tempdir().expect("tempdir");
    git(dir.path(), &["init"]);
    git(dir.path(), &["config", "user.name", "ush test"]);
    git(dir.path(), &["config", "user.email", "ush@example.com"]);
    std::fs::write(dir.path().join("tracked.txt"), "hello\n").expect("write tracked file");
    git(dir.path(), &["add", "tracked.txt"]);
    git(dir.path(), &["commit", "-m", "initial commit"]);
    git(dir.path(), &["checkout", "-b", "feature/git-hints"]);
    git(dir.path(), &["tag", "v0.1.0"]);
    git(
        dir.path(),
        &["remote", "add", "origin", "git@example.com:repo.git"],
    );
    std::fs::write(dir.path().join("dirty.txt"), "dirty\n").expect("write dirty file");
    git(dir.path(), &["add", "dirty.txt"]);
    git(dir.path(), &["stash", "push", "-m", "temp work"]);
    dir
}

#[test]
fn completes_git_subcommands_with_descriptions() {
    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    let (_, pairs) = helper(PathBuf::from("."))
        .complete("git ch", 6, &ctx)
        .expect("complete");

    assert!(pairs.iter().any(|pair| pair.replacement == "checkout"));
    assert!(pairs.iter().any(|pair| {
        pair.replacement == "cherry-pick" && pair.display.contains("apply the changes")
    }));
}

#[test]
fn shows_commit_help_hint() {
    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    let helper = helper(PathBuf::from("."));

    assert_eq!(
        helper.hint("git commit ", 11, &ctx),
        Some(
            "  record changes to the repository  usage: [-m <message>] [-a] [--amend] [--fixup <commit>] [--] [<pathspec>...]".to_string()
        )
    );
}

#[test]
fn completes_remotes_branches_and_stashes_from_repo_state() {
    let repo = seed_repo();
    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    let helper = helper(repo.path().to_path_buf());

    let (_, push_pairs) = helper
        .complete("git push or", 11, &ctx)
        .expect("push complete");
    let (_, switch_pairs) = helper
        .complete("git switch fe", 13, &ctx)
        .expect("switch complete");
    let (_, stash_pairs) = helper
        .complete("git stash st", 12, &ctx)
        .expect("stash complete");

    assert!(push_pairs.iter().any(|pair| pair.replacement == "origin"));
    assert!(
        switch_pairs
            .iter()
            .any(|pair| pair.replacement == "feature/git-hints")
    );
    assert!(
        stash_pairs
            .iter()
            .any(|pair| pair.replacement.starts_with("stash@{"))
    );
}

#[test]
fn completes_pathspecs_for_git_add_without_a_slash_prefix() {
    let repo = seed_repo();
    std::fs::write(repo.path().join("src.txt"), "src\n").expect("write src file");
    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    let helper = helper(repo.path().to_path_buf());

    let (_, pairs) = helper.complete("git add sr", 10, &ctx).expect("complete");

    assert!(pairs.iter().any(|pair| pair.replacement == "src.txt"));
}
