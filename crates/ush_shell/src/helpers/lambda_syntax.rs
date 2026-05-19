use anyhow::{Result, anyhow};

pub(super) fn parse_call(source: &str) -> Result<(&str, Vec<String>)> {
    let open = source
        .find('(')
        .ok_or_else(|| anyhow!("expected function call syntax"))?;
    let close = source
        .rfind(')')
        .ok_or_else(|| anyhow!("expected function call syntax"))?;
    Ok((
        source[..open].trim(),
        split_top_level(source[open + 1..close].trim()),
    ))
}

pub(super) fn parse_list_literal(source: &str) -> Result<Vec<String>> {
    let inner = source
        .trim()
        .strip_prefix('[')
        .and_then(|inner| inner.strip_suffix(']'))
        .ok_or_else(|| anyhow!("expected list literal syntax"))?
        .trim();
    Ok(split_top_level(inner))
}

pub(super) fn parse_string_arg(source: &str) -> Result<String> {
    parse_string_literal(source).ok_or_else(|| anyhow!("expected string literal, found {source}"))
}

pub(super) fn parse_string_literal(source: &str) -> Option<String> {
    (source.len() >= 2
        && ((source.starts_with('"') && source.ends_with('"'))
            || (source.starts_with('\'') && source.ends_with('\''))))
    .then(|| source[1..source.len() - 1].to_string())
}

pub(super) fn block_body(source: &str) -> &str {
    let trimmed = source.trim();
    trimmed
        .strip_prefix('{')
        .and_then(|inner| inner.strip_suffix('}'))
        .map(str::trim)
        .unwrap_or(trimmed)
}

fn split_top_level(source: &str) -> Vec<String> {
    if source.is_empty() {
        return Vec::new();
    }

    let mut args = Vec::new();
    let mut single = false;
    let mut double = false;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut start = 0usize;
    for (index, ch) in source.char_indices() {
        match ch {
            '\'' if !double => single = !single,
            '"' if !single => double = !double,
            '[' if !single && !double => bracket_depth += 1,
            ']' if !single && !double && bracket_depth > 0 => bracket_depth -= 1,
            '{' if !single && !double => brace_depth += 1,
            '}' if !single && !double && brace_depth > 0 => brace_depth -= 1,
            '(' if !single && !double => paren_depth += 1,
            ')' if !single && !double && paren_depth > 0 => paren_depth -= 1,
            ',' if !single
                && !double
                && bracket_depth == 0
                && brace_depth == 0
                && paren_depth == 0 =>
            {
                args.push(source[start..index].trim().to_string());
                start = index + 1;
            }
            _ => {}
        }
    }
    if !source.is_empty() {
        args.push(source[start..].trim().to_string());
    }
    args
}
