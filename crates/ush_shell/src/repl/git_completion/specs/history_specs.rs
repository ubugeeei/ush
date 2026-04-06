use super::types::{ArgKind, GitOptionSpec};

pub(in crate::repl::git_completion::specs) const MERGE_OPTIONS: &[GitOptionSpec] = &[
    GitOptionSpec {
        names: &["--ff-only"],
        summary: "abort unless fast-forward is possible",
        value: None,
    },
    GitOptionSpec {
        names: &["--no-ff"],
        summary: "always create a merge commit",
        value: None,
    },
    GitOptionSpec {
        names: &["--squash"],
        summary: "prepare a squashed commit without merging",
        value: None,
    },
    GitOptionSpec {
        names: &["--abort"],
        summary: "abort an in-progress merge",
        value: None,
    },
];

pub(in crate::repl::git_completion::specs) const REBASE_OPTIONS: &[GitOptionSpec] = &[
    GitOptionSpec {
        names: &["-i", "--interactive"],
        summary: "edit commits during the rebase",
        value: None,
    },
    GitOptionSpec {
        names: &["--onto"],
        summary: "rebase onto a different base",
        value: Some(ArgKind::Ref),
    },
    GitOptionSpec {
        names: &["--continue"],
        summary: "continue an in-progress rebase",
        value: None,
    },
    GitOptionSpec {
        names: &["--abort"],
        summary: "abort an in-progress rebase",
        value: None,
    },
    GitOptionSpec {
        names: &["--skip"],
        summary: "skip the current patch",
        value: None,
    },
    GitOptionSpec {
        names: &["--rebase-merges"],
        summary: "preserve merge commits",
        value: None,
    },
];

pub(in crate::repl::git_completion::specs) const RESET_OPTIONS: &[GitOptionSpec] = &[
    GitOptionSpec {
        names: &["--soft"],
        summary: "move HEAD only",
        value: None,
    },
    GitOptionSpec {
        names: &["--mixed"],
        summary: "reset HEAD and index",
        value: None,
    },
    GitOptionSpec {
        names: &["--hard"],
        summary: "reset HEAD, index, and working tree",
        value: None,
    },
    GitOptionSpec {
        names: &["--keep"],
        summary: "keep local changes when possible",
        value: None,
    },
    GitOptionSpec {
        names: &["--merge"],
        summary: "keep unmerged changes",
        value: None,
    },
];

pub(in crate::repl::git_completion::specs) const REVERT_OPTIONS: &[GitOptionSpec] = &[
    GitOptionSpec {
        names: &["--no-edit"],
        summary: "reuse the generated message without editing",
        value: None,
    },
    GitOptionSpec {
        names: &["--continue"],
        summary: "continue an in-progress revert",
        value: None,
    },
    GitOptionSpec {
        names: &["--abort"],
        summary: "abort an in-progress revert",
        value: None,
    },
];

pub(in crate::repl::git_completion::specs) const CHERRY_PICK_OPTIONS: &[GitOptionSpec] = &[
    GitOptionSpec {
        names: &["-x"],
        summary: "append commit origin information",
        value: None,
    },
    GitOptionSpec {
        names: &["--continue"],
        summary: "continue an in-progress cherry-pick",
        value: None,
    },
    GitOptionSpec {
        names: &["--abort"],
        summary: "abort an in-progress cherry-pick",
        value: None,
    },
    GitOptionSpec {
        names: &["--no-commit"],
        summary: "apply changes without creating a commit",
        value: None,
    },
];

pub(in crate::repl::git_completion::specs) const DIFF_OPTIONS: &[GitOptionSpec] = &[
    GitOptionSpec {
        names: &["--staged", "--cached"],
        summary: "diff staged changes",
        value: None,
    },
    GitOptionSpec {
        names: &["--stat"],
        summary: "show a diffstat summary",
        value: None,
    },
    GitOptionSpec {
        names: &["--name-only"],
        summary: "list changed file names only",
        value: None,
    },
    GitOptionSpec {
        names: &["--word-diff"],
        summary: "show word-level changes",
        value: None,
    },
];

pub(in crate::repl::git_completion::specs) const SHOW_OPTIONS: &[GitOptionSpec] = &[
    GitOptionSpec {
        names: &["--stat"],
        summary: "show file-level statistics",
        value: None,
    },
    GitOptionSpec {
        names: &["--summary"],
        summary: "show commit summary information",
        value: None,
    },
    GitOptionSpec {
        names: &["--name-only"],
        summary: "list changed file names only",
        value: None,
    },
    GitOptionSpec {
        names: &["-p", "--patch"],
        summary: "show the full patch",
        value: None,
    },
];

pub(in crate::repl::git_completion::specs) const LOG_OPTIONS: &[GitOptionSpec] = &[
    GitOptionSpec {
        names: &["--oneline"],
        summary: "show one commit per line",
        value: None,
    },
    GitOptionSpec {
        names: &["--graph"],
        summary: "draw the commit graph",
        value: None,
    },
    GitOptionSpec {
        names: &["--decorate"],
        summary: "show ref names",
        value: None,
    },
    GitOptionSpec {
        names: &["-n", "--max-count"],
        summary: "limit the number of commits shown",
        value: Some(ArgKind::Number),
    },
    GitOptionSpec {
        names: &["--all"],
        summary: "walk all refs",
        value: None,
    },
];
