mod path_pairs;

use std::collections::BTreeSet;

use rustyline::completion::Pair;

use crate::commands::BUILTIN_COMMANDS;

use super::UshHelper;
use super::specs::{ArgKind, BuiltinOptionSpec, BuiltinSpec, SIGNAL_CHOICES, command_summary};
use path_pairs::complete_path_pairs;

pub(super) fn complete_arg(
    helper: &UshHelper,
    start: usize,
    word: &str,
    kind: ArgKind,
) -> (usize, Vec<Pair>) {
    if kind.path_like() {
        return (start, complete_path_pairs(helper.cwd.as_path(), word));
    }

    (
        start,
        match kind {
            ArgKind::Alias => alias_pairs(helper, word),
            ArgKind::AliasAssignment => alias_assignment_pairs(word),
            ArgKind::Builtin => builtin_pairs(word, true),
            ArgKind::Choice(values) => choice_pairs(word, values),
            ArgKind::Command => command_name_pairs(helper, word),
            ArgKind::EnvAssignment => env_assignment_pairs(helper, word),
            ArgKind::EnvName => env_name_pairs(helper, word),
            ArgKind::Job => job_pairs(helper, word),
            ArgKind::Number | ArgKind::Port | ArgKind::Signal | ArgKind::Text => match kind {
                ArgKind::Signal => choice_pairs(word, SIGNAL_CHOICES),
                _ => Vec::new(),
            },
            ArgKind::Path => Vec::new(),
        },
    )
}

pub(super) fn command_name_pairs(helper: &UshHelper, needle: &str) -> Vec<Pair> {
    if needle.is_empty() {
        return Vec::new();
    }
    helper
        .commands
        .iter()
        .filter(|item| item.starts_with(needle))
        .map(|item| {
            let summary = command_summary(item)
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| "command".to_string());
            described_pair(item, &summary)
        })
        .collect()
}

pub(super) fn env_assignment_pairs(helper: &UshHelper, needle: &str) -> Vec<Pair> {
    if needle.is_empty() || needle.contains('=') {
        return Vec::new();
    }
    helper
        .env_names
        .iter()
        .filter(|name| name.starts_with(needle))
        .map(|name| described_pair(&format!("{name}="), "set environment variable"))
        .collect()
}

pub(super) fn option_pairs(prefix: &str, options: &[BuiltinOptionSpec]) -> Vec<Pair> {
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

pub(super) fn matching_options<'a>(
    prefix: &str,
    options: &'a [BuiltinOptionSpec],
) -> Vec<&'a BuiltinOptionSpec> {
    options
        .iter()
        .filter(|option| option.names.iter().any(|name| name.starts_with(prefix)))
        .collect()
}

pub(super) fn option_hint(options: Vec<&BuiltinOptionSpec>) -> Option<String> {
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

pub(super) fn command_hint(spec: &BuiltinSpec, expected: Option<ArgKind>) -> String {
    let mut hint = format!("  {}", spec.summary);
    if let Some(kind) = expected {
        hint.push_str("  next: ");
        hint.push_str(kind.placeholder());
    }
    if !spec.usage.is_empty() {
        hint.push_str("  usage: ");
        hint.push_str(spec.usage);
    }
    hint
}

pub(super) fn dedupe_pairs(pairs: Vec<Pair>) -> Vec<Pair> {
    let mut seen = BTreeSet::new();
    pairs
        .into_iter()
        .filter(|pair| seen.insert(pair.replacement.clone()))
        .collect()
}

fn builtin_pairs(needle: &str, include_empty: bool) -> Vec<Pair> {
    if needle.is_empty() && !include_empty {
        return Vec::new();
    }
    let mut seen = BTreeSet::new();
    BUILTIN_COMMANDS
        .iter()
        .copied()
        .filter(|name| name.starts_with(needle))
        .filter(|name| seen.insert(*name))
        .filter_map(|name| command_summary(name).map(|summary| described_pair(name, summary)))
        .collect()
}

fn alias_pairs(helper: &UshHelper, needle: &str) -> Vec<Pair> {
    if needle.is_empty() {
        return Vec::new();
    }
    helper
        .alias_names
        .iter()
        .filter(|name| name.starts_with(needle))
        .map(|name| described_pair(name, "alias"))
        .collect()
}

fn alias_assignment_pairs(needle: &str) -> Vec<Pair> {
    if needle.is_empty() || needle.contains('=') {
        return Vec::new();
    }
    vec![described_pair(&format!("{needle}="), "define alias")]
}

fn env_name_pairs(helper: &UshHelper, needle: &str) -> Vec<Pair> {
    if needle.is_empty() {
        return Vec::new();
    }
    helper
        .env_names
        .iter()
        .filter(|name| name.starts_with(needle))
        .map(|name| described_pair(name, "environment variable"))
        .collect()
}

fn job_pairs(helper: &UshHelper, needle: &str) -> Vec<Pair> {
    helper
        .jobs
        .iter()
        .filter(|job| needle.is_empty() || job.spec.starts_with(needle))
        .map(|job| described_pair(&job.spec, &job.summary))
        .collect()
}

fn choice_pairs(needle: &str, choices: &'static [&'static str]) -> Vec<Pair> {
    choices
        .iter()
        .copied()
        .filter(|choice| {
            choice.starts_with(&needle.to_ascii_lowercase()) || choice.starts_with(needle)
        })
        .map(|choice| described_pair(choice, "value"))
        .collect()
}

fn option_summary(option: &BuiltinOptionSpec) -> String {
    match option.value {
        Some(kind) => format!("{} {}", option.summary, kind.placeholder()),
        None => option.summary.to_string(),
    }
}

fn described_pair(replacement: &str, summary: &str) -> Pair {
    Pair {
        display: format!("{replacement}  {summary}"),
        replacement: replacement.to_string(),
    }
}
