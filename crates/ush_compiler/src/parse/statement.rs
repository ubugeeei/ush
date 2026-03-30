use alloc::boxed::Box;

use anyhow::{Context, Result, anyhow, bail};

use super::{
    super::{
        ast::{EnumDef, FunctionDef, Statement, VariantDef, VariantFields},
        util::{is_identifier, parse_brace_body, parse_paren_body, split_once_top_level},
    },
    SourceLine,
    expr::{parse_expr, parse_named_type_list, parse_pattern, parse_type_list},
    signature,
};
use crate::types::{AstString as String, HeapVec as Vec};

pub(super) fn parse_block(
    lines: &[SourceLine<'_>],
    cursor: &mut usize,
    allow_declarations: bool,
) -> Result<Vec<Statement>> {
    let mut statements = Vec::new();
    while *cursor < lines.len() {
        let (line_no, line) = &lines[*cursor];
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            *cursor += 1;
            continue;
        }
        if trimmed == "}" {
            break;
        }

        let statement = parse_statement(trimmed, lines, cursor, allow_declarations)
            .with_context(|| format!("line {line_no}"))?;
        if !is_block_statement(&statement) {
            *cursor += 1;
        }
        statements.push(statement);
    }
    Ok(statements)
}

pub(super) fn parse_let(source: &str) -> Result<Statement> {
    let (name, expr) = split_assignment(source).ok_or_else(|| anyhow!("invalid let binding"))?;
    if let Some(rest) = expr.strip_prefix("async ") {
        return Ok(Statement::Spawn {
            name: name.into(),
            call: signature::parse_call(rest, true)?,
        });
    }
    if expr.trim_start().starts_with("await ") {
        return Ok(Statement::Await {
            name: name.into(),
            task: signature::parse_await_task(expr)?,
        });
    }
    Ok(Statement::Let {
        name: name.into(),
        expr: parse_expr(expr)?,
    })
}

fn parse_statement(
    trimmed: &str,
    lines: &[SourceLine<'_>],
    cursor: &mut usize,
    allow_declarations: bool,
) -> Result<Statement> {
    if allow_declarations {
        if let Some(rest) = trimmed.strip_prefix("enum ") {
            return parse_enum(rest, lines, cursor);
        }
        if let Some(rest) = trimmed.strip_prefix("fn ") {
            return parse_function(rest, lines, cursor);
        }
    }
    if let Some(rest) = trimmed.strip_prefix("let ") {
        return parse_let(rest);
    }
    if let Some(rest) = trimmed.strip_prefix("print ") {
        return Ok(Statement::Print(parse_expr(rest)?));
    }
    if let Some(rest) = trimmed.strip_prefix("shell ") {
        return Ok(Statement::Shell(parse_expr(rest)?));
    }
    if let Some(rest) = trimmed.strip_prefix("return ") {
        return Ok(Statement::Return(parse_expr(rest)?));
    }
    if let Some(rest) = trimmed.strip_prefix("async ") {
        return Ok(Statement::Call(signature::parse_call(rest, true)?));
    }
    if let Some(subject) = trimmed.strip_prefix("match ") {
        return parse_match(subject, lines, cursor);
    }
    if signature::looks_like_call(trimmed) {
        return Ok(Statement::Call(signature::parse_call(trimmed, false)?));
    }
    bail!("unsupported syntax: {trimmed}")
}

fn parse_enum(header: &str, lines: &[SourceLine<'_>], cursor: &mut usize) -> Result<Statement> {
    let name = header
        .strip_suffix('{')
        .ok_or_else(|| anyhow!("expected `{{` after enum name"))?
        .trim();
    let mut variants = Vec::new();
    let name = parse_name(name)?;

    *cursor += 1;
    while *cursor < lines.len() {
        let variant_line = lines[*cursor].1.trim().trim_end_matches(',').trim();
        if variant_line == "}" {
            break;
        }
        if variant_line.is_empty() || variant_line.starts_with('#') {
            *cursor += 1;
            continue;
        }
        variants.push(parse_variant_def(variant_line)?);
        *cursor += 1;
    }
    finish_block(lines, cursor, "enum definition")?;
    Ok(Statement::Enum(EnumDef { name, variants }))
}

fn parse_function(header: &str, lines: &[SourceLine<'_>], cursor: &mut usize) -> Result<Statement> {
    let head = header
        .strip_suffix('{')
        .ok_or_else(|| anyhow!("expected `{{` after function signature"))?
        .trim();
    let (name, params, return_type) = signature::parse_function_header(head)?;
    *cursor += 1;
    let body = parse_block(lines, cursor, false)?;
    finish_block(lines, cursor, "function body")?;
    Ok(Statement::Function(FunctionDef {
        name,
        params,
        return_type,
        body,
    }))
}

fn parse_match(subject: &str, lines: &[SourceLine<'_>], cursor: &mut usize) -> Result<Statement> {
    let subject = subject
        .strip_suffix('{')
        .ok_or_else(|| anyhow!("expected `{{` after match subject"))?
        .trim();
    let mut arms = Vec::new();
    *cursor += 1;

    while *cursor < lines.len() {
        let (line_no, line) = &lines[*cursor];
        let trimmed = line.trim();
        if trimmed == "}" {
            break;
        }
        if trimmed.is_empty() || trimmed.starts_with('#') {
            *cursor += 1;
            continue;
        }

        let (pattern, statement) = trimmed
            .split_once("=>")
            .ok_or_else(|| anyhow!("invalid match arm"))?;
        arms.push((
            parse_pattern(pattern.trim())?,
            Box::new(
                parse_inline_statement(statement.trim())
                    .with_context(|| format!("line {line_no}: invalid match arm body"))?,
            ),
        ));
        *cursor += 1;
    }
    finish_block(lines, cursor, "match expression")?;
    Ok(Statement::Match {
        expr: parse_expr(subject)?,
        arms,
    })
}

fn parse_variant_def(source: &str) -> Result<VariantDef> {
    if let Some((head, inner)) = parse_paren_body(source) {
        return Ok(VariantDef {
            name: parse_name(head)?,
            fields: VariantFields::Tuple(parse_type_list(inner)?),
        });
    }
    if let Some((head, inner)) = parse_brace_body(source) {
        return Ok(VariantDef {
            name: parse_name(head)?,
            fields: VariantFields::Struct(parse_named_type_list(inner)?),
        });
    }
    Ok(VariantDef {
        name: parse_name(source)?,
        fields: VariantFields::Unit,
    })
}

fn parse_inline_statement(source: &str) -> Result<Statement> {
    if let Some(rest) = source.strip_prefix("print ") {
        return Ok(Statement::Print(parse_expr(rest)?));
    }
    if let Some(rest) = source.strip_prefix("shell ") {
        return Ok(Statement::Shell(parse_expr(rest)?));
    }
    if let Some(rest) = source.strip_prefix("return ") {
        return Ok(Statement::Return(parse_expr(rest)?));
    }
    if let Some(rest) = source.strip_prefix("let ") {
        return parse_let(rest);
    }
    if let Some(rest) = source.strip_prefix("async ") {
        return Ok(Statement::Call(signature::parse_call(rest, true)?));
    }
    if signature::looks_like_call(source) {
        return Ok(Statement::Call(signature::parse_call(source, false)?));
    }
    bail!("unsupported inline statement: {source}")
}

fn split_assignment(source: &str) -> Option<(&str, &str)> {
    let (name, expr) = split_once_top_level(source, '=')?;
    Some((name.trim(), expr.trim()))
}

fn finish_block(lines: &[SourceLine<'_>], cursor: &mut usize, kind: &str) -> Result<()> {
    if *cursor >= lines.len() || lines[*cursor].1.trim() != "}" {
        bail!("unterminated {kind}");
    }
    *cursor += 1;
    Ok(())
}

fn is_block_statement(statement: &Statement) -> bool {
    matches!(
        statement,
        Statement::Enum(_) | Statement::Function(_) | Statement::Match { .. }
    )
}

fn parse_name(source: &str) -> Result<String> {
    let trimmed = source.trim();
    if is_identifier(trimmed) {
        Ok(trimmed.into())
    } else {
        bail!("invalid identifier: {trimmed}")
    }
}
