mod render_rows;

use std::{
    collections::{BTreeMap, HashMap},
    fmt::Write as _,
};

use crate::commands::CommandLookup;

use self::render_rows::{
    render_alias_row, render_env_row, render_which_match_row, render_which_row,
    truncate_history_entry,
};
use super::common::{BLUE_BOLD, BOLD, CYAN_BOLD, badge, dim, paint, pluralize};

pub fn render_aliases(aliases: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} {}",
        paint(BLUE_BOLD, "alias"),
        dim(pluralize(aliases.len(), "alias", "aliases"))
    );
    if aliases.is_empty() {
        let _ = writeln!(out, "{}", dim("(empty)"));
        return out;
    }

    let _ = writeln!(
        out,
        "{}",
        dim("shell shortcuts expanded before command lookup")
    );
    for (name, value) in aliases {
        render_alias_row(&mut out, name, value);
    }
    out
}

pub fn render_lookup(command: &str, rows: &[(String, Option<CommandLookup>)]) -> String {
    let mut counts = [0usize; 4];
    for (_, result) in rows {
        match result {
            Some(CommandLookup::Alias(_)) => counts[0] += 1,
            Some(CommandLookup::Builtin) => counts[1] += 1,
            Some(CommandLookup::External(_)) => counts[2] += 1,
            None => counts[3] += 1,
        }
    }

    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} {}",
        paint(BLUE_BOLD, command),
        dim(pluralize(rows.len(), "target", "targets"))
    );
    let meta = [
        (counts[0], "alias", "aliases"),
        (counts[1], "builtin", "builtins"),
        (counts[2], "external command", "external commands"),
        (counts[3], "missing target", "missing targets"),
    ]
    .into_iter()
    .filter(|(count, _, _)| *count > 0)
    .map(|(count, singular, plural)| pluralize(count, singular, plural))
    .collect::<Vec<_>>();
    if !meta.is_empty() {
        let _ = writeln!(out, "{}", dim(meta.join(", ")));
    }
    for (name, result) in rows {
        render_which_row(&mut out, name, result.as_ref());
    }
    out
}

pub fn render_which(command: &str, rows: &[(String, Vec<CommandLookup>)]) -> String {
    let mut alias_count = 0usize;
    let mut builtin_count = 0usize;
    let mut external_count = 0usize;
    let mut missing_count = 0usize;

    for (_, matches) in rows {
        if matches.is_empty() {
            missing_count += 1;
            continue;
        }

        for result in matches {
            match result {
                CommandLookup::Alias(_) => alias_count += 1,
                CommandLookup::Builtin => builtin_count += 1,
                CommandLookup::External(_) => external_count += 1,
            }
        }
    }

    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} {}",
        paint(BLUE_BOLD, command),
        dim(pluralize(rows.len(), "target", "targets"))
    );

    let mut meta = Vec::new();
    if alias_count > 0 {
        meta.push(pluralize(alias_count, "alias", "aliases"));
    }
    if builtin_count > 0 {
        meta.push(pluralize(builtin_count, "builtin", "builtins"));
    }
    if external_count > 0 {
        meta.push(pluralize(
            external_count,
            "external command",
            "external commands",
        ));
    }
    if missing_count > 0 {
        meta.push(pluralize(
            missing_count,
            "missing target",
            "missing targets",
        ));
    }
    if !meta.is_empty() {
        let _ = writeln!(out, "{}", dim(meta.join(", ")));
    }
    let _ = writeln!(
        out,
        "{}",
        dim("current match follows alias, builtin, then PATH order")
    );

    for (name, matches) in rows {
        if matches.is_empty() {
            render_which_match_row(&mut out, name, None, false);
            continue;
        }

        for (index, result) in matches.iter().enumerate() {
            render_which_match_row(&mut out, name, Some(result), index == 0);
        }
    }

    out
}

pub fn render_env_map(env: &HashMap<String, String>) -> String {
    let mut entries = env.iter().collect::<Vec<_>>();
    entries.sort_by_key(|(name, _)| *name);

    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} {}",
        paint(BLUE_BOLD, "env"),
        dim(pluralize(entries.len(), "variable", "variables"))
    );
    if entries.is_empty() {
        let _ = writeln!(out, "{}", dim("(empty)"));
        return out;
    }
    for (name, value) in entries {
        render_env_row(&mut out, name, value);
    }
    out
}

pub fn render_history(entries: &[String], limit: Option<usize>) -> String {
    let shown = limit.unwrap_or(entries.len()).min(entries.len());
    let start = entries.len().saturating_sub(shown);

    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} {}",
        paint(BLUE_BOLD, "history"),
        dim(pluralize(entries.len(), "entry", "entries"))
    );
    let mut meta = vec![if shown < entries.len() {
        format!("showing latest {shown}")
    } else {
        format!("showing all {shown}")
    }];
    if let Some(last) = entries.last() {
        meta.push(format!("latest {}", truncate_history_entry(last, 48)));
    }
    let _ = writeln!(out, "{}", dim(meta.join(", ")));
    if shown == 0 {
        let _ = writeln!(out, "{}", dim("(empty)"));
        return out;
    }
    for (offset, entry) in entries[start..].iter().enumerate() {
        let index = start + offset + 1;
        let _ = writeln!(out, "{} {}", badge(index, CYAN_BOLD), paint(BOLD, entry));
    }
    out
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::render_env_map;

    #[test]
    fn render_env_map_marks_empty_state() {
        let rendered = render_env_map(&HashMap::new());
        assert!(rendered.contains("env"));
        assert!(rendered.contains("0 variables"));
        assert!(rendered.contains("(empty)"));
    }
}
