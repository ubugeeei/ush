use compact_str::CompactString;
use rustc_hash::FxHashSet;
use rustyline::completion::Pair;

pub(crate) fn candidate_pairs<I, S>(needle: &str, items: I) -> Vec<Pair>
where
    I: IntoIterator<Item = S>,
    S: Into<CompactString>,
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
        pairs.push(Pair {
            display: replacement.clone(),
            replacement,
        });
    }

    pairs
}
