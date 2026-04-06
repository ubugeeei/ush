use super::types::{ArgKind, GitOptionSpec};

pub(in crate::repl::git_completion::specs) const SWITCH_OPTIONS: &[GitOptionSpec] = &[
    GitOptionSpec {
        names: &["-c"],
        summary: "create and switch to a new branch",
        value: Some(ArgKind::LocalBranch),
    },
    GitOptionSpec {
        names: &["-C"],
        summary: "reset and switch to a branch",
        value: Some(ArgKind::LocalBranch),
    },
    GitOptionSpec {
        names: &["--detach"],
        summary: "switch to a detached HEAD",
        value: None,
    },
    GitOptionSpec {
        names: &["--track"],
        summary: "set up tracking for a remote branch",
        value: None,
    },
    GitOptionSpec {
        names: &["--discard-changes"],
        summary: "throw away local changes when switching",
        value: None,
    },
];

pub(in crate::repl::git_completion::specs) const CHECKOUT_OPTIONS: &[GitOptionSpec] = &[
    GitOptionSpec {
        names: &["-b"],
        summary: "create and check out a new branch",
        value: Some(ArgKind::LocalBranch),
    },
    GitOptionSpec {
        names: &["-B"],
        summary: "reset and check out a branch",
        value: Some(ArgKind::LocalBranch),
    },
    GitOptionSpec {
        names: &["--detach"],
        summary: "check out a detached HEAD",
        value: None,
    },
    GitOptionSpec {
        names: &["--track"],
        summary: "set up tracking for a remote branch",
        value: None,
    },
    GitOptionSpec {
        names: &["-p", "--patch"],
        summary: "select hunks interactively",
        value: None,
    },
];

pub(in crate::repl::git_completion::specs) const BRANCH_OPTIONS: &[GitOptionSpec] = &[
    GitOptionSpec {
        names: &["-a", "--all"],
        summary: "list local and remote branches",
        value: None,
    },
    GitOptionSpec {
        names: &["-r", "--remotes"],
        summary: "list remote-tracking branches",
        value: None,
    },
    GitOptionSpec {
        names: &["-d", "--delete"],
        summary: "delete a branch",
        value: Some(ArgKind::LocalBranch),
    },
    GitOptionSpec {
        names: &["-D"],
        summary: "force delete a branch",
        value: Some(ArgKind::LocalBranch),
    },
    GitOptionSpec {
        names: &["--show-current"],
        summary: "print the current branch name",
        value: None,
    },
    GitOptionSpec {
        names: &["--set-upstream-to"],
        summary: "change upstream tracking branch",
        value: Some(ArgKind::RemoteBranch),
    },
];

pub(in crate::repl::git_completion::specs) const PUSH_OPTIONS: &[GitOptionSpec] = &[
    GitOptionSpec {
        names: &["-u", "--set-upstream"],
        summary: "set upstream tracking after push",
        value: None,
    },
    GitOptionSpec {
        names: &["--force-with-lease"],
        summary: "force push with remote safety checks",
        value: None,
    },
    GitOptionSpec {
        names: &["--force"],
        summary: "force push",
        value: None,
    },
    GitOptionSpec {
        names: &["--tags"],
        summary: "push all tags too",
        value: None,
    },
    GitOptionSpec {
        names: &["--dry-run"],
        summary: "show what would be pushed",
        value: None,
    },
    GitOptionSpec {
        names: &["--delete"],
        summary: "delete refs on the remote",
        value: None,
    },
];

pub(in crate::repl::git_completion::specs) const PULL_OPTIONS: &[GitOptionSpec] = &[
    GitOptionSpec {
        names: &["--rebase"],
        summary: "rebase instead of merge",
        value: None,
    },
    GitOptionSpec {
        names: &["--ff-only"],
        summary: "abort unless fast-forward is possible",
        value: None,
    },
    GitOptionSpec {
        names: &["--no-rebase"],
        summary: "merge instead of rebasing",
        value: None,
    },
    GitOptionSpec {
        names: &["--tags"],
        summary: "fetch tags too",
        value: None,
    },
];

pub(in crate::repl::git_completion::specs) const FETCH_OPTIONS: &[GitOptionSpec] = &[
    GitOptionSpec {
        names: &["--all"],
        summary: "fetch all remotes",
        value: None,
    },
    GitOptionSpec {
        names: &["--prune"],
        summary: "remove stale remote-tracking refs",
        value: None,
    },
    GitOptionSpec {
        names: &["--tags"],
        summary: "fetch all tags",
        value: None,
    },
];
