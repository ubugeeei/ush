use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use rustyline::completion::Pair;
use serde_json::Value as JsonValue;
use toml::Value as TomlValue;

use super::syntax;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskEntry {
    pub source: &'static str,
    pub name: String,
    pub command: String,
}

pub(crate) enum ContextualCompletion {
    Pairs(Vec<Pair>),
    Path,
}

#[derive(Clone, Copy)]
struct OptionSpec {
    names: &'static [&'static str],
    values: usize,
    path_value: bool,
    short_inline_value: bool,
}

const GIT_GLOBAL_OPTIONS: &[&str] = &[
    "--help",
    "--version",
    "--paginate",
    "--no-pager",
    "-C",
    "-c",
    "--git-dir",
    "--work-tree",
    "--namespace",
];

const GIT_SUBCOMMANDS: &[&str] = &[
    "add",
    "am",
    "archive",
    "bisect",
    "branch",
    "bundle",
    "checkout",
    "cherry-pick",
    "clean",
    "clone",
    "commit",
    "config",
    "describe",
    "diff",
    "fetch",
    "format-patch",
    "grep",
    "help",
    "init",
    "log",
    "merge",
    "mv",
    "pull",
    "push",
    "rebase",
    "reflog",
    "remote",
    "reset",
    "restore",
    "revert",
    "rm",
    "show",
    "shortlog",
    "stash",
    "status",
    "submodule",
    "switch",
    "tag",
    "worktree",
];

const GIT_REMOTE_SUBCOMMANDS: &[&str] = &[
    "add",
    "get-url",
    "prune",
    "remove",
    "rename",
    "set-branches",
    "set-head",
    "set-url",
    "show",
    "update",
];

const MAKE_OPTIONS: &[&str] = &[
    "-B",
    "--always-make",
    "-C",
    "--directory",
    "-f",
    "--file",
    "--makefile",
    "-j",
    "--jobs",
    "-k",
    "--keep-going",
    "-n",
    "--dry-run",
    "-q",
    "--question",
    "-s",
    "--silent",
    "--warn-undefined-variables",
];

const JUST_OPTIONS: &[&str] = &[
    "--allow-missing",
    "--check",
    "--choose",
    "--command",
    "--dump",
    "--dump-format",
    "--edit",
    "--evaluate",
    "--fmt",
    "--global-justfile",
    "--groups",
    "--help",
    "--init",
    "--justfile",
    "--list",
    "--man",
    "--no-deps",
    "--quiet",
    "--set",
    "--show",
    "--summary",
    "--unsorted",
    "--usage",
    "--variables",
    "--working-directory",
    "-c",
    "-d",
    "-f",
    "-l",
    "-q",
    "-s",
    "-u",
    "-v",
];

const MISE_TOP_LEVEL: &[&str] = &[
    "activate",
    "backends",
    "bin-paths",
    "cache",
    "completion",
    "config",
    "doctor",
    "env",
    "exec",
    "fmt",
    "generate",
    "install",
    "latest",
    "lock",
    "ls",
    "outdated",
    "plugins",
    "prepare",
    "prune",
    "registry",
    "reshim",
    "run",
    "search",
    "set",
    "settings",
    "shell",
    "sync",
    "tasks",
    "trust",
    "uninstall",
    "upgrade",
    "use",
    "watch",
    "where",
    "which",
];

const MISE_TASKS_SUBCOMMANDS: &[&str] = &["add", "deps", "edit", "info", "ls", "run", "validate"];

const NPM_COMMANDS: &[&str] = &[
    "access",
    "audit",
    "cache",
    "ci",
    "completion",
    "config",
    "dedupe",
    "diff",
    "doctor",
    "exec",
    "help",
    "init",
    "install",
    "link",
    "ls",
    "outdated",
    "pack",
    "prefix",
    "publish",
    "query",
    "rebuild",
    "repo",
    "run",
    "start",
    "stop",
    "test",
    "uninstall",
    "update",
    "version",
    "view",
];

const VITE_COMMANDS: &[&str] = &["build", "dev", "optimize", "preview", "serve"];

const VITE_OPTIONS: &[&str] = &[
    "--base",
    "--clearScreen",
    "--config",
    "--debug",
    "--host",
    "--https",
    "--mode",
    "--open",
    "--port",
    "--strictPort",
];

const TASK_SOURCE_ORDER: &[&str] = &["make", "just", "mise", "npm", "vp"];

const MAKE_OPTION_SPECS: &[OptionSpec] = &[
    option_spec(&["-C", "--directory"], 1, true, false),
    option_spec(&["-f", "--file", "--makefile"], 1, true, true),
    option_spec(
        &["-I", "--include-dir", "-o", "--old-file", "--assume-old"],
        1,
        true,
        true,
    ),
    option_spec(
        &["-W", "--what-if", "--new-file", "--assume-new"],
        1,
        true,
        true,
    ),
    option_spec(
        &["-j", "--jobs", "-l", "--load-average", "--max-load"],
        1,
        false,
        true,
    ),
    option_spec(&["--debug", "-N", "--NeXT-option"], 1, false, true),
];

const JUST_OPTION_SPECS: &[OptionSpec] = &[
    option_spec(&["--chooser"], 1, false, false),
    option_spec(&["--command"], 1, false, false),
    option_spec(&["--dump-format"], 1, false, false),
    option_spec(&["--justfile", "-f"], 1, true, true),
    option_spec(&["--set"], 2, false, false),
    option_spec(&["--shell"], 1, false, false),
    option_spec(&["--shell-arg"], 1, false, false),
    option_spec(&["--show", "-s"], 1, false, false),
    option_spec(&["--tempdir"], 1, true, false),
    option_spec(&["--usage"], 1, false, false),
    option_spec(&["--working-directory", "-d"], 1, true, true),
];

const MISE_OPTION_SPECS: &[OptionSpec] = &[
    option_spec(&["-C", "--cd"], 1, true, false),
    option_spec(&["-E", "--env"], 1, false, false),
    option_spec(&["-j", "--jobs"], 1, false, false),
    option_spec(&["--output"], 1, false, false),
];

const NPM_OPTION_SPECS: &[OptionSpec] = &[
    option_spec(&["--workspace", "-w"], 1, false, false),
    option_spec(&["--prefix"], 1, true, false),
    option_spec(&["--userconfig"], 1, true, false),
];

const GIT_GLOBAL_OPTION_SPECS: &[OptionSpec] = &[
    option_spec(&["-C"], 1, true, true),
    option_spec(&["-c"], 1, false, false),
    option_spec(&["--git-dir", "--work-tree", "--namespace"], 1, true, false),
];

const fn option_spec(
    names: &'static [&'static str],
    values: usize,
    path_value: bool,
    short_inline_value: bool,
) -> OptionSpec {
    OptionSpec {
        names,
        values,
        path_value,
        short_inline_value,
    }
}

pub(crate) fn complete(
    cwd: &Path,
    prefix: &str,
    word_start: usize,
    word: &str,
) -> Option<ContextualCompletion> {
    let tokens = current_segment_tokens(prefix, word_start);
    let (command_index, command) = find_command(&tokens)?;
    let args = &tokens[command_index + 1..];

    match command {
        "git" => git_completion(cwd, args, word),
        "make" | "gmake" => make_completion(cwd, args, word),
        "just" => just_completion(cwd, args, word),
        "mise" => mise_completion(cwd, args, word),
        "npm" => npm_completion(cwd, args, word),
        "vp" | "vite" => Some(ContextualCompletion::Pairs(candidate_pairs(
            word,
            if word.starts_with('-') {
                VITE_OPTIONS.iter().copied()
            } else {
                VITE_COMMANDS.iter().copied()
            },
        ))),
        _ => None,
    }
}

pub(crate) fn discover_tasks(cwd: &Path) -> Vec<TaskEntry> {
    let mut entries = Vec::new();

    entries.extend(
        load_make_targets(cwd, &[])
            .into_iter()
            .map(|name| TaskEntry {
                source: "make",
                command: format!("make {name}"),
                name,
            }),
    );
    entries.extend(load_just_recipes(cwd).into_iter().map(|name| TaskEntry {
        source: "just",
        command: format!("just {name}"),
        name,
    }));
    entries.extend(load_mise_tasks(cwd).into_iter().map(|name| TaskEntry {
        source: "mise",
        command: format!("mise run {name}"),
        name,
    }));
    entries.extend(load_npm_scripts(cwd).into_iter().map(|name| TaskEntry {
        source: "npm",
        command: format!("npm run {name}"),
        name,
    }));
    entries.extend(load_vite_tasks(cwd).into_iter().map(|name| TaskEntry {
        source: "vp",
        command: format!("vp {name}"),
        name: name.to_string(),
    }));

    entries.sort_by(compare_task_entries);
    entries.dedup();
    entries
}

fn current_segment_tokens(prefix: &str, word_start: usize) -> Vec<String> {
    let tokens = syntax::tokenize(&prefix[..word_start]);
    let start = tokens
        .iter()
        .rposition(|token| matches!(token.as_str(), "|" | "||" | "&&" | ";" | "&"))
        .map_or(0, |index| index + 1);
    tokens[start..].to_vec()
}

fn find_command(tokens: &[String]) -> Option<(usize, &str)> {
    for (index, token) in tokens.iter().enumerate() {
        if syntax::is_assignment(token) || matches!(token.as_str(), "!" | "command") {
            continue;
        }
        return Some((index, token.as_str()));
    }
    None
}

fn git_completion(cwd: &Path, args: &[String], word: &str) -> Option<ContextualCompletion> {
    let pending = pending_value_kind(args, GIT_GLOBAL_OPTION_SPECS);
    if matches!(pending, Some(true)) {
        return Some(ContextualCompletion::Path);
    }
    if pending.is_some() {
        return None;
    }

    let Some((index, subcommand)) = find_git_subcommand(args) else {
        return Some(ContextualCompletion::Pairs(candidate_pairs(
            word,
            if word.starts_with('-') {
                GIT_GLOBAL_OPTIONS.iter().copied()
            } else {
                GIT_SUBCOMMANDS.iter().copied()
            },
        )));
    };

    let tail = &args[index + 1..];
    if word.starts_with('-') {
        let options = git_subcommand_options(subcommand);
        if !options.is_empty() {
            return Some(ContextualCompletion::Pairs(candidate_pairs(
                word,
                options.iter().copied(),
            )));
        }
    }

    match subcommand {
        "add" | "mv" | "rm" => Some(ContextualCompletion::Path),
        "branch" | "checkout" | "cherry-pick" | "log" | "merge" | "rebase" | "reset" | "revert"
        | "show" | "switch" | "tag" => git_ref_or_path_completion(cwd, tail, word),
        "restore" => {
            if tail
                .iter()
                .any(|arg| arg == "--source" || arg.starts_with("--source="))
            {
                return Some(ContextualCompletion::Pairs(candidate_pairs(
                    word,
                    git_refs(cwd),
                )));
            }
            Some(ContextualCompletion::Path)
        }
        "diff" => {
            if word_looks_like_path(word) || tail.iter().any(|arg| arg == "--") {
                Some(ContextualCompletion::Path)
            } else {
                Some(ContextualCompletion::Pairs(candidate_pairs(
                    word,
                    git_refs(cwd),
                )))
            }
        }
        "fetch" | "pull" | "push" => Some(ContextualCompletion::Pairs(git_remote_or_ref_pairs(
            cwd, tail, word,
        ))),
        "remote" => git_remote_completion(cwd, tail, word),
        "stash" => Some(ContextualCompletion::Pairs(candidate_pairs(
            word,
            [
                "apply", "branch", "clear", "drop", "list", "pop", "push", "show",
            ],
        ))),
        _ => None,
    }
}

fn make_completion(cwd: &Path, args: &[String], word: &str) -> Option<ContextualCompletion> {
    if matches!(pending_value_kind(args, MAKE_OPTION_SPECS), Some(true)) {
        return Some(ContextualCompletion::Path);
    }
    if pending_value_kind(args, MAKE_OPTION_SPECS).is_some() {
        return None;
    }

    if word.starts_with('-') {
        return Some(ContextualCompletion::Pairs(candidate_pairs(
            word,
            MAKE_OPTIONS.iter().copied(),
        )));
    }

    Some(ContextualCompletion::Pairs(candidate_pairs(
        word,
        load_make_targets(cwd, args),
    )))
}

fn just_completion(cwd: &Path, args: &[String], word: &str) -> Option<ContextualCompletion> {
    if matches!(pending_value_kind(args, JUST_OPTION_SPECS), Some(true)) {
        return Some(ContextualCompletion::Path);
    }
    if pending_value_kind(args, JUST_OPTION_SPECS).is_some() {
        return None;
    }

    if word.starts_with('-') {
        return Some(ContextualCompletion::Pairs(candidate_pairs(
            word,
            JUST_OPTIONS.iter().copied(),
        )));
    }

    Some(ContextualCompletion::Pairs(candidate_pairs(
        word,
        load_just_recipes(cwd),
    )))
}

fn mise_completion(cwd: &Path, args: &[String], word: &str) -> Option<ContextualCompletion> {
    if matches!(pending_value_kind(args, MISE_OPTION_SPECS), Some(true)) {
        return Some(ContextualCompletion::Path);
    }
    if pending_value_kind(args, MISE_OPTION_SPECS).is_some() {
        return None;
    }

    let positionals = positional_args(args, MISE_OPTION_SPECS);
    let tasks = load_mise_tasks(cwd);

    if positionals.is_empty() {
        if word.starts_with('-') {
            return None;
        }
        let mut items = MISE_TOP_LEVEL
            .iter()
            .map(|item| (*item).to_string())
            .collect::<Vec<_>>();
        items.extend(tasks);
        return Some(ContextualCompletion::Pairs(candidate_pairs(word, items)));
    }

    match positionals[0].as_str() {
        "run" | "r" | "watch" | "w" => Some(ContextualCompletion::Pairs(candidate_pairs(
            word,
            load_mise_tasks(cwd),
        ))),
        "tasks" | "t" => {
            if positionals.len() == 1 {
                return Some(ContextualCompletion::Pairs(candidate_pairs(
                    word,
                    MISE_TASKS_SUBCOMMANDS.iter().copied(),
                )));
            }
            match positionals[1].as_str() {
                "run" | "r" | "edit" | "info" => Some(ContextualCompletion::Pairs(
                    candidate_pairs(word, load_mise_tasks(cwd)),
                )),
                _ => None,
            }
        }
        _ => Some(ContextualCompletion::Pairs(candidate_pairs(word, tasks))),
    }
}

fn npm_completion(cwd: &Path, args: &[String], word: &str) -> Option<ContextualCompletion> {
    if matches!(pending_value_kind(args, NPM_OPTION_SPECS), Some(true)) {
        return Some(ContextualCompletion::Path);
    }
    if pending_value_kind(args, NPM_OPTION_SPECS).is_some() {
        return None;
    }

    let positionals = positional_args(args, NPM_OPTION_SPECS);
    if positionals.is_empty() {
        return Some(ContextualCompletion::Pairs(candidate_pairs(
            word,
            NPM_COMMANDS.iter().copied(),
        )));
    }

    match positionals[0].as_str() {
        "run" | "run-script" | "rum" | "urn" => Some(ContextualCompletion::Pairs(candidate_pairs(
            word,
            load_npm_scripts(cwd),
        ))),
        _ => None,
    }
}

fn git_ref_or_path_completion(
    cwd: &Path,
    tail: &[String],
    word: &str,
) -> Option<ContextualCompletion> {
    if tail.iter().any(|arg| arg == "--") || (!word.is_empty() && word_looks_like_path(word)) {
        return Some(ContextualCompletion::Path);
    }
    Some(ContextualCompletion::Pairs(candidate_pairs(
        word,
        git_refs(cwd),
    )))
}

fn git_remote_completion(cwd: &Path, tail: &[String], word: &str) -> Option<ContextualCompletion> {
    let positionals = positional_args(tail, &[]);
    if positionals.is_empty() {
        return Some(ContextualCompletion::Pairs(candidate_pairs(
            word,
            GIT_REMOTE_SUBCOMMANDS.iter().copied(),
        )));
    }

    match positionals[0].as_str() {
        "show" | "prune" | "remove" | "rename" | "set-head" | "set-url" => Some(
            ContextualCompletion::Pairs(candidate_pairs(word, git_remotes(cwd))),
        ),
        _ => None,
    }
}

fn git_remote_or_ref_pairs(cwd: &Path, tail: &[String], word: &str) -> Vec<Pair> {
    let positionals = positional_args(tail, &[]);
    if positionals.is_empty() {
        return candidate_pairs(word, git_remotes(cwd));
    }
    if positionals.len() == 1 {
        return candidate_pairs(word, git_refs(cwd));
    }
    Vec::new()
}

fn find_git_subcommand(args: &[String]) -> Option<(usize, &str)> {
    let mut pending = 0usize;
    for (index, arg) in args.iter().enumerate() {
        if pending > 0 {
            pending -= 1;
            continue;
        }
        if let Some((spec, inline)) = match_option(arg, GIT_GLOBAL_OPTION_SPECS) {
            if spec.values > 0 && !inline {
                pending = spec.values;
            }
            continue;
        }
        if arg.starts_with('-') {
            continue;
        }
        return Some((index, arg.as_str()));
    }
    None
}

fn git_subcommand_options(command: &str) -> &'static [&'static str] {
    match command {
        "add" => &["-A", "-u", "-p", "--all", "--patch", "--update"],
        "branch" => &["-a", "-d", "-D", "-m", "-M", "--all", "--list"],
        "checkout" => &[
            "-b", "-B", "--detach", "--ours", "--theirs", "--track", "--",
        ],
        "commit" => &["-a", "-m", "--amend", "--fixup", "--no-edit"],
        "diff" => &["--cached", "--name-only", "--staged", "--stat", "--"],
        "fetch" => &["--all", "--prune", "--tags"],
        "log" => &["--graph", "--name-only", "--oneline", "--stat"],
        "merge" => &["--abort", "--continue", "--ff-only", "--no-ff"],
        "pull" => &["--ff-only", "--rebase", "--tags"],
        "push" => &["-f", "-u", "--force", "--set-upstream", "--tags"],
        "rebase" => &["-i", "--abort", "--continue", "--onto"],
        "reset" => &["--hard", "--mixed", "--soft"],
        "restore" => &["--source", "--staged", "--worktree", "--"],
        "show" => &["--name-only", "--stat"],
        "status" => &["-b", "-s", "--branch", "--short"],
        "switch" => &["-c", "-C", "--detach", "--guess", "--track"],
        _ => &[],
    }
}

fn candidate_pairs<I, S>(needle: &str, items: I) -> Vec<Pair>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut unique = BTreeSet::new();
    for item in items {
        let item = item.as_ref();
        if needle.is_empty() || item.starts_with(needle) {
            unique.insert(item.to_string());
        }
    }
    unique
        .into_iter()
        .map(|item| Pair {
            display: item.clone(),
            replacement: item,
        })
        .collect()
}

fn compare_task_entries(left: &TaskEntry, right: &TaskEntry) -> std::cmp::Ordering {
    task_source_rank(left.source)
        .cmp(&task_source_rank(right.source))
        .then_with(|| left.name.cmp(&right.name))
        .then_with(|| left.command.cmp(&right.command))
}

fn task_source_rank(source: &str) -> usize {
    TASK_SOURCE_ORDER
        .iter()
        .position(|candidate| *candidate == source)
        .unwrap_or(TASK_SOURCE_ORDER.len())
}

fn word_looks_like_path(word: &str) -> bool {
    word.starts_with('.')
        || word.starts_with('/')
        || word.starts_with('~')
        || word.contains('/')
        || word.is_empty()
}

fn pending_value_kind(args: &[String], specs: &[OptionSpec]) -> Option<bool> {
    let mut pending: Option<(usize, bool)> = None;
    for arg in args {
        if let Some((remaining, path_value)) = pending {
            if remaining > 1 {
                pending = Some((remaining - 1, path_value));
            } else {
                pending = None;
            }
            continue;
        }

        if let Some((spec, inline)) = match_option(arg, specs) {
            if spec.values > 0 && !inline {
                pending = Some((spec.values, spec.path_value));
            }
        }
    }
    pending.map(|(_, path_value)| path_value)
}

fn positional_args(args: &[String], specs: &[OptionSpec]) -> Vec<String> {
    let mut values = Vec::new();
    let mut pending = 0usize;

    for arg in args {
        if pending > 0 {
            pending -= 1;
            continue;
        }
        if let Some((spec, inline)) = match_option(arg, specs) {
            if spec.values > 0 && !inline {
                pending = spec.values;
            }
            continue;
        }
        if arg.starts_with('-') {
            continue;
        }
        values.push(arg.clone());
    }

    values
}

fn match_option<'a>(arg: &str, specs: &'a [OptionSpec]) -> Option<(&'a OptionSpec, bool)> {
    for spec in specs {
        for name in spec.names {
            if arg == *name {
                return Some((spec, false));
            }
            if let Some(long) = name.strip_prefix("--") {
                let prefix = format!("--{long}=");
                if arg.starts_with(&prefix) {
                    return Some((spec, true));
                }
            }
            if spec.short_inline_value
                && name.starts_with('-')
                && !name.starts_with("--")
                && arg.starts_with(name)
                && arg.len() > name.len()
            {
                return Some((spec, true));
            }
        }
    }
    None
}

fn load_make_targets(cwd: &Path, args: &[String]) -> Vec<String> {
    let path = explicit_makefile_path(cwd, args).or_else(|| {
        ["GNUmakefile", "makefile", "Makefile"]
            .iter()
            .map(|name| cwd.join(name))
            .find(|path| path.is_file())
    });

    path.and_then(|path| fs::read_to_string(path).ok())
        .map(|source| parse_make_targets(&source))
        .unwrap_or_default()
}

fn explicit_makefile_path(cwd: &Path, args: &[String]) -> Option<PathBuf> {
    let mut path = None;
    let mut index = 0usize;
    while index < args.len() {
        let arg = args[index].as_str();
        match arg {
            "-f" | "--file" | "--makefile" => {
                if let Some(next) = args.get(index + 1) {
                    path = Some(resolve_relative(cwd, next));
                }
                index += 2;
                continue;
            }
            _ if arg.starts_with("--file=") => {
                path = Some(resolve_relative(cwd, &arg["--file=".len()..]));
            }
            _ if arg.starts_with("--makefile=") => {
                path = Some(resolve_relative(cwd, &arg["--makefile=".len()..]));
            }
            _ if arg.starts_with("-f") && arg.len() > 2 => {
                path = Some(resolve_relative(cwd, &arg[2..]));
            }
            _ => {}
        }
        index += 1;
    }
    path
}

fn parse_make_targets(source: &str) -> Vec<String> {
    let mut targets = BTreeSet::new();
    let mut logical = String::new();

    for raw in source.lines() {
        let line = raw.trim_end();
        if line.ends_with('\\') {
            logical.push_str(line.trim_end_matches('\\'));
            logical.push(' ');
            continue;
        }
        logical.push_str(line);
        let current = logical.trim().to_string();
        logical.clear();

        if current.is_empty() || current.starts_with('#') || raw.starts_with(char::is_whitespace) {
            continue;
        }
        if current.contains(":=")
            || current.contains("?=")
            || current.contains("+=")
            || current.contains("!=")
            || current.starts_with("define ")
        {
            continue;
        }

        let Some((head, _)) = current.split_once(':') else {
            continue;
        };
        if head.contains('=') {
            continue;
        }

        for name in head.split_whitespace() {
            if name.starts_with('.') || name.contains('%') || name.is_empty() {
                continue;
            }
            targets.insert(name.to_string());
        }
    }

    targets.into_iter().collect()
}

fn load_just_recipes(cwd: &Path) -> Vec<String> {
    find_upward(cwd, &["justfile", ".justfile", "Justfile"])
        .and_then(|path| fs::read_to_string(path).ok())
        .map(|source| parse_just_recipes(&source))
        .unwrap_or_default()
}

fn parse_just_recipes(source: &str) -> Vec<String> {
    let mut recipes = BTreeSet::new();

    for raw in source.lines() {
        if raw.starts_with(char::is_whitespace) {
            continue;
        }
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
            continue;
        }
        if line.starts_with("alias ")
            || line.starts_with("set ")
            || line.starts_with("export ")
            || line.starts_with("import ")
            || line.starts_with("mod ")
        {
            continue;
        }
        if line.contains(":=") {
            continue;
        }

        let Some((head, _)) = line.split_once(':') else {
            continue;
        };
        let Some(name) = head.split_whitespace().next() else {
            continue;
        };
        if name.chars().all(|ch| {
            ch == '_' || ch == '-' || ch == ':' || ch == '/' || ch.is_ascii_alphanumeric()
        }) {
            recipes.insert(name.to_string());
        }
    }

    recipes.into_iter().collect()
}

fn load_npm_scripts(cwd: &Path) -> Vec<String> {
    let Some(json) = load_package_json(cwd) else {
        return Vec::new();
    };
    let Some(scripts) = json.get("scripts").and_then(JsonValue::as_object) else {
        return Vec::new();
    };

    let mut names = scripts.keys().cloned().collect::<Vec<_>>();
    names.sort();
    names
}

fn load_vite_tasks(cwd: &Path) -> Vec<&'static str> {
    if has_vite_project(cwd) {
        VITE_COMMANDS.to_vec()
    } else {
        Vec::new()
    }
}

fn load_mise_tasks(cwd: &Path) -> Vec<String> {
    let mut tasks = BTreeSet::new();
    let mut ancestors = cwd.ancestors().collect::<Vec<_>>();
    ancestors.reverse();

    for dir in ancestors {
        for file in ["mise.toml", ".mise.toml"] {
            let path = dir.join(file);
            if path.is_file() {
                load_mise_toml_tasks(&path, &mut tasks);
            }
        }

        for task_dir in [
            "mise-tasks",
            ".mise-tasks",
            ".mise/tasks",
            "mise/tasks",
            ".config/mise/tasks",
        ] {
            let task_root = dir.join(task_dir);
            collect_mise_task_scripts(&task_root, &task_root, &mut tasks);
        }
    }

    tasks.into_iter().collect()
}

fn load_mise_toml_tasks(path: &Path, tasks: &mut BTreeSet<String>) {
    let Ok(source) = fs::read_to_string(path) else {
        return;
    };
    let Ok(toml) = toml::from_str::<TomlValue>(&source) else {
        return;
    };
    let Some(table) = toml.get("tasks").and_then(TomlValue::as_table) else {
        return;
    };
    for (name, value) in table {
        if value.is_table() || value.is_str() || value.is_array() {
            tasks.insert(name.to_string());
        }
    }
}

fn collect_mise_task_scripts(base: &Path, dir: &Path, tasks: &mut BTreeSet<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_mise_task_scripts(base, &path, tasks);
            continue;
        }
        let Ok(relative) = path.strip_prefix(base) else {
            continue;
        };
        let name = relative.to_string_lossy().replace('\\', "/");
        if !name.is_empty() {
            tasks.insert(name);
        }
    }
}

fn git_refs(cwd: &Path) -> Vec<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args([
            "for-each-ref",
            "--format=%(refname:short)",
            "refs/heads",
            "refs/remotes",
            "refs/tags",
        ])
        .output();
    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    let mut refs = BTreeSet::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if line.ends_with("/HEAD") || line.is_empty() {
            continue;
        }
        refs.insert(line.to_string());
    }
    refs.into_iter().collect()
}

fn git_remotes(cwd: &Path) -> Vec<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .arg("remote")
        .output();
    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    let mut remotes = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    remotes.sort();
    remotes
}

fn load_package_json(cwd: &Path) -> Option<JsonValue> {
    let path = find_upward(cwd, &["package.json"])?;
    let source = fs::read_to_string(path).ok()?;
    serde_json::from_str(&source).ok()
}

fn has_vite_project(cwd: &Path) -> bool {
    if find_upward(
        cwd,
        &[
            "vite.config.js",
            "vite.config.cjs",
            "vite.config.mjs",
            "vite.config.ts",
            "vite.config.cts",
            "vite.config.mts",
        ],
    )
    .is_some()
    {
        return true;
    }

    let Some(json) = load_package_json(cwd) else {
        return false;
    };

    if package_json_has_name(&json, "vite") {
        return true;
    }

    json.get("scripts")
        .and_then(JsonValue::as_object)
        .into_iter()
        .flat_map(|scripts| scripts.values())
        .filter_map(JsonValue::as_str)
        .any(|value| matches!(value.split_whitespace().next(), Some("vite" | "vp")))
}

fn package_json_has_name(json: &JsonValue, name: &str) -> bool {
    [
        "dependencies",
        "devDependencies",
        "optionalDependencies",
        "peerDependencies",
    ]
    .into_iter()
    .filter_map(|field| json.get(field).and_then(JsonValue::as_object))
    .any(|table| table.contains_key(name))
}

fn find_upward(cwd: &Path, names: &[&str]) -> Option<PathBuf> {
    for dir in cwd.ancestors() {
        for name in names {
            let path = dir.join(name);
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

fn resolve_relative(cwd: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path, process::Command};

    use rustyline::{Context, completion::Completer, history::DefaultHistory};
    use tempfile::tempdir;

    use crate::repl::UshHelper;

    fn helper(cwd: &Path) -> UshHelper {
        UshHelper::new(
            vec![
                "git".to_string(),
                "just".to_string(),
                "make".to_string(),
                "mise".to_string(),
                "npm".to_string(),
                "vp".to_string(),
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

    #[test]
    fn discovers_tasks_across_supported_sources() {
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
        fs::write(
            dir.path().join("package.json"),
            r#"{"scripts":{"build":"vite build"},"devDependencies":{"vite":"^7.0.0"}}"#,
        )
        .expect("write package");

        let entries = super::discover_tasks(dir.path());

        assert!(entries.iter().any(|entry| entry.command == "make build"));
        assert!(entries.iter().any(|entry| entry.command == "just fmt"));
        assert!(entries.iter().any(|entry| entry.command == "mise run lint"));
        assert!(entries.iter().any(|entry| entry.command == "npm run build"));
        assert!(entries.iter().any(|entry| entry.command == "vp dev"));
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
}
