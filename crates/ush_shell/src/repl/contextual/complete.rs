use std::path::Path;

use compact_str::CompactString;

use super::{
    candidates::candidate_pairs,
    catalog::{
        JUST_OPTIONS, JUST_OPTION_SPECS, MAKE_OPTIONS, MAKE_OPTION_SPECS, MISE_OPTION_SPECS,
        MISE_TASKS_SUBCOMMANDS, MISE_TOP_LEVEL, NPM_COMMANDS, NPM_OPTION_SPECS, VP_COMMANDS,
        VP_OPTIONS,
    },
    discover::{load_just_recipes, load_make_targets, load_mise_tasks, load_npm_scripts, load_vp_tasks},
    git::complete_git,
    options::{pending_value_kind, positional_args},
    types::{ContextualCompletion, Names, Tokens},
};
use crate::repl::syntax;

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
        "git" => complete_git(cwd, args, word),
        "make" | "gmake" => make_completion(cwd, args, word),
        "just" => just_completion(cwd, args, word),
        "mise" => mise_completion(cwd, args, word),
        "npm" => npm_completion(cwd, args, word),
        "vp" | "vite" => vp_completion(cwd, word),
        _ => None,
    }
}

fn current_segment_tokens(prefix: &str, word_start: usize) -> Tokens {
    let tokens = syntax::tokenize(&prefix[..word_start]);
    let start = tokens
        .iter()
        .rposition(|token| matches!(token.as_str(), "|" | "||" | "&&" | ";" | "&"))
        .map_or(0, |index| index + 1);

    tokens
        .into_iter()
        .skip(start)
        .map(CompactString::from)
        .collect()
}

fn find_command(tokens: &[CompactString]) -> Option<(usize, &str)> {
    for (index, token) in tokens.iter().enumerate() {
        if !syntax::is_assignment(token) && !matches!(token.as_str(), "!" | "command") {
            return Some((index, token.as_str()));
        }
    }
    None
}

fn make_completion(cwd: &Path, args: &[CompactString], word: &str) -> Option<ContextualCompletion> {
    let pending = pending_value_kind(args, MAKE_OPTION_SPECS);
    if matches!(pending, Some(true)) {
        return Some(ContextualCompletion::Path);
    }
    if pending.is_some() {
        return None;
    }
    let items = if word.starts_with('-') {
        candidate_pairs(word, MAKE_OPTIONS.iter().copied())
    } else {
        candidate_pairs(word, load_make_targets(cwd, args))
    };
    Some(ContextualCompletion::Pairs(items))
}

fn just_completion(cwd: &Path, args: &[CompactString], word: &str) -> Option<ContextualCompletion> {
    let pending = pending_value_kind(args, JUST_OPTION_SPECS);
    if matches!(pending, Some(true)) {
        return Some(ContextualCompletion::Path);
    }
    if pending.is_some() {
        return None;
    }
    let items = if word.starts_with('-') {
        candidate_pairs(word, JUST_OPTIONS.iter().copied())
    } else {
        candidate_pairs(word, load_just_recipes(cwd))
    };
    Some(ContextualCompletion::Pairs(items))
}

fn mise_completion(cwd: &Path, args: &[CompactString], word: &str) -> Option<ContextualCompletion> {
    let pending = pending_value_kind(args, MISE_OPTION_SPECS);
    if matches!(pending, Some(true)) {
        return Some(ContextualCompletion::Path);
    }
    if pending.is_some() {
        return None;
    }

    let positionals = positional_args(args, MISE_OPTION_SPECS);
    let tasks = load_mise_tasks(cwd);
    if positionals.is_empty() {
        if word.starts_with('-') {
            return None;
        }
        let mut items = Names::new();
        for item in MISE_TOP_LEVEL {
            items.push(CompactString::from(*item));
        }
        items.extend(tasks);
        return Some(ContextualCompletion::Pairs(candidate_pairs(word, items)));
    }

    let items = match positionals[0].as_str() {
        "run" | "r" | "watch" | "w" => tasks,
        "tasks" | "t" if positionals.len() == 1 => {
            return Some(ContextualCompletion::Pairs(candidate_pairs(
                word,
                MISE_TASKS_SUBCOMMANDS.iter().copied(),
            )));
        }
        "tasks" | "t" => match positionals[1].as_str() {
            "run" | "r" | "edit" | "info" => load_mise_tasks(cwd),
            _ => return None,
        },
        _ => load_mise_tasks(cwd),
    };

    Some(ContextualCompletion::Pairs(candidate_pairs(word, items)))
}

fn npm_completion(cwd: &Path, args: &[CompactString], word: &str) -> Option<ContextualCompletion> {
    let pending = pending_value_kind(args, NPM_OPTION_SPECS);
    if matches!(pending, Some(true)) {
        return Some(ContextualCompletion::Path);
    }
    if pending.is_some() {
        return None;
    }

    let positionals = positional_args(args, NPM_OPTION_SPECS);
    let items = if positionals.is_empty() {
        candidate_pairs(word, NPM_COMMANDS.iter().copied())
    } else {
        match positionals[0].as_str() {
            "run" | "run-script" | "rum" | "urn" => candidate_pairs(word, load_npm_scripts(cwd)),
            _ => return None,
        }
    };
    Some(ContextualCompletion::Pairs(items))
}

fn vp_completion(cwd: &Path, word: &str) -> Option<ContextualCompletion> {
    let items = if word.starts_with('-') {
        candidate_pairs(word, VP_OPTIONS.iter().copied())
    } else if load_vp_tasks(cwd).is_empty() {
        candidate_pairs(word, VP_COMMANDS.iter().copied())
    } else {
        candidate_pairs(word, load_vp_tasks(cwd))
    };
    Some(ContextualCompletion::Pairs(items))
}
