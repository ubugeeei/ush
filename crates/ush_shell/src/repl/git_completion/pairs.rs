mod path_pairs;
mod ref_pairs;

use std::collections::BTreeSet;

use rustyline::completion::Pair;

use crate::repl::UshHelper;

use super::specs::{ArgKind, GIT_COMMANDS, GitCommandSpec, GitOptionSpec};
use path_pairs::complete_path_pairs;
use ref_pairs::{
    branch_pairs, commit_pairs, local_branch_pairs, ref_pairs, remote_branch_pairs, remote_pairs,
    stash_pairs, tag_pairs,
};

pub(super) fn complete_arg(
    helper: &UshHelper,
    start: usize,
    word: &str,
    kind: ArgKind,
) -> (usize, Vec<Pair>) {
    if kind.path_like() {
        return (start, complete_path_pairs(helper.cwd(), word));
    }

    let pairs = match kind {
        ArgKind::Branch => dedupe_pairs(branch_pairs(helper.cwd(), word)),
        ArgKind::Commit => dedupe_pairs(commit_pairs(helper.cwd(), word)),
        ArgKind::Config | ArgKind::Message | ArgKind::Number | ArgKind::Url => Vec::new(),
        ArgKind::LocalBranch => local_branch_pairs(helper.cwd(), word),
        ArgKind::Path | ArgKind::Pathspec => Vec::new(),
        ArgKind::Ref => dedupe_pairs(ref_pairs(helper.cwd(), word)),
        ArgKind::Remote => remote_pairs(helper.cwd(), word),
        ArgKind::RemoteBranch => remote_branch_pairs(helper.cwd(), word),
        ArgKind::Stash => stash_pairs(helper.cwd(), word),
        ArgKind::Tag => tag_pairs(helper.cwd(), word),
    };
    (start, pairs)
}

pub(super) fn command_pairs(prefix: &str) -> Vec<Pair> {
    GIT_COMMANDS
        .iter()
        .filter(|spec| spec.name.starts_with(prefix))
        .map(|spec| described_pair(spec.name, spec.summary))
        .collect()
}

pub(super) fn matching_commands(prefix: &str) -> Vec<&'static GitCommandSpec> {
    GIT_COMMANDS
        .iter()
        .filter(|spec| spec.name.starts_with(prefix))
        .collect()
}

pub(super) fn option_pairs(prefix: &str, options: &[GitOptionSpec]) -> Vec<Pair> {
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
    options: &'a [GitOptionSpec],
) -> Vec<&'a GitOptionSpec> {
    options
        .iter()
        .filter(|option| option.names.iter().any(|name| name.starts_with(prefix)))
        .collect()
}

pub(super) fn option_hint(options: Vec<&GitOptionSpec>) -> Option<String> {
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

pub(super) fn command_hint(spec: &GitCommandSpec, expected: Option<ArgKind>) -> String {
    let mut hint = format!("  {}", spec.summary);
    if let Some(kind) = expected {
        hint.push_str("  next: ");
        hint.push_str(kind.placeholder());
    }
    hint.push_str("  usage: ");
    hint.push_str(spec.usage);
    hint
}

pub(super) fn dedupe_pairs(pairs: Vec<Pair>) -> Vec<Pair> {
    let mut seen = BTreeSet::new();
    pairs
        .into_iter()
        .filter(|pair| seen.insert(pair.replacement.clone()))
        .collect()
}

pub(super) fn described_pair(replacement: &str, summary: &str) -> Pair {
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
