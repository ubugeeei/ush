use anyhow::{Context, Result, anyhow, bail};

use super::{
    super::{
        ast::{Attribute, Statement, StatementKind},
        util::strip_top_level_suffix,
    },
    SourceLine, attr, declaration,
    expr::parse_expr,
    signature,
    statement_support::{
        is_tail_position, parse_alias, parse_shell_escape, parse_shell_statement,
        parse_statement_expr, split_assignment, trim_statement_terminator,
    },
};
use crate::types::HeapVec as Vec;

pub(super) fn parse_block(
    lines: &[SourceLine<'_>],
    cursor: &mut usize,
    allow_declarations: bool,
    allow_tail_expr: bool,
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
        let (trimmed, terminated) = trim_statement_terminator(trimmed);
        let tail_position = allow_tail_expr && !terminated && is_tail_position(lines, *cursor);
        let statement = parse_statement(
            *line_no,
            trimmed,
            lines,
            cursor,
            allow_declarations,
            &attrs,
            tail_position,
        )
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

pub(super) fn parse_let(source: &str) -> Result<StatementKind> {
    let (name, expr) = split_assignment(source).ok_or_else(|| anyhow!("invalid let binding"))?;
    if let Some(rest) = expr.strip_prefix("async ") {
        return Ok(StatementKind::Spawn {
            name: name.into(),
            call: signature::parse_call(rest, true)?,
        });
    }
    if let Some(task) = signature::parse_await_task(expr)? {
        return Ok(StatementKind::Await {
            name: name.into(),
            task,
        });
    }
    Ok(StatementKind::Let {
        name: name.into(),
        expr: parse_expr(expr)?,
    })
}

pub(super) fn parse_inline_body(
    line_no: usize,
    source: &str,
    tail_position: bool,
) -> Result<Statement> {
    let (source, terminated) = trim_statement_terminator(source.trim());
    let tail_position = tail_position && !terminated;
    if let Some(kind) = parse_shell_escape(source)? {
        return Ok(Statement::new(line_no, kind));
    }
    if let Some(rest) = source.strip_prefix("print ") {
        return Ok(Statement::new(
            line_no,
            StatementKind::Print(parse_statement_expr(rest)?),
        ));
    }
    if let Some(rest) = source.strip_prefix("shell ") {
        return Ok(Statement::new(line_no, parse_shell_statement(rest)?));
    }
    if let Some(rest) = source.strip_prefix("raise ") {
        return Ok(Statement::new(
            line_no,
            StatementKind::Raise(parse_statement_expr(rest)?),
        ));
    }
    if let Some(rest) = source.strip_prefix("return ") {
        return Ok(Statement::new(
            line_no,
            StatementKind::Return(parse_statement_expr(rest)?),
        ));
    }
    if let Some(rest) = source.strip_prefix("let ") {
        return Ok(Statement::new(line_no, parse_let(rest)?));
    }
    if let Some(rest) = source.strip_prefix("alias ") {
        return Ok(Statement::new(line_no, parse_alias(rest)?));
    }
    if let Some(rest) = source.strip_prefix("async ") {
        return Ok(Statement::new(
            line_no,
            StatementKind::Call(signature::parse_call(rest, true)?),
        ));
    }
    if tail_position && let Ok(expr) = parse_expr(source) {
        return Ok(Statement::new(line_no, StatementKind::Expr(expr)));
    }
    if let Some(base) = strip_top_level_suffix(source, '?')
        && signature::looks_like_call(base)
    {
        return Ok(Statement::new(
            line_no,
            StatementKind::TryCall(signature::parse_call(base, false)?),
        ));
    }
    if signature::looks_like_call(source) {
        return Ok(Statement::new(
            line_no,
            StatementKind::Call(signature::parse_call(source, false)?),
        ));
    }
    if let Ok(expr) = parse_expr(source) {
        return Ok(Statement::new(line_no, StatementKind::Expr(expr)));
    }
    bail!("unsupported inline statement: {source}")
}

fn parse_statement(
    line_no: usize,
    trimmed: &str,
    lines: &[SourceLine<'_>],
    cursor: &mut usize,
    allow_declarations: bool,
    attrs: &[Attribute],
    tail_position: bool,
) -> Result<Statement> {
    if let Some(kind) = parse_shell_escape(trimmed)? {
        return Ok(Statement::new(line_no, kind));
    }
    if allow_declarations {
        if let Some(kind) =
            declaration::parse_declaration(line_no, trimmed, lines, cursor, attrs, tail_position)?
        {
            return Ok(Statement::new(line_no, kind));
        }
    }
    if let Some(rest) = trimmed.strip_prefix("let ") {
        return Ok(Statement::new(line_no, parse_let(rest)?));
    }
    if let Some(rest) = trimmed.strip_prefix("print ") {
        return Ok(Statement::new(
            line_no,
            StatementKind::Print(parse_statement_expr(rest)?),
        ));
    }
    if let Some(rest) = trimmed.strip_prefix("shell ") {
        return Ok(Statement::new(line_no, parse_shell_statement(rest)?));
    }
    if let Some(rest) = trimmed.strip_prefix("raise ") {
        return Ok(Statement::new(
            line_no,
            StatementKind::Raise(parse_statement_expr(rest)?),
        ));
    }
    if let Some(rest) = trimmed.strip_prefix("return ") {
        return Ok(Statement::new(
            line_no,
            StatementKind::Return(parse_statement_expr(rest)?),
        ));
    }
    if let Some(rest) = trimmed.strip_prefix("alias ") {
        return Ok(Statement::new(line_no, parse_alias(rest)?));
    }
    if let Some(rest) = trimmed.strip_prefix("async ") {
        return Ok(Statement::new(
            line_no,
            StatementKind::Call(signature::parse_call(rest, true)?),
        ));
    }
    if let Some(subject) = trimmed.strip_prefix("match ") {
        return Ok(Statement::new(
            line_no,
            declaration::parse_match(subject, lines, cursor, tail_position)?,
        ));
    }
    if tail_position && let Ok(expr) = parse_expr(trimmed) {
        return Ok(Statement::new(line_no, StatementKind::Expr(expr)));
    }
    if let Some(base) = strip_top_level_suffix(trimmed, '?')
        && signature::looks_like_call(base)
    {
        return Ok(Statement::new(
            line_no,
            StatementKind::TryCall(signature::parse_call(base, false)?),
        ));
    }
    if signature::looks_like_call(trimmed) {
        return Ok(Statement::new(
            line_no,
            StatementKind::Call(signature::parse_call(trimmed, false)?),
        ));
    }
    if let Ok(expr) = parse_expr(trimmed) {
        return Ok(Statement::new(line_no, StatementKind::Expr(expr)));
    }
    bail!("unsupported syntax: {trimmed}")
}

fn is_block_statement(statement: &Statement) -> bool {
    matches!(
        statement.kind,
        StatementKind::Enum(_)
            | StatementKind::Trait(_)
            | StatementKind::Impl(_)
            | StatementKind::Function(_)
            | StatementKind::Match { .. }
    )
}

fn accepts_attributes(statement: &Statement) -> bool {
    matches!(statement.kind, StatementKind::Function(_))
}
