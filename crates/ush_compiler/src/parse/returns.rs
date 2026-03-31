use anyhow::{Result, anyhow, bail};

use crate::{
    ast::Type,
    errors::{ErrorSet, ErrorType},
    types::AstString as String,
    util::{
        is_identifier, parse_type, split_once_top_level, split_top_level, strip_wrapping_parens,
    },
};

pub(super) fn parse_function_return(source: &str) -> Result<(Option<Type>, Option<ErrorSet>)> {
    if source.is_empty() {
        return Ok((Some(Type::Unit), None));
    }
    let ty = source
        .strip_prefix("->")
        .ok_or_else(|| anyhow!("invalid function signature suffix: {source}"))?
        .trim();
    if let Some((errors, value)) = split_once_top_level(ty, '!') {
        return Ok((
            Some(parse_type_name(value)?),
            Some(parse_error_set(errors)?),
        ));
    }
    Ok((Some(parse_type_name(ty)?), None))
}

fn parse_type_name(source: &str) -> Result<Type> {
    parse_type(source).ok_or_else(|| anyhow!("invalid type: {source}"))
}

fn parse_error_set(source: &str) -> Result<ErrorSet> {
    let inner = strip_wrapping_parens(source).unwrap_or(source).trim();
    if inner.is_empty() {
        bail!("error set cannot be empty");
    }

    let mut errors = ErrorSet::default();
    for part in split_top_level(inner, '|') {
        let part = part.trim();
        if part.is_empty() {
            bail!("invalid error set: {source}");
        }
        errors.insert(parse_error_type(part)?);
    }
    Ok(errors)
}

fn parse_error_type(source: &str) -> Result<ErrorType> {
    let trimmed = source.trim();
    if trimmed == "unknown" {
        return Ok(ErrorType::Unknown);
    }
    if is_identifier(trimmed) {
        return Ok(ErrorType::Known(String::from(trimmed)));
    }
    bail!("invalid error type: {trimmed}")
}
