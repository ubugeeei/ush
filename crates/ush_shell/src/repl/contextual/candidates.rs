use compact_str::CompactString;
use rustc_hash::FxHashSet;
use rustyline::completion::Pair;

use crate::repl::display;

pub(crate) fn typed_candidate_pairs<I, S>(needle: &str, items: I, detail: &'static str) -> Vec<Pair>
where
    I: IntoIterator<Item = S>,
    S: Into<CompactString>,
{
    described_candidate_pairs(needle, items, |_| Some(detail))
}

pub(crate) fn described_candidate_pairs<I, S, F>(needle: &str, items: I, describe: F) -> Vec<Pair>
where
    I: IntoIterator<Item = S>,
    S: Into<CompactString>,
    F: Fn(&str) -> Option<&'static str>,
{
    let mut seen = FxHashSet::default();
    let mut pairs = Vec::new();

    for item in items {
        let item: CompactString = item.into();
        if !needle.is_empty() && !item.starts_with(needle) {
            continue;
        }
        if !seen.insert(item.clone()) {
            continue;
        }
        let replacement = item.to_string();
        pairs.push(display::same_pair(replacement, describe(item.as_str())));
    }

    pairs
}
