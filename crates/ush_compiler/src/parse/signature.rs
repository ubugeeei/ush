use anyhow::{Result, anyhow, bail};
use memchr::{memchr, memrchr};

use super::{
    super::{
        ast::{Call, FunctionParam},
        util::{is_identifier, parse_type, split_once_top_level, split_top_level},
    },
    expr::parse_expr,
};
use crate::types::{AstString as String, HeapVec as Vec};

pub(super) fn parse_function_header(
    header: &str,
) -> Result<(String, Vec<FunctionParam>, Option<super::super::ast::Type>)> {
    let (name, inner, tail) =
        split_paren_form(header).ok_or_else(|| anyhow!("functions must use `fn name(args)`"))?;
    Ok((
        parse_name(name)?,
        parse_params(inner)?,
        parse_return_type(tail)?,
    ))
}

pub(super) fn parse_call(source: &str, asynchronous: bool) -> Result<Call> {
    let (name, inner, tail) =
        split_paren_form(source).ok_or_else(|| anyhow!("invalid function call: {source}"))?;
    if !tail.is_empty() {
        bail!("invalid function call: {source}");
    }
    Ok(Call {
        name: parse_name(name)?,
        args: split_top_level(inner, ',')
            .into_iter()
            .filter(|part| !part.is_empty())
            .map(parse_expr)
            .collect::<Result<Vec<_>>>()?,
        asynchronous,
    })
}

pub(super) fn looks_like_call(source: &str) -> bool {
    split_paren_form(source)
        .map(|(name, _, tail)| is_identifier(name) && tail.is_empty())
        .unwrap_or(false)
}

fn parse_params(source: &str) -> Result<Vec<FunctionParam>> {
    split_top_level(source, ',')
        .into_iter()
        .filter(|part| !part.is_empty())
        .map(|part| {
            let (name, ty) = split_once_top_level(part, ':')
                .ok_or_else(|| anyhow!("invalid parameter: {part}"))?;
            Ok(FunctionParam {
                name: parse_name(name)?,
                ty: parse_type(ty).ok_or_else(|| anyhow!("invalid type: {ty}"))?,
            })
        })
        .collect()
}

pub(super) fn parse_await_task(source: &str) -> Result<String> {
    let task = source
        .trim()
        .strip_prefix("await ")
        .ok_or_else(|| anyhow!("invalid await expression: {source}"))?
        .trim();
    parse_name(task)
}

fn split_paren_form(source: &str) -> Option<(&str, &str, &str)> {
    let open = memchr(b'(', source.as_bytes())?;
    let close = memrchr(b')', source.as_bytes())?;
    (close > open).then_some((
        source[..open].trim(),
        source[open + 1..close].trim(),
        source[close + 1..].trim(),
    ))
}

fn parse_return_type(source: &str) -> Result<Option<super::super::ast::Type>> {
    if source.is_empty() {
        return Ok(None);
    }
    let ty = source
        .strip_prefix("->")
        .ok_or_else(|| anyhow!("invalid function signature suffix: {source}"))?
        .trim();
    Ok(Some(
        parse_type(ty).ok_or_else(|| anyhow!("invalid type: {ty}"))?,
    ))
}

fn parse_name(source: &str) -> Result<String> {
    let trimmed = source.trim();
    if is_identifier(trimmed) {
        Ok(trimmed.into())
    } else {
        bail!("invalid identifier: {trimmed}")
    }
}
