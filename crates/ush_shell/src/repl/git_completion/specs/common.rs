use super::types::{ArgKind, GitOptionSpec};

pub(in crate::repl::git_completion) const GLOBAL_OPTIONS: &[GitOptionSpec] = &[
    GitOptionSpec {
        names: &["-C"],
        summary: "run as if git started in <path>",
        value: Some(ArgKind::Path),
    },
    GitOptionSpec {
        names: &["-c"],
        summary: "set a temporary config value",
        value: Some(ArgKind::Config),
    },
    GitOptionSpec {
        names: &["--git-dir"],
        summary: "use an alternate repository path",
        value: Some(ArgKind::Path),
    },
    GitOptionSpec {
        names: &["--work-tree"],
        summary: "use an alternate working tree",
        value: Some(ArgKind::Path),
    },
    GitOptionSpec {
        names: &["-p", "--paginate"],
        summary: "pipe output into a pager",
        value: None,
    },
    GitOptionSpec {
        names: &["--no-pager"],
        summary: "disable the pager",
        value: None,
    },
    GitOptionSpec {
        names: &["-h", "--help"],
        summary: "show help",
        value: None,
    },
    GitOptionSpec {
        names: &["--version"],
        summary: "show git version",
        value: None,
    },
];

pub(in crate::repl::git_completion::specs) const STATUS_OPTIONS: &[GitOptionSpec] = &[
    GitOptionSpec {
        names: &["-s", "--short"],
        summary: "show the short status format",
        value: None,
    },
    GitOptionSpec {
        names: &["-b", "--branch"],
        summary: "show branch information",
        value: None,
    },
    GitOptionSpec {
        names: &["--porcelain"],
        summary: "emit machine-readable status",
        value: None,
    },
    GitOptionSpec {
        names: &["--ignored"],
        summary: "show ignored files too",
        value: None,
    },
];

pub(in crate::repl::git_completion::specs) const ADD_OPTIONS: &[GitOptionSpec] = &[
    GitOptionSpec {
        names: &["-A", "--all"],
        summary: "stage all tracked and untracked changes",
        value: None,
    },
    GitOptionSpec {
        names: &["-p", "--patch"],
        summary: "stage hunks interactively",
        value: None,
    },
    GitOptionSpec {
        names: &["-u", "--update"],
        summary: "stage tracked file updates only",
        value: None,
    },
    GitOptionSpec {
        names: &["-N", "--intent-to-add"],
        summary: "record intent to add",
        value: None,
    },
];

pub(in crate::repl::git_completion::specs) const RESTORE_OPTIONS: &[GitOptionSpec] = &[
    GitOptionSpec {
        names: &["--source"],
        summary: "restore from a specific tree-ish",
        value: Some(ArgKind::Ref),
    },
    GitOptionSpec {
        names: &["--staged"],
        summary: "restore the index",
        value: None,
    },
    GitOptionSpec {
        names: &["--worktree"],
        summary: "restore the working tree",
        value: None,
    },
    GitOptionSpec {
        names: &["-p", "--patch"],
        summary: "select hunks interactively",
        value: None,
    },
];

pub(in crate::repl::git_completion::specs) const COMMIT_OPTIONS: &[GitOptionSpec] = &[
    GitOptionSpec {
        names: &["-m", "--message"],
        summary: "set the commit message",
        value: Some(ArgKind::Message),
    },
    GitOptionSpec {
        names: &["-a", "--all"],
        summary: "stage modified tracked files before committing",
        value: None,
    },
    GitOptionSpec {
        names: &["--amend"],
        summary: "rewrite the previous commit",
        value: None,
    },
    GitOptionSpec {
        names: &["--fixup"],
        summary: "create a fixup commit for <commit>",
        value: Some(ArgKind::Commit),
    },
    GitOptionSpec {
        names: &["--reuse-message"],
        summary: "reuse the message from <commit>",
        value: Some(ArgKind::Commit),
    },
    GitOptionSpec {
        names: &["--no-edit"],
        summary: "reuse the previous commit message without editing",
        value: None,
    },
];
