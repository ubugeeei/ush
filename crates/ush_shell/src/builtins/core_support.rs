use anyhow::{Result, bail};

pub(super) fn with_trailing_newline(text: String) -> String {
    if text.is_empty() {
        text
    } else {
        format!("{text}\n")
    }
}

pub(super) fn parse_history_limit(args: &[String]) -> Result<Option<usize>> {
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
                limit = Some(parse_history_limit_value(
                    arg.split_once('=').expect("split").1,
                )?);
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

pub(super) fn render_history_plain(entries: &[String], limit: Option<usize>) -> String {
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

fn parse_history_limit_value(value: &str) -> Result<usize> {
    let limit = value.parse::<usize>()?;
    Ok(limit.max(1))
}
