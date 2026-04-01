use anyhow::{Result, anyhow, bail};

use super::HelperKind;
use super::flat_lambda::parse_flat_lambda;
use super::lambda_syntax::{block_body, parse_call, parse_string_arg, parse_string_literal};

#[derive(Debug, Clone)]
pub(super) enum Transform {
    Identity,
    Constant(String),
    Upper,
    Lower,
    Trim,
    Replace { from: String, to: String },
}

#[derive(Debug, Clone)]
pub(super) enum Predicate {
    Constant(bool),
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
        "map" | "fmap" => parse_transform_lambda(inner).map(HelperKind::Map),
        "each" => parse_transform_lambda(inner).map(HelperKind::Each),
        "filter" => parse_predicate_lambda(inner).map(HelperKind::Filter),
        "any" => parse_predicate_lambda(inner).map(HelperKind::Any),
        "some" => parse_predicate_lambda(inner).map(HelperKind::Some),
        "flat" | "ffmap" => parse_flat_lambda(inner).map(HelperKind::Flat),
        _ => return None,
    };
    Some(kind)
}

pub(super) fn apply_transform(transform: &Transform, input: &str) -> String {
    match transform {
        Transform::Identity => input.to_string(),
        Transform::Constant(value) => value.clone(),
        Transform::Upper => input.to_uppercase(),
        Transform::Lower => input.to_lowercase(),
        Transform::Trim => input.trim().to_string(),
        Transform::Replace { from, to } => input.replace(from, to),
    }
}

impl Predicate {
    pub(super) fn matches(&self, input: &str) -> bool {
        match self {
            Predicate::Constant(value) => *value,
            Predicate::Contains(needle) => input.contains(needle),
            Predicate::StartsWith(prefix) => input.starts_with(prefix),
            Predicate::EndsWith(suffix) => input.ends_with(suffix),
            Predicate::Equals(target) => input == target,
        }
    }
}

fn parse_transform_lambda(source: &str) -> Result<Transform> {
    let lambda = parse_lambda(source)?;
    let body = block_body(lambda.body);
    if body.is_empty() {
        return Ok(Transform::Constant(String::new()));
    }
    if let Some(value) = parse_string_literal(body) {
        return Ok(Transform::Constant(value));
    }
    if matches_arg(body, &lambda) || is_print_identity(body, &lambda) {
        return Ok(Transform::Identity);
    }

    let (name, args) = parse_call(body)?;
    match name {
        "upper" if args.len() == 1 => parse_unary_transform(&args[0], &lambda, Transform::Upper),
        "lower" if args.len() == 1 => parse_unary_transform(&args[0], &lambda, Transform::Lower),
        "trim" if args.len() == 1 => parse_unary_transform(&args[0], &lambda, Transform::Trim),
        "replace" if args.len() == 3 => parse_replace_transform(&args, &lambda),
        _ => bail!("unsupported transform lambda: {source}"),
    }
}

fn parse_predicate_lambda(source: &str) -> Result<Predicate> {
    let lambda = parse_lambda(source)?;
    let body = block_body(lambda.body);
    match body {
        "" | "false" => return Ok(Predicate::Constant(false)),
        "true" => return Ok(Predicate::Constant(true)),
        _ => {}
    }
    let (name, args) = parse_call(body)?;
    match name {
        "contains" if args.len() == 2 => parse_predicate_call(&args, &lambda, Predicate::Contains),
        "starts_with" if args.len() == 2 => {
            parse_predicate_call(&args, &lambda, Predicate::StartsWith)
        }
        "ends_with" if args.len() == 2 => parse_predicate_call(&args, &lambda, Predicate::EndsWith),
        "eq" if args.len() == 2 => parse_predicate_call(&args, &lambda, Predicate::Equals),
        _ => bail!("unsupported predicate lambda: {source}"),
    }
}

#[derive(Debug, Clone)]
struct Lambda<'a> {
    arg: Option<&'a str>,
    body: &'a str,
}

fn parse_lambda(source: &str) -> Result<Lambda<'_>> {
    let (head, body) = source
        .split_once("->")
        .ok_or_else(|| anyhow!("expected `\\arg -> ...` lambda"))?;
    let head = head.trim();
    if let Some(args) = head.strip_prefix('\\') {
        let args = args.trim();
        if args.is_empty() {
            return Ok(Lambda {
                arg: None,
                body: body.trim(),
            });
        }
        let parts = args
            .split(',')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        if parts.len() > 1 {
            bail!("helper lambdas support at most one argument right now");
        }
        return Ok(Lambda {
            arg: Some(parts[0]),
            body: body.trim(),
        });
    }
    if head == "it" {
        return Ok(Lambda {
            arg: Some("it"),
            body: body.trim(),
        });
    }
    bail!("expected helper lambda like `\\it -> ...`")
}

fn matches_arg(source: &str, lambda: &Lambda<'_>) -> bool {
    matches!(lambda.arg, Some(arg) if source == arg)
}

fn is_print_identity(source: &str, lambda: &Lambda<'_>) -> bool {
    matches!(lambda.arg, Some(arg) if source == format!("print({arg})"))
}

fn parse_unary_transform(
    arg: &str,
    lambda: &Lambda<'_>,
    transform: Transform,
) -> Result<Transform> {
    if matches_arg(arg, lambda) {
        return Ok(transform);
    }
    if let Some(value) = parse_string_literal(arg) {
        return Ok(Transform::Constant(apply_transform(&transform, &value)));
    }
    bail!("unsupported transform lambda argument: {arg}")
}

fn parse_replace_transform(args: &[String], lambda: &Lambda<'_>) -> Result<Transform> {
    if matches_arg(&args[0], lambda) {
        return Ok(Transform::Replace {
            from: parse_string_arg(&args[1])?,
            to: parse_string_arg(&args[2])?,
        });
    }
    if let Some(value) = parse_string_literal(&args[0]) {
        return Ok(Transform::Constant(value.replace(
            &parse_string_arg(&args[1])?,
            &parse_string_arg(&args[2])?,
        )));
    }
    bail!("unsupported transform lambda: replace({})", args.join(", "))
}

fn parse_predicate_call(
    args: &[String],
    lambda: &Lambda<'_>,
    build: impl FnOnce(String) -> Predicate,
) -> Result<Predicate> {
    if matches_arg(&args[0], lambda) {
        return Ok(build(parse_string_arg(&args[1])?));
    }
    if let Some(value) = parse_string_literal(&args[0]) {
        return Ok(Predicate::Constant(
            build(parse_string_arg(&args[1])?).matches(&value),
        ));
    }
    bail!("unsupported predicate lambda: {}", args.join(", "))
}
