use alloc::boxed::Box;

use anyhow::Result;

use super::{
    super::ast::{Expr, MethodCall},
    expr::parse_atom,
    path::parse_identifier,
    signature,
};
use crate::types::HeapVec as Vec;
use crate::util::parse_paren_body;

pub(super) fn parse_member_chain(source: &str) -> Result<Option<Expr>> {
    let parts = split_members(source);
    if parts.len() <= 1 {
        return Ok(None);
    }
    let mut expr = parse_atom(parts[0])?;
    for part in &parts[1..] {
        if let Some((head, inner)) = parse_paren_body(part) {
            expr = Expr::MethodCall(MethodCall {
                receiver: Box::new(expr),
                method: parse_identifier(head.trim())?,
                args: signature::parse_call_args(inner)?,
            });
            continue;
        }
        expr = Expr::Field {
            base: Box::new(expr),
            name: parse_identifier(part.trim())?,
        };
    }
    Ok(Some(expr))
}

pub(super) fn split_range(source: &str) -> Option<(&str, &str)> {
    let mut single = false;
    let mut double = false;
    let mut paren = 0usize;
    let mut brace = 0usize;
    let mut bracket = 0usize;
    let bytes = source.as_bytes();

    for index in 0..bytes.len().saturating_sub(1) {
        match bytes[index] {
            b'\'' if !double => single = !single,
            b'"' if !single => double = !double,
            b'(' if !single && !double => paren += 1,
            b')' if !single && !double && paren > 0 => paren -= 1,
            b'{' if !single && !double => brace += 1,
            b'}' if !single && !double && brace > 0 => brace -= 1,
            b'[' if !single && !double => bracket += 1,
            b']' if !single && !double && bracket > 0 => bracket -= 1,
            b'.' if !single
                && !double
                && paren == 0
                && brace == 0
                && bracket == 0
                && bytes[index + 1] == b'.' =>
            {
                return Some((source[..index].trim(), source[index + 2..].trim()));
            }
            _ => {}
        }
    }
    None
}

fn split_members(source: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let (mut single, mut double, mut paren, mut brace, mut bracket) =
        (false, false, 0usize, 0usize, 0usize);
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
            b'.' if !single && !double && paren == 0 && brace == 0 && bracket == 0 => {
                let prev = index.checked_sub(1).and_then(|it| bytes.get(it)).copied();
                let next = bytes.get(index + 1).copied();
                if matches!(
                    (prev, next),
                    (Some(b'.'), _) | (_, Some(b'.')) | (_, Some(b':'))
                ) {
                    continue;
                }
                parts.push(source[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }

    if parts.is_empty() {
        return parts;
    }
    parts.push(source[start..].trim());
    parts
}
