use anyhow::{Context, Result, anyhow, bail};

use super::{
    super::{
        ast::{Attribute, Expr, Statement},
        util::split_once_top_level,
    },
    SourceLine, attr, declaration,
    expr::parse_expr,
    signature,
};
use crate::types::HeapVec as Vec;

pub(super) fn parse_block(
    lines: &[SourceLine<'_>],
    cursor: &mut usize,
    allow_declarations: bool,
) -> Result<Vec<Statement>> {
    let mut statements = Vec::new();
    let mut attrs = Vec::new();
    while *cursor < lines.len() {
        let (line_no, line) = &lines[*cursor];
        let trimmed = line.trim();
        if trimmed.starts_with("#[") {
            attrs.push(
                attr::parse_attribute_line(trimmed).with_context(|| format!("line {line_no}"))?,
            );
            *cursor += 1;
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with('#') {
            *cursor += 1;
            continue;
        }
        if trimmed == "}" {
            if !attrs.is_empty() {
                bail!("line {line_no}: dangling attributes before closing brace");
            }
            break;
        }
        let statement = parse_statement(trimmed, lines, cursor, allow_declarations, &attrs)
            .with_context(|| format!("line {line_no}"))?;
        if !attrs.is_empty() && !accepts_attributes(&statement) {
            bail!("line {line_no}: attributes are only valid on declarations");
        }
        if !is_block_statement(&statement) {
            *cursor += 1;
        }
        attrs.clear();
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
    if let Some(task) = signature::parse_await_task(expr)? {
        return Ok(Statement::Await {
            name: name.into(),
            task,
        });
    }
    Ok(Statement::Let {
        name: name.into(),
        expr: parse_expr(expr)?,
    })
}

pub(super) fn parse_inline_statement(source: &str) -> Result<Statement> {
    if let Some(statement) = parse_shell_escape(source)? {
        return Ok(statement);
    }
    if let Some(rest) = source.strip_prefix("print ") {
        return Ok(Statement::Print(parse_statement_expr(rest)?));
    }
    if let Some(rest) = source.strip_prefix("shell ") {
        return Ok(Statement::Shell(parse_statement_expr(rest)?));
    }
    if let Some(rest) = source.strip_prefix("return ") {
        return Ok(Statement::Return(parse_statement_expr(rest)?));
    }
    if let Some(rest) = source.strip_prefix("let ") {
        return parse_let(rest);
    }
    if let Some(rest) = source.strip_prefix("alias ") {
        return parse_alias(rest);
    }
    if let Some(rest) = source.strip_prefix("async ") {
        return Ok(Statement::Call(signature::parse_call(rest, true)?));
    }
    if signature::looks_like_call(source) {
        return Ok(Statement::Call(signature::parse_call(source, false)?));
    }
    bail!("unsupported inline statement: {source}")
}

fn parse_statement(
    trimmed: &str,
    lines: &[SourceLine<'_>],
    cursor: &mut usize,
    allow_declarations: bool,
    attrs: &[Attribute],
) -> Result<Statement> {
    if let Some(statement) = parse_shell_escape(trimmed)? {
        return Ok(statement);
    }
    if allow_declarations {
        if let Some(statement) = declaration::parse_declaration(trimmed, lines, cursor, attrs)? {
            return Ok(statement);
        }
    }
    if let Some(rest) = trimmed.strip_prefix("let ") {
        return parse_let(rest);
    }
    if let Some(rest) = trimmed.strip_prefix("print ") {
        return Ok(Statement::Print(parse_statement_expr(rest)?));
    }
    if let Some(rest) = trimmed.strip_prefix("shell ") {
        return Ok(Statement::Shell(parse_statement_expr(rest)?));
    }
    if let Some(rest) = trimmed.strip_prefix("return ") {
        return Ok(Statement::Return(parse_statement_expr(rest)?));
    }
    if let Some(rest) = trimmed.strip_prefix("alias ") {
        return parse_alias(rest);
    }
    if let Some(rest) = trimmed.strip_prefix("async ") {
        return Ok(Statement::Call(signature::parse_call(rest, true)?));
    }
    if let Some(subject) = trimmed.strip_prefix("match ") {
        return declaration::parse_match(subject, lines, cursor);
    }
    if signature::looks_like_call(trimmed) {
        return Ok(Statement::Call(signature::parse_call(trimmed, false)?));
    }
    bail!("unsupported syntax: {trimmed}")
}

fn parse_alias(source: &str) -> Result<Statement> {
    let (name, value) = split_assignment(source).ok_or_else(|| anyhow!("invalid alias binding"))?;
    Ok(Statement::Alias {
        name: name.into(),
        value: parse_expr(value)?,
    })
}

fn split_assignment(source: &str) -> Option<(&str, &str)> {
    let (name, expr) = split_once_top_level(source, '=')?;
    Some((name.trim(), expr.trim()))
}

fn parse_statement_expr(source: &str) -> Result<super::super::ast::Expr> {
    parse_expr(source.trim().strip_prefix('$').unwrap_or(source).trim())
}

fn parse_shell_escape(source: &str) -> Result<Option<Statement>> {
    let Some(rest) = source.strip_prefix('$') else {
        return Ok(None);
    };
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return Ok(None);
    }
    let command = rest.trim();
    if command.is_empty() {
        bail!("shell escape requires a command");
    }
    Ok(Some(Statement::Shell(Expr::String(command.into()))))
}

fn is_block_statement(statement: &Statement) -> bool {
    matches!(
        statement,
        Statement::Enum(_)
            | Statement::Trait(_)
            | Statement::Impl(_)
            | Statement::Function(_)
            | Statement::Match { .. }
    )
}

fn accepts_attributes(statement: &Statement) -> bool {
    matches!(statement, Statement::Function(_))
}
