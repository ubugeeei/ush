use std::fmt::Write as _;

use crate::commands::CommandLookup;

use super::super::common::{
    BLUE_BOLD, BOLD, CYAN_BOLD, GREEN_BOLD, MAGENTA_BOLD, RED_BOLD, YELLOW_BOLD, badge, dim, paint,
};

pub(super) fn render_alias_row(out: &mut String, name: &str, value: &str) {
    let _ = writeln!(
        out,
        "{} {} {}",
        paint(CYAN_BOLD, name),
        badge("alias", BLUE_BOLD),
        paint(BOLD, value)
    );
}

pub(super) fn render_which_row(out: &mut String, name: &str, result: Option<&CommandLookup>) {
    let line = match result {
        Some(CommandLookup::Alias(value)) => format!(
            "{} {} {}",
            paint(CYAN_BOLD, name),
            badge("alias", BLUE_BOLD),
            paint(BOLD, value)
        ),
        Some(CommandLookup::Builtin) => format!(
            "{} {} {}",
            paint(CYAN_BOLD, name),
            badge("builtin", YELLOW_BOLD),
            dim("shell builtin")
        ),
        Some(CommandLookup::External(path)) => format!(
            "{} {} {}",
            paint(CYAN_BOLD, name),
            badge("external", GREEN_BOLD),
            dim(path.display())
        ),
        None => format!(
            "{} {} {}",
            paint(CYAN_BOLD, name),
            badge("not found", RED_BOLD),
            dim("unavailable on PATH")
        ),
    };
    let _ = writeln!(out, "{line}");
}

pub(super) fn render_which_match_row(
    out: &mut String,
    name: &str,
    result: Option<&CommandLookup>,
    current: bool,
) {
    let current_badge = current.then(|| badge("current", MAGENTA_BOLD));

    match result {
        Some(CommandLookup::Alias(value)) => {
            let mut parts = vec![paint(CYAN_BOLD, name)];
            if let Some(current_badge) = current_badge.as_ref() {
                parts.push(current_badge.clone());
            }
            parts.push(badge("alias", BLUE_BOLD));
            parts.push(paint(BOLD, value));
            let _ = writeln!(out, "{}", parts.join(" "));
        }
        Some(CommandLookup::Builtin) => {
            let mut parts = vec![paint(CYAN_BOLD, name)];
            if let Some(current_badge) = current_badge.as_ref() {
                parts.push(current_badge.clone());
            }
            parts.push(badge("builtin", YELLOW_BOLD));
            parts.push(dim("shell builtin"));
            let _ = writeln!(out, "{}", parts.join(" "));
        }
        Some(CommandLookup::External(path)) => {
            let mut parts = vec![paint(CYAN_BOLD, name)];
            if let Some(current_badge) = current_badge.as_ref() {
                parts.push(current_badge.clone());
            }
            parts.push(badge("external", GREEN_BOLD));
            parts.push(dim(path.display()));
            let _ = writeln!(out, "{}", parts.join(" "));
        }
        None => {
            let _ = writeln!(
                out,
                "{} {} {}",
                paint(CYAN_BOLD, name),
                badge("not found", RED_BOLD),
                dim("unavailable on PATH")
            );
        }
    }
}

pub(super) fn render_env_row(out: &mut String, name: &str, value: &str) {
    let display_value = if value.is_empty() {
        dim("(empty)")
    } else {
        paint(BOLD, value)
    };
    let _ = writeln!(
        out,
        "{} {} {}",
        paint(CYAN_BOLD, name),
        dim("="),
        display_value
    );
}

pub(super) fn truncate_history_entry(entry: &str, max_chars: usize) -> String {
    if entry.chars().count() <= max_chars {
        return format!("`{entry}`");
    }
    let truncated = entry
        .chars()
        .take(max_chars.saturating_sub(1))
        .collect::<String>();
    format!("`{}...`", truncated.trim_end())
}
