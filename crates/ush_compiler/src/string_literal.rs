use crate::scan::starts_triple_quote;
use crate::types::{AstString as String, AstVec as Vec};

pub(crate) fn parse_string_literal(source: &str) -> Option<String> {
    if source.len() >= 6
        && starts_triple_quote(source, 0)
        && starts_triple_quote(source, source.len() - 3)
    {
        return Some(parse_multiline_string(&source[3..source.len() - 3]));
    }
    if source.len() < 2 {
        return None;
    }
    let bytes = source.as_bytes();
    let quote = *bytes.first()?;
    if quote != b'\'' && quote != b'"' {
        return None;
    }
    if bytes.last().copied()? != quote {
        return None;
    }
    Some(source[1..source.len() - 1].into())
}

fn parse_multiline_string(body: &str) -> String {
    let body = body.strip_prefix('\n').unwrap_or(body);
    let mut lines = body.split('\n').collect::<Vec<_>>();
    if lines.last().is_some_and(|line| line.trim().is_empty()) {
        lines.pop();
    }
    let indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.chars()
                .take_while(|ch| matches!(ch, ' ' | '\t'))
                .count()
        })
        .min()
        .unwrap_or(0);
    let mut out = String::default();
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(strip_indent(line, indent));
    }
    out
}

fn strip_indent(line: &str, indent: usize) -> &str {
    let mut offset = 0usize;
    let mut removed = 0usize;
    for ch in line.chars() {
        if removed == indent || !matches!(ch, ' ' | '\t') {
            break;
        }
        removed += 1;
        offset += ch.len_utf8();
    }
    &line[offset..]
}
