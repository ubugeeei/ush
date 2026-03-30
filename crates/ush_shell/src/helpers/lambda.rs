use anyhow::{Result, anyhow, bail};

use super::HelperKind;

#[derive(Debug, Clone)]
pub(super) enum Transform {
    Identity,
    Upper,
    Lower,
    Trim,
    Replace { from: String, to: String },
}

#[derive(Debug, Clone)]
pub(super) enum Predicate {
    Contains(String),
    StartsWith(String),
    EndsWith(String),
    Equals(String),
}

pub(super) fn parse_lambda_helper(raw: &str) -> Option<Result<HelperKind>> {
    let open = raw.find('(')?;
    let close = raw.rfind(')')?;
    if close <= open {
        return Some(Err(anyhow!("invalid helper invocation: {raw}")));
    }

    let name = raw[..open].trim();
    let inner = raw[open + 1..close].trim();
    let kind = match name {
        "map" => parse_transform_lambda(inner).map(HelperKind::Map),
        "each" => parse_transform_lambda(inner).map(HelperKind::Each),
        "filter" => parse_predicate_lambda(inner).map(HelperKind::Filter),
        "any" => parse_predicate_lambda(inner).map(HelperKind::Any),
        "some" => parse_predicate_lambda(inner).map(HelperKind::Some),
        _ => return None,
    };
    Some(kind)
}

pub(super) fn apply_transform(transform: &Transform, input: &str) -> String {
    match transform {
        Transform::Identity => input.to_string(),
        Transform::Upper => input.to_uppercase(),
        Transform::Lower => input.to_lowercase(),
        Transform::Trim => input.trim().to_string(),
        Transform::Replace { from, to } => input.replace(from, to),
    }
}

impl Predicate {
    pub(super) fn matches(&self, input: &str) -> bool {
        match self {
            Predicate::Contains(needle) => input.contains(needle),
            Predicate::StartsWith(prefix) => input.starts_with(prefix),
            Predicate::EndsWith(suffix) => input.ends_with(suffix),
            Predicate::Equals(target) => input == target,
        }
    }
}

fn parse_transform_lambda(source: &str) -> Result<Transform> {
    let body = lambda_body(source)?;
    if body == "it" || body == "print(it)" {
        return Ok(Transform::Identity);
    }

    let (name, args) = parse_call(body)?;
    match name {
        "upper" if args.as_slice() == ["it"] => Ok(Transform::Upper),
        "lower" if args.as_slice() == ["it"] => Ok(Transform::Lower),
        "trim" if args.as_slice() == ["it"] => Ok(Transform::Trim),
        "replace" if args.len() == 3 && args[0] == "it" => Ok(Transform::Replace {
            from: parse_string_arg(&args[1])?,
            to: parse_string_arg(&args[2])?,
        }),
        _ => bail!("unsupported transform lambda: {source}"),
    }
}

fn parse_predicate_lambda(source: &str) -> Result<Predicate> {
    let body = lambda_body(source)?;
    let (name, args) = parse_call(body)?;
    match name {
        "contains" if args.len() == 2 && args[0] == "it" => {
            Ok(Predicate::Contains(parse_string_arg(&args[1])?))
        }
        "starts_with" if args.len() == 2 && args[0] == "it" => {
            Ok(Predicate::StartsWith(parse_string_arg(&args[1])?))
        }
        "ends_with" if args.len() == 2 && args[0] == "it" => {
            Ok(Predicate::EndsWith(parse_string_arg(&args[1])?))
        }
        "eq" if args.len() == 2 && args[0] == "it" => {
            Ok(Predicate::Equals(parse_string_arg(&args[1])?))
        }
        _ => bail!("unsupported predicate lambda: {source}"),
    }
}

fn lambda_body(source: &str) -> Result<&str> {
    let (arg, body) = source
        .split_once("->")
        .ok_or_else(|| anyhow!("expected `it -> ...` lambda"))?;
    if arg.trim() != "it" {
        bail!("only `it` lambdas are supported right now");
    }
    Ok(body.trim())
}

fn parse_call(source: &str) -> Result<(&str, Vec<String>)> {
    let open = source
        .find('(')
        .ok_or_else(|| anyhow!("expected function call syntax"))?;
    let close = source
        .rfind(')')
        .ok_or_else(|| anyhow!("expected function call syntax"))?;
    Ok((
        source[..open].trim(),
        split_args(source[open + 1..close].trim()),
    ))
}

fn split_args(source: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut single = false;
    let mut double = false;
    let mut start = 0usize;
    for (index, ch) in source.char_indices() {
        match ch {
            '\'' if !double => single = !single,
            '"' if !single => double = !double,
            ',' if !single && !double => {
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

fn parse_string_arg(source: &str) -> Result<String> {
    if source.len() >= 2
        && ((source.starts_with('"') && source.ends_with('"'))
            || (source.starts_with('\'') && source.ends_with('\'')))
    {
        return Ok(source[1..source.len() - 1].to_string());
    }
    bail!("expected string literal, found {source}")
}
