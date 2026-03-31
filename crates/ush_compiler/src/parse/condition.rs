use anyhow::{Result, bail};

use super::{
    super::ast::Condition,
    expr::{parse_expr, parse_pattern},
};
use crate::types::HeapVec as Vec;

pub(super) fn parse_condition(source: &str) -> Result<Condition> {
    let source = source.trim();
    if let Some(parts) = split_logic(source, "||") {
        return Ok(Condition::Or(
            parts
                .into_iter()
                .map(parse_condition)
                .collect::<Result<Vec<_>>>()?,
        ));
    }
    if let Some(parts) = split_logic(source, "&&") {
        return Ok(Condition::And(
            parts
                .into_iter()
                .map(parse_condition)
                .collect::<Result<Vec<_>>>()?,
        ));
    }
    if let Some(rest) = source.strip_prefix("let ") {
        return parse_binding_condition(rest);
    }
    if let Some(condition) = parse_optional_binding(source)? {
        return Ok(condition);
    }
    Ok(Condition::Expr(parse_expr(source)?))
}

fn parse_binding_condition(source: &str) -> Result<Condition> {
    let Some((pattern, expr)) = split_binding(source) else {
        bail!("expected `let pattern = expr`");
    };
    Ok(Condition::Let {
        pattern: parse_pattern(pattern)?,
        expr: parse_expr(expr)?,
    })
}

fn parse_optional_binding(source: &str) -> Result<Option<Condition>> {
    let Some((pattern, expr)) = split_binding(source) else {
        return Ok(None);
    };
    Ok(Some(Condition::Let {
        pattern: parse_pattern(pattern)?,
        expr: parse_expr(expr)?,
    }))
}

fn split_logic<'a>(source: &'a str, token: &str) -> Option<Vec<&'a str>> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut single = false;
    let mut double = false;
    let mut paren = 0usize;
    let mut brace = 0usize;
    let mut bracket = 0usize;
    let bytes = source.as_bytes();

    let mut index = 0usize;
    while index + token.len() <= bytes.len() {
        match bytes[index] {
            b'\'' if !double => single = !single,
            b'"' if !single => double = !double,
            b'(' if !single && !double => paren += 1,
            b')' if !single && !double && paren > 0 => paren -= 1,
            b'{' if !single && !double => brace += 1,
            b'}' if !single && !double && brace > 0 => brace -= 1,
            b'[' if !single && !double => bracket += 1,
            b']' if !single && !double && bracket > 0 => bracket -= 1,
            _ => {}
        }
        if !single
            && !double
            && paren == 0
            && brace == 0
            && bracket == 0
            && source[index..].starts_with(token)
        {
            parts.push(source[start..index].trim());
            start = index + token.len();
            index += token.len();
            continue;
        }
        index += 1;
    }
    if parts.is_empty() {
        return None;
    }
    parts.push(source[start..].trim());
    Some(parts)
}

fn split_binding(source: &str) -> Option<(&str, &str)> {
    let mut single = false;
    let mut double = false;
    let mut paren = 0usize;
    let mut brace = 0usize;
    let mut bracket = 0usize;
    let bytes = source.as_bytes();

    for index in 0..bytes.len() {
        match bytes[index] {
            b'\'' if !double => single = !single,
            b'"' if !single => double = !double,
            b'(' if !single && !double => paren += 1,
            b')' if !single && !double && paren > 0 => paren -= 1,
            b'{' if !single && !double => brace += 1,
            b'}' if !single && !double && brace > 0 => brace -= 1,
            b'[' if !single && !double => bracket += 1,
            b']' if !single && !double && bracket > 0 => bracket -= 1,
            b'=' if !single && !double && paren == 0 && brace == 0 && bracket == 0 => {
                let prev = index.checked_sub(1).and_then(|idx| bytes.get(idx)).copied();
                let next = bytes.get(index + 1).copied();
                if matches!(prev, Some(b'=') | Some(b'!') | Some(b'<') | Some(b'>'))
                    || matches!(next, Some(b'='))
                {
                    continue;
                }
                return Some((source[..index].trim(), source[index + 1..].trim()));
            }
            _ => {}
        }
    }
    None
}
