use std::{collections::BTreeSet, path::Path, process::Command};

use rustyline::completion::Pair;

use super::{UshHelper, syntax};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArgKind {
    Branch,
    Commit,
    Config,
    LocalBranch,
    Message,
    Number,
    Path,
    Pathspec,
    Ref,
    Remote,
    RemoteBranch,
    Stash,
    Tag,
    Url,
}

impl ArgKind {
    fn placeholder(self) -> &'static str {
        match self {
            Self::Branch | Self::LocalBranch => "<branch>",
            Self::Commit => "<commit>",
            Self::Config => "<name=value>",
            Self::Message => "<message>",
            Self::Number => "<n>",
            Self::Path => "<path>",
            Self::Pathspec => "<pathspec>",
            Self::Ref => "<ref>",
            Self::Remote => "<remote>",
            Self::RemoteBranch => "<remote/branch>",
            Self::Stash => "<stash>",
            Self::Tag => "<tag>",
            Self::Url => "<url>",
        }
    }

    fn path_like(self) -> bool {
        matches!(self, Self::Path | Self::Pathspec)
    }
}

#[derive(Clone, Copy, Debug)]
struct GitOptionSpec {
    names: &'static [&'static str],
    summary: &'static str,
    value: Option<ArgKind>,
}

#[derive(Clone, Copy, Debug)]
struct GitCommandSpec {
    name: &'static str,
    summary: &'static str,
    usage: &'static str,
    options: &'static [GitOptionSpec],
    positionals: &'static [ArgKind],
    trailing: Option<ArgKind>,
    after_double_dash: Option<ArgKind>,
}

impl GitCommandSpec {
    fn positional_kind(self, index: usize) -> Option<ArgKind> {
        self.positionals
            .get(index)
            .copied()
            .or(if index >= self.positionals.len() {
                self.trailing
            } else {
                None
            })
    }
}

const GLOBAL_OPTIONS: &[GitOptionSpec] = &[
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

const STATUS_OPTIONS: &[GitOptionSpec] = &[
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

const ADD_OPTIONS: &[GitOptionSpec] = &[
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

const RESTORE_OPTIONS: &[GitOptionSpec] = &[
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

const COMMIT_OPTIONS: &[GitOptionSpec] = &[
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

const SWITCH_OPTIONS: &[GitOptionSpec] = &[
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

const CHECKOUT_OPTIONS: &[GitOptionSpec] = &[
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

const BRANCH_OPTIONS: &[GitOptionSpec] = &[
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

const PUSH_OPTIONS: &[GitOptionSpec] = &[
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

const PULL_OPTIONS: &[GitOptionSpec] = &[
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

const FETCH_OPTIONS: &[GitOptionSpec] = &[
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

const MERGE_OPTIONS: &[GitOptionSpec] = &[
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

const REBASE_OPTIONS: &[GitOptionSpec] = &[
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

const RESET_OPTIONS: &[GitOptionSpec] = &[
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

const REVERT_OPTIONS: &[GitOptionSpec] = &[
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

const CHERRY_PICK_OPTIONS: &[GitOptionSpec] = &[
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

const DIFF_OPTIONS: &[GitOptionSpec] = &[
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

const SHOW_OPTIONS: &[GitOptionSpec] = &[
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

const LOG_OPTIONS: &[GitOptionSpec] = &[
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

const CLONE_OPTIONS: &[GitOptionSpec] = &[
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

const INIT_OPTIONS: &[GitOptionSpec] = &[
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

const TAG_OPTIONS: &[GitOptionSpec] = &[
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

const STASH_OPTIONS: &[GitOptionSpec] = &[
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

const GIT_COMMANDS: &[GitCommandSpec] = &[
    GitCommandSpec {
        name: "status",
        summary: "show the working tree status",
        usage: "[--short] [--branch] [--] [<pathspec>...]",
        options: STATUS_OPTIONS,
        positionals: &[],
        trailing: Some(ArgKind::Pathspec),
        after_double_dash: Some(ArgKind::Pathspec),
    },
    GitCommandSpec {
        name: "add",
        summary: "stage file contents",
        usage: "[-A] [-p] [-u] [--] [<pathspec>...]",
        options: ADD_OPTIONS,
        positionals: &[],
        trailing: Some(ArgKind::Pathspec),
        after_double_dash: Some(ArgKind::Pathspec),
    },
    GitCommandSpec {
        name: "restore",
        summary: "restore files in the working tree or index",
        usage: "[--source <ref>] [--staged] [--worktree] [--] [<pathspec>...]",
        options: RESTORE_OPTIONS,
        positionals: &[],
        trailing: Some(ArgKind::Pathspec),
        after_double_dash: Some(ArgKind::Pathspec),
    },
    GitCommandSpec {
        name: "commit",
        summary: "record changes to the repository",
        usage: "[-m <message>] [-a] [--amend] [--fixup <commit>] [--] [<pathspec>...]",
        options: COMMIT_OPTIONS,
        positionals: &[],
        trailing: None,
        after_double_dash: Some(ArgKind::Pathspec),
    },
    GitCommandSpec {
        name: "switch",
        summary: "switch branches",
        usage: "[-c <branch>] [-C <branch>] [--detach] [<branch>]",
        options: SWITCH_OPTIONS,
        positionals: &[ArgKind::Branch],
        trailing: None,
        after_double_dash: None,
    },
    GitCommandSpec {
        name: "checkout",
        summary: "switch branches or restore files",
        usage: "[-b <branch>] [-B <branch>] [--detach] [<ref>] [--] [<pathspec>...]",
        options: CHECKOUT_OPTIONS,
        positionals: &[ArgKind::Ref],
        trailing: None,
        after_double_dash: Some(ArgKind::Pathspec),
    },
    GitCommandSpec {
        name: "branch",
        summary: "list, create, or delete branches",
        usage: "[-a|-r] [<branch>] [<start-point>]",
        options: BRANCH_OPTIONS,
        positionals: &[ArgKind::LocalBranch, ArgKind::Ref],
        trailing: None,
        after_double_dash: None,
    },
    GitCommandSpec {
        name: "push",
        summary: "update remote refs",
        usage: "[-u] [--force-with-lease] [<remote>] [<branch>]",
        options: PUSH_OPTIONS,
        positionals: &[ArgKind::Remote, ArgKind::Branch],
        trailing: None,
        after_double_dash: None,
    },
    GitCommandSpec {
        name: "pull",
        summary: "fetch from and integrate with another repository",
        usage: "[--rebase|--ff-only] [<remote>] [<remote/branch>]",
        options: PULL_OPTIONS,
        positionals: &[ArgKind::Remote, ArgKind::RemoteBranch],
        trailing: None,
        after_double_dash: None,
    },
    GitCommandSpec {
        name: "fetch",
        summary: "download refs from another repository",
        usage: "[--all] [--prune] [<remote>] [<remote/branch>]",
        options: FETCH_OPTIONS,
        positionals: &[ArgKind::Remote, ArgKind::RemoteBranch],
        trailing: None,
        after_double_dash: None,
    },
    GitCommandSpec {
        name: "merge",
        summary: "join development histories together",
        usage: "[--ff-only|--no-ff] [--squash] <ref>...",
        options: MERGE_OPTIONS,
        positionals: &[ArgKind::Ref],
        trailing: Some(ArgKind::Ref),
        after_double_dash: None,
    },
    GitCommandSpec {
        name: "rebase",
        summary: "reapply commits on top of another base",
        usage: "[-i] [--onto <ref>] [<upstream>] [<branch>]",
        options: REBASE_OPTIONS,
        positionals: &[ArgKind::Ref, ArgKind::Branch],
        trailing: None,
        after_double_dash: None,
    },
    GitCommandSpec {
        name: "reset",
        summary: "reset current HEAD to the specified state",
        usage: "[--soft|--mixed|--hard] [<commit>] [--] [<pathspec>...]",
        options: RESET_OPTIONS,
        positionals: &[ArgKind::Commit],
        trailing: None,
        after_double_dash: Some(ArgKind::Pathspec),
    },
    GitCommandSpec {
        name: "revert",
        summary: "create commits that reverse earlier commits",
        usage: "[--no-edit] <commit>...",
        options: REVERT_OPTIONS,
        positionals: &[ArgKind::Commit],
        trailing: Some(ArgKind::Commit),
        after_double_dash: None,
    },
    GitCommandSpec {
        name: "cherry-pick",
        summary: "apply the changes from existing commits",
        usage: "[-x] [--no-commit] <commit>...",
        options: CHERRY_PICK_OPTIONS,
        positionals: &[ArgKind::Commit],
        trailing: Some(ArgKind::Commit),
        after_double_dash: None,
    },
    GitCommandSpec {
        name: "diff",
        summary: "show changes between commits, branches, or files",
        usage: "[--staged] [<ref>] [<ref>] [--] [<pathspec>...]",
        options: DIFF_OPTIONS,
        positionals: &[ArgKind::Ref, ArgKind::Ref],
        trailing: None,
        after_double_dash: Some(ArgKind::Pathspec),
    },
    GitCommandSpec {
        name: "show",
        summary: "show an object",
        usage: "[--stat] [--summary] [<object>]",
        options: SHOW_OPTIONS,
        positionals: &[ArgKind::Ref],
        trailing: None,
        after_double_dash: None,
    },
    GitCommandSpec {
        name: "log",
        summary: "show commit logs",
        usage: "[--oneline] [--graph] [-n <n>] [<ref>] [--] [<pathspec>...]",
        options: LOG_OPTIONS,
        positionals: &[ArgKind::Ref],
        trailing: None,
        after_double_dash: Some(ArgKind::Pathspec),
    },
    GitCommandSpec {
        name: "clone",
        summary: "clone a repository into a new directory",
        usage: "[--depth <n>] [-b <branch>] <url> [<path>]",
        options: CLONE_OPTIONS,
        positionals: &[ArgKind::Url, ArgKind::Path],
        trailing: None,
        after_double_dash: None,
    },
    GitCommandSpec {
        name: "init",
        summary: "create an empty Git repository",
        usage: "[--bare] [-b <branch>] [<path>]",
        options: INIT_OPTIONS,
        positionals: &[ArgKind::Path],
        trailing: None,
        after_double_dash: None,
    },
    GitCommandSpec {
        name: "tag",
        summary: "create, list, delete, or verify tags",
        usage: "[-a] [-d <tag>] [<tag>] [<commit>]",
        options: TAG_OPTIONS,
        positionals: &[ArgKind::Tag, ArgKind::Commit],
        trailing: None,
        after_double_dash: None,
    },
    GitCommandSpec {
        name: "stash",
        summary: "stash the changes in a dirty working directory",
        usage: "[push|pop|apply|list|show|drop] [<stash>]",
        options: STASH_OPTIONS,
        positionals: &[ArgKind::Stash],
        trailing: None,
        after_double_dash: None,
    },
];

#[derive(Debug, Default)]
struct ParsedOptions {
    positionals: Vec<String>,
    expected_value: Option<ArgKind>,
    after_double_dash: bool,
}

#[derive(Debug)]
struct GitContext {
    start: usize,
    word: String,
    subcommand: Option<&'static GitCommandSpec>,
    raw_subcommand: Option<String>,
    global_expected_value: Option<ArgKind>,
    subcommand_args: Vec<String>,
}

pub fn complete(
    helper: &UshHelper,
    line: &str,
    pos: usize,
) -> rustyline::Result<Option<(usize, Vec<Pair>)>> {
    let Some(context) = parse_context(line, pos) else {
        return Ok(None);
    };

    if let Some(kind) = context.global_expected_value {
        return complete_arg(helper, context.start, &context.word, kind).map(Some);
    }

    let Some(spec) = context.subcommand else {
        if context.raw_subcommand.is_some() {
            return Ok(Some((context.start, Vec::new())));
        }

        let mut pairs = command_pairs(&context.word);
        if context.word.is_empty() || context.word.starts_with('-') {
            pairs.extend(option_pairs(&context.word, GLOBAL_OPTIONS));
        }
        return Ok(Some((context.start, dedupe_pairs(pairs))));
    };

    let parsed = parse_options(&context.subcommand_args, spec.options);
    if let Some(kind) = parsed.expected_value {
        return complete_arg(helper, context.start, &context.word, kind).map(Some);
    }

    if !parsed.after_double_dash && context.word.starts_with('-') {
        return Ok(Some((
            context.start,
            option_pairs(&context.word, spec.options),
        )));
    }

    let expected = if parsed.after_double_dash {
        spec.after_double_dash
    } else {
        spec.positional_kind(parsed.positionals.len())
    };

    let mut pairs = if let Some(kind) = expected {
        complete_arg(helper, context.start, &context.word, kind)?.1
    } else {
        Vec::new()
    };

    if context.word.is_empty() && !parsed.after_double_dash {
        pairs.extend(option_pairs("", spec.options));
        pairs = dedupe_pairs(pairs);
    }

    Ok(Some((context.start, pairs)))
}

pub fn hint(_helper: &UshHelper, line: &str, pos: usize) -> Option<String> {
    let context = parse_context(line, pos)?;

    if let Some(kind) = context.global_expected_value {
        return Some(format!("  global option value: {}", kind.placeholder()));
    }

    let Some(spec) = context.subcommand else {
        if context.raw_subcommand.is_some() {
            return None;
        }
        if context.word.starts_with('-') {
            let options = matching_options(&context.word, GLOBAL_OPTIONS);
            return option_hint(options);
        }
        let matches = matching_commands(&context.word);
        return if matches.len() == 1 {
            Some(command_hint(matches[0], None))
        } else {
            Some("  next: <subcommand>  common: status, add, commit, switch, checkout, branch, push, pull, fetch".to_string())
        };
    };

    let parsed = parse_options(&context.subcommand_args, spec.options);
    if let Some(kind) = parsed.expected_value {
        return Some(command_hint(spec, Some(kind)));
    }
    if !parsed.after_double_dash && context.word.starts_with('-') {
        let options = matching_options(&context.word, spec.options);
        return option_hint(options).or_else(|| Some(command_hint(spec, None)));
    }

    let expected = if parsed.after_double_dash {
        spec.after_double_dash
    } else {
        spec.positional_kind(parsed.positionals.len())
    };
    Some(command_hint(spec, expected))
}

fn parse_context(line: &str, pos: usize) -> Option<GitContext> {
    let prefix = &line[..pos];
    let start = syntax::word_start(line, pos);
    let word = prefix[start..].to_string();
    let tokens = syntax::tokenize(&prefix[..start]);
    let segment_start = tokens
        .iter()
        .rposition(|token| matches!(token.as_str(), "|" | "||" | "&&" | ";" | "&"))
        .map_or(0, |index| index + 1);
    let segment = &tokens[segment_start..];

    let command_index = segment.iter().position(|token| {
        !(syntax::is_assignment(token) || matches!(token.as_str(), "!" | "command"))
    })?;

    if segment.get(command_index).map(String::as_str) != Some("git") {
        return None;
    }

    let mut index = command_index + 1;
    let mut global_expected_value = None;
    while let Some(token) = segment.get(index) {
        if let Some(option) = find_option(token, GLOBAL_OPTIONS) {
            if option_consumes_inline_value(token, option) {
                index += 1;
                continue;
            }
            if let Some(kind) = option.value {
                if segment.get(index + 1).is_some() {
                    index += 2;
                } else {
                    global_expected_value = Some(kind);
                    index += 1;
                    break;
                }
            } else {
                index += 1;
            }
            continue;
        }
        break;
    }

    let raw_subcommand = segment.get(index).cloned();
    let subcommand = raw_subcommand
        .as_deref()
        .and_then(|name| GIT_COMMANDS.iter().find(|spec| spec.name == name));
    let subcommand_args = if subcommand.is_some() {
        segment[index + 1..].to_vec()
    } else {
        Vec::new()
    };

    Some(GitContext {
        start,
        word,
        subcommand,
        raw_subcommand,
        global_expected_value,
        subcommand_args,
    })
}

fn parse_options(tokens: &[String], options: &[GitOptionSpec]) -> ParsedOptions {
    let mut parsed = ParsedOptions::default();
    let mut index = 0usize;

    while let Some(token) = tokens.get(index) {
        if parsed.after_double_dash {
            parsed.positionals.push(token.clone());
            index += 1;
            continue;
        }
        if token == "--" {
            parsed.after_double_dash = true;
            index += 1;
            continue;
        }
        if let Some(option) = find_option(token, options) {
            if option_consumes_inline_value(token, option) {
                index += 1;
                continue;
            }
            if let Some(kind) = option.value {
                if let Some(value) = tokens.get(index + 1)
                    && value != "--"
                {
                    index += 2;
                    continue;
                }
                parsed.expected_value = Some(kind);
                break;
            }
            index += 1;
            continue;
        }
        parsed.positionals.push(token.clone());
        index += 1;
    }

    parsed
}

fn complete_arg(
    helper: &UshHelper,
    start: usize,
    word: &str,
    kind: ArgKind,
) -> rustyline::Result<(usize, Vec<Pair>)> {
    if kind.path_like() {
        return Ok((start, complete_path_pairs(helper.cwd.as_path(), word)));
    }
    Ok((start, pairs_for_arg(helper.cwd.as_path(), kind, word)))
}

fn pairs_for_arg(cwd: &Path, kind: ArgKind, needle: &str) -> Vec<Pair> {
    match kind {
        ArgKind::Branch => dedupe_pairs(branch_pairs(cwd, needle)),
        ArgKind::Commit => dedupe_pairs(commit_pairs(cwd, needle)),
        ArgKind::Config | ArgKind::Message | ArgKind::Number | ArgKind::Url => Vec::new(),
        ArgKind::LocalBranch => local_branch_pairs(cwd, needle),
        ArgKind::Path | ArgKind::Pathspec => Vec::new(),
        ArgKind::Ref => dedupe_pairs(ref_pairs(cwd, needle)),
        ArgKind::Remote => remote_pairs(cwd, needle),
        ArgKind::RemoteBranch => remote_branch_pairs(cwd, needle),
        ArgKind::Stash => stash_pairs(cwd, needle),
        ArgKind::Tag => tag_pairs(cwd, needle),
    }
}

fn command_pairs(prefix: &str) -> Vec<Pair> {
    GIT_COMMANDS
        .iter()
        .filter(|spec| spec.name.starts_with(prefix))
        .map(|spec| described_pair(spec.name, spec.summary))
        .collect()
}

fn matching_commands(prefix: &str) -> Vec<&'static GitCommandSpec> {
    GIT_COMMANDS
        .iter()
        .filter(|spec| spec.name.starts_with(prefix))
        .collect()
}

fn option_pairs(prefix: &str, options: &[GitOptionSpec]) -> Vec<Pair> {
    dedupe_pairs(
        options
            .iter()
            .flat_map(|option| {
                option
                    .names
                    .iter()
                    .filter(move |name| name.starts_with(prefix))
                    .map(move |name| described_pair(name, &option_summary(option)))
            })
            .collect(),
    )
}

fn matching_options<'a>(prefix: &str, options: &'a [GitOptionSpec]) -> Vec<&'a GitOptionSpec> {
    options
        .iter()
        .filter(|option| option.names.iter().any(|name| name.starts_with(prefix)))
        .collect()
}

fn option_hint(options: Vec<&GitOptionSpec>) -> Option<String> {
    if options.len() != 1 {
        return None;
    }
    let option = options[0];
    let name = option.names.iter().max_by_key(|name| name.len()).copied()?;
    let value = option
        .value
        .map(|kind| format!(" {}", kind.placeholder()))
        .unwrap_or_default();
    Some(format!("  {name}{value}  {}", option.summary))
}

fn command_hint(spec: &GitCommandSpec, expected: Option<ArgKind>) -> String {
    let mut hint = format!("  {}", spec.summary);
    if let Some(kind) = expected {
        hint.push_str("  next: ");
        hint.push_str(kind.placeholder());
    }
    hint.push_str("  usage: ");
    hint.push_str(spec.usage);
    hint
}

fn local_branch_pairs(cwd: &Path, needle: &str) -> Vec<Pair> {
    ref_pairs_for_namespace(cwd, "refs/heads", needle, "local branch")
}

fn remote_branch_pairs(cwd: &Path, needle: &str) -> Vec<Pair> {
    ref_pairs_for_namespace(cwd, "refs/remotes", needle, "remote branch")
        .into_iter()
        .filter(|pair| !pair.replacement.ends_with("/HEAD"))
        .collect()
}

fn tag_pairs(cwd: &Path, needle: &str) -> Vec<Pair> {
    ref_pairs_for_namespace(cwd, "refs/tags", needle, "tag")
}

fn branch_pairs(cwd: &Path, needle: &str) -> Vec<Pair> {
    let mut pairs = local_branch_pairs(cwd, needle);
    pairs.extend(remote_branch_pairs(cwd, needle));
    pairs.extend(tag_pairs(cwd, needle));
    dedupe_pairs(pairs)
}

fn ref_pairs(cwd: &Path, needle: &str) -> Vec<Pair> {
    let mut pairs = branch_pairs(cwd, needle);
    pairs.extend(recent_commit_pairs(cwd, needle));
    dedupe_pairs(pairs)
}

fn commit_pairs(cwd: &Path, needle: &str) -> Vec<Pair> {
    let mut pairs = recent_commit_pairs(cwd, needle);
    pairs.extend(branch_pairs(cwd, needle));
    dedupe_pairs(pairs)
}

fn ref_pairs_for_namespace(cwd: &Path, namespace: &str, needle: &str, label: &str) -> Vec<Pair> {
    git_rows(
        cwd,
        &[
            "for-each-ref",
            "--format=%(refname:short)%09%(subject)",
            namespace,
        ],
    )
    .into_iter()
    .filter_map(|row| {
        let (name, subject) = split_row(&row);
        name.starts_with(needle).then(|| {
            let summary = if subject.is_empty() {
                label.to_string()
            } else {
                format!("{label}  {subject}")
            };
            described_pair(&name, &summary)
        })
    })
    .collect()
}

fn remote_pairs(cwd: &Path, needle: &str) -> Vec<Pair> {
    git_rows(cwd, &["remote"])
        .into_iter()
        .filter(|remote| remote.starts_with(needle))
        .map(|remote| described_pair(&remote, "remote"))
        .collect()
}

fn stash_pairs(cwd: &Path, needle: &str) -> Vec<Pair> {
    git_rows(cwd, &["stash", "list", "--format=%gd%x09%s"])
        .into_iter()
        .filter_map(|row| {
            let (name, subject) = split_row(&row);
            name.starts_with(needle)
                .then(|| described_pair(&name, &format!("stash  {subject}")))
        })
        .collect()
}

fn recent_commit_pairs(cwd: &Path, needle: &str) -> Vec<Pair> {
    git_rows(cwd, &["log", "--format=%h%x09%s", "-n", "30", "--all"])
        .into_iter()
        .filter_map(|row| {
            let (sha, subject) = split_row(&row);
            sha.starts_with(needle)
                .then(|| described_pair(&sha, &subject))
        })
        .collect()
}

fn complete_path_pairs(cwd: &Path, word: &str) -> Vec<Pair> {
    let (dir_prefix, file_prefix) = split_path_prefix(word);
    let search_dir = resolve_completion_dir(cwd, &dir_prefix);
    let mut entries: Vec<_> = std::fs::read_dir(search_dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            if !file_name.starts_with(&file_prefix) {
                return None;
            }
            if !file_prefix.starts_with('.') && file_name.starts_with('.') {
                return None;
            }

            let metadata = entry.metadata().ok()?;
            let suffix = if metadata.is_dir() { "/" } else { "" };
            let replacement = format!("{}{}{}", dir_prefix, escape_shell_text(&file_name), suffix);
            let summary = if metadata.is_dir() {
                "directory"
            } else {
                "path"
            };
            Some(described_pair(&replacement, summary))
        })
        .collect();
    entries.sort_by(|left, right| left.replacement.cmp(&right.replacement));
    entries
}

fn git_rows(cwd: &Path, args: &[&str]) -> Vec<String> {
    let Ok(output) = Command::new("git").current_dir(cwd).args(args).output() else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn split_path_prefix(word: &str) -> (String, String) {
    if word == "~" {
        return ("~/".to_string(), String::new());
    }
    if let Some(index) = word.rfind('/') {
        return (word[..=index].to_string(), word[index + 1..].to_string());
    }
    (String::new(), word.to_string())
}

fn resolve_completion_dir(cwd: &Path, dir_prefix: &str) -> std::path::PathBuf {
    if dir_prefix.is_empty() {
        return cwd.to_path_buf();
    }
    if let Some(rest) = dir_prefix.strip_prefix("~/")
        && let Some(home) = std::env::var_os("HOME")
    {
        return Path::new(&home).join(rest);
    }
    let path = Path::new(dir_prefix);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

fn escape_shell_text(value: &str) -> String {
    value.replace(' ', "\\ ")
}

fn described_pair(replacement: &str, summary: &str) -> Pair {
    Pair {
        display: format!("{replacement}  {summary}"),
        replacement: replacement.to_string(),
    }
}

fn option_summary(option: &GitOptionSpec) -> String {
    match option.value {
        Some(kind) => format!("{} {}", option.summary, kind.placeholder()),
        None => option.summary.to_string(),
    }
}

fn dedupe_pairs(pairs: Vec<Pair>) -> Vec<Pair> {
    let mut seen = BTreeSet::new();
    pairs
        .into_iter()
        .filter(|pair| seen.insert(pair.replacement.clone()))
        .collect()
}

fn find_option<'a>(token: &str, options: &'a [GitOptionSpec]) -> Option<&'a GitOptionSpec> {
    let exact = options.iter().find(|option| option.names.contains(&token));
    if exact.is_some() {
        return exact;
    }
    options
        .iter()
        .find(|option| option.value.is_some() && option_consumes_inline_value(token, option))
}

fn option_consumes_inline_value(token: &str, option: &GitOptionSpec) -> bool {
    option.value.is_some()
        && option.names.iter().any(|name| {
            token
                .strip_prefix(&format!("{name}="))
                .is_some_and(|value| !value.is_empty())
                || (name.starts_with('-')
                    && !name.starts_with("--")
                    && token.starts_with(name)
                    && token.len() > name.len())
        })
}

fn split_row(row: &str) -> (String, String) {
    row.split_once('\t')
        .map(|(left, right)| (left.to_string(), right.to_string()))
        .unwrap_or_else(|| (row.to_string(), String::new()))
}

#[cfg(test)]
mod tests {
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
}
