use std::{path::Path, process::Command};

use compact_str::CompactString;
use rustc_hash::FxHashSet;

use super::{
    candidates::candidate_pairs,
    catalog::{
        GIT_GLOBAL_OPTIONS, GIT_GLOBAL_OPTION_SPECS, GIT_REMOTE_SUBCOMMANDS, GIT_SUBCOMMAND_OPTIONS,
        GIT_SUBCOMMANDS,
    },
    options::{match_option, pending_value_kind, positional_args},
    types::{ContextualCompletion, Names},
};

pub(crate) fn complete_git(
    cwd: &Path,
    args: &[CompactString],
    word: &str,
) -> Option<ContextualCompletion> {
    let pending = pending_value_kind(args, GIT_GLOBAL_OPTION_SPECS);
    if matches!(pending, Some(true)) {
        return Some(ContextualCompletion::Path);
    }
    if pending.is_some() {
        return None;
    }

    let Some((index, subcommand)) = find_git_subcommand(args) else {
        let items = if word.starts_with('-') {
            GIT_GLOBAL_OPTIONS.iter().copied()
        } else {
            GIT_SUBCOMMANDS.iter().copied()
        };
        return Some(ContextualCompletion::Pairs(candidate_pairs(word, items)));
    };

    let tail = &args[index + 1..];
    if word.starts_with('-')
        && let Some(options) = GIT_SUBCOMMAND_OPTIONS.get(subcommand)
    {
        return Some(ContextualCompletion::Pairs(candidate_pairs(word, options.iter().copied())));
    }

    match subcommand {
        "add" | "mv" | "rm" => Some(ContextualCompletion::Path),
        "branch" | "checkout" | "cherry-pick" | "log" | "merge" | "rebase" | "reset"
        | "revert" | "show" | "switch" | "tag" => git_ref_or_path_completion(cwd, tail, word),
        "restore" => {
            if tail
                .iter()
                .any(|arg| arg == "--source" || arg.starts_with("--source="))
            {
                return Some(ContextualCompletion::Pairs(candidate_pairs(word, git_refs(cwd))));
            }
            Some(ContextualCompletion::Path)
        }
        "diff" => {
            if word_looks_like_path(word) || tail.iter().any(|arg| arg == "--") {
                Some(ContextualCompletion::Path)
            } else {
                Some(ContextualCompletion::Pairs(candidate_pairs(word, git_refs(cwd))))
            }
        }
        "fetch" | "pull" | "push" => {
            Some(ContextualCompletion::Pairs(git_remote_or_ref_pairs(cwd, tail, word)))
        }
        "remote" => git_remote_completion(cwd, tail, word),
        "stash" => Some(ContextualCompletion::Pairs(candidate_pairs(
            word,
            ["apply", "branch", "clear", "drop", "list", "pop", "push", "show"],
        ))),
        _ => None,
    }
}

fn find_git_subcommand(args: &[CompactString]) -> Option<(usize, &str)> {
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
        if !arg.starts_with('-') {
            return Some((index, arg.as_str()));
        }
    }

    None
}

fn git_ref_or_path_completion(
    cwd: &Path,
    tail: &[CompactString],
    word: &str,
) -> Option<ContextualCompletion> {
    if tail.iter().any(|arg| arg == "--") || (!word.is_empty() && word_looks_like_path(word)) {
        return Some(ContextualCompletion::Path);
    }
    Some(ContextualCompletion::Pairs(candidate_pairs(word, git_refs(cwd))))
}

fn git_remote_completion(
    cwd: &Path,
    tail: &[CompactString],
    word: &str,
) -> Option<ContextualCompletion> {
    let positionals = positional_args(tail, &[]);
    if positionals.is_empty() {
        return Some(ContextualCompletion::Pairs(candidate_pairs(
            word,
            GIT_REMOTE_SUBCOMMANDS.iter().copied(),
        )));
    }

    match positionals[0].as_str() {
        "show" | "prune" | "remove" | "rename" | "set-head" | "set-url" => {
            Some(ContextualCompletion::Pairs(candidate_pairs(word, git_remotes(cwd))))
        }
        _ => None,
    }
}

fn git_remote_or_ref_pairs(cwd: &Path, tail: &[CompactString], word: &str) -> Vec<rustyline::completion::Pair> {
    let positionals = positional_args(tail, &[]);
    if positionals.is_empty() {
        return candidate_pairs(word, git_remotes(cwd));
    }
    if positionals.len() == 1 {
        return candidate_pairs(word, git_refs(cwd));
    }
    Vec::new()
}

fn git_refs(cwd: &Path) -> Names {
    git_lines(
        cwd,
        &[
            "for-each-ref",
            "--format=%(refname:short)",
            "refs/heads",
            "refs/remotes",
            "refs/tags",
        ],
        |line| !line.is_empty() && !line.ends_with("/HEAD"),
    )
}

fn git_remotes(cwd: &Path) -> Names {
    git_lines(cwd, &["remote"], |line| !line.is_empty())
}

fn git_lines(cwd: &Path, args: &[&str], keep: impl Fn(&str) -> bool) -> Names {
    let Ok(output) = Command::new("git").arg("-C").arg(cwd).args(args).output() else {
        return Names::new();
    };
    if !output.status.success() {
        return Names::new();
    }

    let mut seen = FxHashSet::default();
    let mut names = Names::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if keep(line) {
            let line = CompactString::from(line);
            if seen.insert(line.clone()) {
                names.push(line);
            }
        }
    }
    names.sort_unstable();
    names
}

fn word_looks_like_path(word: &str) -> bool {
    word.starts_with('.')
        || word.starts_with('/')
        || word.starts_with('~')
        || word.contains('/')
        || word.is_empty()
}
