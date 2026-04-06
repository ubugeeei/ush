use std::collections::BTreeMap;

use anyhow::{Result, bail};

use super::is_assignment;

pub(super) fn expand_alias(stage: &str, aliases: &BTreeMap<String, String>) -> Result<String> {
    let mut current = stage.to_string();
    for _ in 0..8 {
        let Some(target) = alias_expansion_target(&current)? else {
            return Ok(current);
        };
        let Some(alias) = aliases.get(&target.word) else {
            return Ok(current);
        };
        current = format!(
            "{}{}{}",
            &current[..target.start],
            alias,
            &current[target.end..]
        );
    }
    Ok(current)
}

#[derive(Debug)]
struct AliasExpansionTarget {
    start: usize,
    end: usize,
    word: String,
}

#[derive(Clone, Copy, Debug)]
struct ShellWordSpan {
    start: usize,
    end: usize,
    quoted: bool,
}

fn alias_expansion_target(stage: &str) -> Result<Option<AliasExpansionTarget>> {
    let mut cursor = skip_whitespace(stage, 0);
    while let Some(span) = next_shell_word_span(stage, cursor)? {
        let raw = &stage[span.start..span.end];
        let word = parse_single_shell_word(raw)?;
        if is_assignment(&word) {
            cursor = skip_whitespace(stage, span.end);
            continue;
        }
        if span.quoted {
            return Ok(None);
        }
        return Ok(Some(AliasExpansionTarget {
            start: span.start,
            end: span.end,
            word,
        }));
    }
    Ok(None)
}

fn skip_whitespace(source: &str, from: usize) -> usize {
    source[from..]
        .char_indices()
        .find_map(|(offset, ch)| (!ch.is_whitespace()).then_some(from + offset))
        .unwrap_or(source.len())
}

fn next_shell_word_span(source: &str, from: usize) -> Result<Option<ShellWordSpan>> {
    let start = skip_whitespace(source, from);
    if start >= source.len() {
        return Ok(None);
    }

    let mut single = false;
    let mut double = false;
    let mut escaped = false;
    let mut quoted = false;

    for (offset, ch) in source[start..].char_indices() {
        let index = start + offset;
        match ch {
            '\\' if !single => {
                escaped = !escaped;
                quoted = true;
            }
            '\'' if !double && !escaped => {
                single = !single;
                escaped = false;
                quoted = true;
            }
            '"' if !single && !escaped => {
                double = !double;
                escaped = false;
                quoted = true;
            }
            _ if ch.is_whitespace() && !single && !double && !escaped => {
                return Ok(Some(ShellWordSpan {
                    start,
                    end: index,
                    quoted,
                }));
            }
            _ => escaped = false,
        }
    }

    if single || double {
        bail!("unterminated quoted string");
    }

    Ok(Some(ShellWordSpan {
        start,
        end: source.len(),
        quoted,
    }))
}

fn parse_single_shell_word(raw: &str) -> Result<String> {
    let mut tokens = shell_words::split(raw)?;
    match tokens.len() {
        1 => Ok(tokens.remove(0)),
        _ => bail!("invalid shell word: {raw}"),
    }
}
