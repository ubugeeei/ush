use super::types::{ArgKind, GitOptionSpec};

pub(in crate::repl::git_completion::specs) const CLONE_OPTIONS: &[GitOptionSpec] = &[
    GitOptionSpec {
        names: &["--depth"],
        summary: "create a shallow clone",
        value: Some(ArgKind::Number),
    },
    GitOptionSpec {
        names: &["-b", "--branch"],
        summary: "check out a specific branch or tag",
        value: Some(ArgKind::Branch),
    },
    GitOptionSpec {
        names: &["--origin"],
        summary: "set the remote name",
        value: Some(ArgKind::Remote),
    },
    GitOptionSpec {
        names: &["--recurse-submodules"],
        summary: "initialize submodules too",
        value: None,
    },
];

pub(in crate::repl::git_completion::specs) const INIT_OPTIONS: &[GitOptionSpec] = &[
    GitOptionSpec {
        names: &["--bare"],
        summary: "create a bare repository",
        value: None,
    },
    GitOptionSpec {
        names: &["-b", "--initial-branch"],
        summary: "name the initial branch",
        value: Some(ArgKind::LocalBranch),
    },
];

pub(in crate::repl::git_completion::specs) const TAG_OPTIONS: &[GitOptionSpec] = &[
    GitOptionSpec {
        names: &["-a"],
        summary: "create an annotated tag",
        value: None,
    },
    GitOptionSpec {
        names: &["-d"],
        summary: "delete a tag",
        value: Some(ArgKind::Tag),
    },
    GitOptionSpec {
        names: &["-l"],
        summary: "list tags",
        value: None,
    },
];

pub(in crate::repl::git_completion::specs) const STASH_OPTIONS: &[GitOptionSpec] = &[
    GitOptionSpec {
        names: &["push"],
        summary: "stash local changes",
        value: None,
    },
    GitOptionSpec {
        names: &["pop"],
        summary: "apply and drop the latest stash",
        value: None,
    },
    GitOptionSpec {
        names: &["apply"],
        summary: "apply a stash",
        value: None,
    },
    GitOptionSpec {
        names: &["list"],
        summary: "list stashes",
        value: None,
    },
    GitOptionSpec {
        names: &["show"],
        summary: "show a stash patch",
        value: None,
    },
    GitOptionSpec {
        names: &["drop"],
        summary: "delete a stash entry",
        value: None,
    },
];
