use alloc::string::ToString;

use memchr::{memchr, memmem, memrchr};
use phf::phf_map;

use super::ast::Type;
use crate::types::{AstString as String, AstVec as Vec, OutputString};

#[derive(Clone, Copy)]
enum PrimitiveType {
    String,
    Int,
    Bool,
}

static PRIMITIVE_TYPES: phf::Map<&'static str, PrimitiveType> = phf_map! {
    "String" => PrimitiveType::String,
    "Int" => PrimitiveType::Int,
    "Bool" => PrimitiveType::Bool,
};

pub(crate) fn split_top_level(source: &str, separator: char) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0usize;
    let (mut single, mut double, mut paren, mut brace) = (false, false, 0usize, 0usize);

    for (index, ch) in source.char_indices() {
        match ch {
            '\'' if !double => single = !single,
            '"' if !single => double = !double,
            '(' if !single && !double => paren += 1,
            ')' if !single && !double && paren > 0 => paren -= 1,
            '{' if !single && !double => brace += 1,
            '}' if !single && !double && brace > 0 => brace -= 1,
            _ if ch == separator && !single && !double && paren == 0 && brace == 0 => {
                result.push(source[start..index].trim());
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }

    if start == 0 {
        let mut single = Vec::new();
        single.push(source.trim());
        return single;
    }
    result.push(source[start..].trim());
    result
}

pub(crate) fn split_top_level_whitespace(source: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = None;
    let (mut single, mut double, mut paren, mut brace) = (false, false, 0usize, 0usize);

    for (index, ch) in source.char_indices() {
        if !ch.is_whitespace() && start.is_none() {
            start = Some(index);
        }
        match ch {
            '\'' if !double => single = !single,
            '"' if !single => double = !double,
            '(' if !single && !double => paren += 1,
            ')' if !single && !double && paren > 0 => paren -= 1,
            '{' if !single && !double => brace += 1,
            '}' if !single && !double && brace > 0 => brace -= 1,
            _ if ch.is_whitespace() && !single && !double && paren == 0 && brace == 0 => {
                if let Some(begin) = start.take() {
                    result.push(source[begin..index].trim());
                }
            }
            _ => {}
        }
    }

    if let Some(begin) = start {
        result.push(source[begin..].trim());
    }
    result
}

pub(crate) fn split_once_top_level(source: &str, separator: char) -> Option<(&str, &str)> {
    let (mut single, mut double, mut paren, mut brace) = (false, false, 0usize, 0usize);
    for (index, ch) in source.char_indices() {
        match ch {
            '\'' if !double => single = !single,
            '"' if !single => double = !double,
            '(' if !single && !double => paren += 1,
            ')' if !single && !double && paren > 0 => paren -= 1,
            '{' if !single && !double => brace += 1,
            '}' if !single && !double && brace > 0 => brace -= 1,
            _ if ch == separator && !single && !double && paren == 0 && brace == 0 => {
                return Some((source[..index].trim(), source[index + 1..].trim()));
            }
            _ => {}
        }
    }
    None
}

pub(crate) fn strip_top_level_suffix(source: &str, suffix: char) -> Option<&str> {
    let trimmed = source.trim();
    let (mut single, mut double, mut paren, mut brace) = (false, false, 0usize, 0usize);
    let mut candidate = None;

    for (index, ch) in trimmed.char_indices() {
        match ch {
            '\'' if !double => single = !single,
            '"' if !single => double = !double,
            '(' if !single && !double => paren += 1,
            ')' if !single && !double && paren > 0 => paren -= 1,
            '{' if !single && !double => brace += 1,
            '}' if !single && !double && brace > 0 => brace -= 1,
            _ if ch == suffix && !single && !double && paren == 0 && brace == 0 => {
                candidate = Some(index);
            }
            _ => {}
        }
    }

    let index = candidate?;
    (index + suffix.len_utf8() == trimmed.len()).then_some(trimmed[..index].trim())
}

pub(crate) fn split_path(source: &str) -> Option<(String, String)> {
    let index = memmem::find(source.as_bytes(), b"::")?;
    Some((
        source[..index].trim().into(),
        source[index + 2..].trim().into(),
    ))
}

pub(crate) fn parse_paren_body(source: &str) -> Option<(&str, &str)> {
    let open = memchr(b'(', source.as_bytes())?;
    let close = memrchr(b')', source.as_bytes())?;
    (close > open).then_some((source[..open].trim(), source[open + 1..close].trim()))
}

pub(crate) fn strip_wrapping_parens(source: &str) -> Option<&str> {
    let trimmed = source.trim();
    if !trimmed.starts_with('(') || !trimmed.ends_with(')') {
        return None;
    }

    let bytes = trimmed.as_bytes();
    let (mut single, mut double, mut depth) = (false, false, 0usize);
    for (index, byte) in bytes.iter().enumerate() {
        match *byte {
            b'\'' if !double => single = !single,
            b'"' if !single => double = !double,
            b'(' if !single && !double => depth += 1,
            b')' if !single && !double => {
                depth = depth.saturating_sub(1);
                if depth == 0 && index + 1 != trimmed.len() {
                    return None;
                }
            }
            _ => {}
        }
    }

    Some(trimmed[1..trimmed.len() - 1].trim())
}

pub(crate) fn parse_brace_body(source: &str) -> Option<(&str, &str)> {
    let open = memchr(b'{', source.as_bytes())?;
    let close = memrchr(b'}', source.as_bytes())?;
    (close > open).then_some((source[..open].trim(), source[open + 1..close].trim()))
}

pub(crate) fn parse_string_literal(source: &str) -> Option<String> {
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

pub(crate) fn is_identifier(source: &str) -> bool {
    let mut chars = source.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

pub(crate) fn parse_type(source: &str) -> Option<Type> {
    let trimmed = source.trim();
    if matches!(trimmed, "()" | "Unit") {
        return Some(Type::Unit);
    }
    match PRIMITIVE_TYPES.get(trimmed).copied() {
        Some(PrimitiveType::String) => Some(Type::String),
        Some(PrimitiveType::Int) => Some(Type::Int),
        Some(PrimitiveType::Bool) => Some(Type::Bool),
        None if is_identifier(trimmed) => Some(Type::Adt(trimmed.into())),
        None => None,
    }
}

pub(crate) fn shell_quote(value: &str) -> OutputString {
    if value.is_empty() {
        return "''".to_string();
    }
    format!("'{}'", value.replace('\'', r#"'"'"'"#))
}
