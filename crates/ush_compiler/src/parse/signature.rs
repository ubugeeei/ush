use anyhow::{Result, anyhow, bail};

use super::{
    super::{
        ast::{Attribute, Call, CallArg, Expr, FunctionParam},
        util::{
            is_identifier, parse_type, split_once_top_level, split_top_level,
            split_top_level_whitespace,
        },
    },
    attr,
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
    parse_call_inner(source, asynchronous, true)
}

pub(super) fn looks_like_call(source: &str) -> bool {
    looks_like_call_inner(source, true)
}

pub(super) fn parse_expr_call(source: &str) -> Result<Option<Call>> {
    Ok(parse_call_parts(source, false)?.map(|(name, args)| Call {
        name,
        args,
        asynchronous: false,
    }))
}

pub(super) fn parse_dollar_call(source: &str, arg: super::super::ast::Expr) -> Result<Call> {
    let mut call = parse_call_inner(source, false, true)?;
    call.args.push(CallArg {
        label: None,
        expr: arg,
    });
    Ok(call)
}

fn parse_call_inner(source: &str, asynchronous: bool, allow_bare_name: bool) -> Result<Call> {
    let (name, args) = parse_call_parts(source, allow_bare_name)?
        .ok_or_else(|| anyhow!("invalid function call: {source}"))?;
    Ok(Call {
        name,
        args,
        asynchronous,
    })
}

fn looks_like_call_inner(source: &str, allow_bare_name: bool) -> bool {
    split_paren_form(source)
        .map(|(name, _, tail)| is_identifier(name) && tail.is_empty())
        .unwrap_or_else(|| {
            parse_call_parts(source, allow_bare_name)
                .ok()
                .flatten()
                .is_some()
        })
}

fn parse_call_parts(source: &str, allow_bare_name: bool) -> Result<Option<(String, Vec<CallArg>)>> {
    if let Some((name, inner, tail)) = split_paren_form(source) {
        if !tail.is_empty() {
            bail!("invalid function call: {source}");
        }
        return Ok(Some((parse_name(name)?, parse_args(inner)?)));
    }

    let parts = split_top_level_whitespace(source.trim());
    if parts.is_empty() || !is_identifier(parts[0]) {
        return Ok(None);
    }
    if parts.len() == 1 && !allow_bare_name {
        return Ok(None);
    }

    let args = if parts.len() == 2 && parts[1] == "()" {
        Vec::new()
    } else {
        parse_call_tokens(&parts[1..])?
    };
    Ok(Some((parts[0].into(), args)))
}

fn parse_args(source: &str) -> Result<Vec<CallArg>> {
    split_top_level(source, ',')
        .into_iter()
        .filter(|part| !part.is_empty())
        .map(parse_call_arg)
        .collect()
}

fn parse_params(source: &str) -> Result<Vec<FunctionParam>> {
    split_top_level(source, ',')
        .into_iter()
        .filter(|part| !part.is_empty())
        .map(|part| {
            let (attrs, rest) = attr::parse_inline_attrs(part)?;
            let (name, ty) = split_once_top_level(rest, ':')
                .ok_or_else(|| anyhow!("invalid parameter: {part}"))?;
            Ok(FunctionParam {
                name: parse_name(name)?,
                ty: parse_type(ty).ok_or_else(|| anyhow!("invalid type: {ty}"))?,
                default: attr_expr(&attrs, "default")?,
                cli_alias: attr_string(&attrs, "alias")?,
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
    let open = source.find('(')?;
    let bytes = source.as_bytes();
    let (mut single, mut double, mut depth) = (false, false, 0usize);

    for (index, byte) in bytes.iter().enumerate().skip(open) {
        match *byte {
            b'\'' if !double => single = !single,
            b'"' if !single => double = !double,
            b'(' if !single && !double => depth += 1,
            b')' if !single && !double => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some((
                        source[..open].trim(),
                        source[open + 1..index].trim(),
                        source[index + 1..].trim(),
                    ));
                }
            }
            _ => {}
        }
    }
    None
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

fn parse_call_tokens(parts: &[&str]) -> Result<Vec<CallArg>> {
    let mut args = Vec::new();
    let mut index = 0usize;
    while index < parts.len() {
        let current = parts[index];
        if let Some(label) = current.strip_suffix(':') {
            args.push(CallArg {
                label: Some(parse_name(label)?),
                expr: parse_expr(
                    parts
                        .get(index + 1)
                        .ok_or_else(|| anyhow!("missing value for `{label}`"))?,
                )?,
            });
            index += 2;
            continue;
        }
        args.push(parse_call_arg(current)?);
        index += 1;
    }
    Ok(args)
}

fn parse_call_arg(source: &str) -> Result<CallArg> {
    if let Some((name, expr)) = split_once_top_level(source, ':') {
        if is_identifier(name) {
            return Ok(CallArg {
                label: Some(parse_name(name)?),
                expr: parse_expr(expr)?,
            });
        }
    }
    Ok(CallArg {
        label: None,
        expr: parse_expr(source)?,
    })
}

fn attr_expr(attrs: &[Attribute], name: &str) -> Result<Option<Expr>> {
    let mut value = None;
    for attr in attrs {
        match attr.name.as_str() {
            other if other == name => {
                if value.is_some() {
                    bail!("duplicate attribute: {name}");
                }
                value = Some(
                    attr.value
                        .clone()
                        .ok_or_else(|| anyhow!("attribute `{name}` requires a value"))?,
                );
            }
            "alias" | "default" => {}
            other => bail!("unsupported parameter attribute: {other}"),
        }
    }
    Ok(value)
}

fn attr_string(attrs: &[Attribute], name: &str) -> Result<Option<String>> {
    match attr_expr(attrs, name)? {
        None => Ok(None),
        Some(Expr::String(value)) => Ok(Some(value)),
        Some(_) => bail!("attribute `{name}` expects a string literal"),
    }
}
