use std::fs;

use anyhow::{Result, bail};

use crate::{Shell, ValueStream, style};

use super::with_trailing_newline;

impl Shell {
    pub(in crate::builtins) fn handle_history(
        &self,
        args: &[String],
    ) -> Result<(ValueStream, i32)> {
        let limit = parse_history_limit(args)?;
        let entries = self.read_history_entries();

        if self.options.stylish {
            return Ok((ValueStream::Text(style::render_history(&entries, limit)), 0));
        }

        let text = render_history_plain(&entries, limit);
        Ok((ValueStream::Text(text), 0))
    }

    pub(super) fn read_history(&self) -> String {
        fs::read_to_string(&self.paths.history_file).unwrap_or_default()
    }

    pub(super) fn read_history_entries(&self) -> Vec<String> {
        self.read_history()
            .lines()
            .map(ToString::to_string)
            .collect()
    }
}

fn parse_history_limit(args: &[String]) -> Result<Option<usize>> {
    let mut pending_limit = false;
    let mut limit = None;

    for arg in args {
        if pending_limit {
            limit = Some(parse_history_limit_value(arg)?);
            pending_limit = false;
            continue;
        }

        match arg.as_str() {
            "--limit" => pending_limit = true,
            _ if arg.starts_with("--limit=") => {
                limit = Some(parse_history_limit_value(arg.split_once('=').unwrap().1)?);
            }
            _ if arg.chars().all(|ch| ch.is_ascii_digit()) => {
                limit = Some(parse_history_limit_value(arg)?);
            }
            _ => bail!("history accepts only a numeric limit or --limit N"),
        }
    }

    if pending_limit {
        bail!("history --limit requires a value");
    }

    Ok(limit)
}

fn parse_history_limit_value(value: &str) -> Result<usize> {
    let limit = value.parse::<usize>()?;
    Ok(limit.max(1))
}

fn render_history_plain(entries: &[String], limit: Option<usize>) -> String {
    let start = entries
        .len()
        .saturating_sub(limit.unwrap_or(entries.len()).min(entries.len()));
    let text = entries[start..]
        .iter()
        .enumerate()
        .map(|(offset, entry)| format!("{}\t{}", start + offset + 1, entry))
        .collect::<Vec<_>>()
        .join("\n");
    with_trailing_newline(text)
}
