mod expr;
mod signature;
mod statement;

use anyhow::{Result, bail};
use bumpalo::{Bump, collections::Vec as BumpVec};

use super::ast::Statement;
use crate::types::HeapVec as Vec;

pub(super) type SourceLine<'a> = (usize, &'a str);

pub(crate) fn parse_program(source: &str) -> Result<Vec<Statement>> {
    let arena = Bump::new();
    let mut lines = BumpVec::new_in(&arena);
    for (index, line) in source.lines().enumerate() {
        lines.push((index + 1, line));
    }
    let mut cursor = 0usize;
    let program = statement::parse_block(&lines, &mut cursor, true)?;

    if cursor < lines.len() {
        let (line_no, line) = &lines[cursor];
        bail!("line {line_no}: unexpected closing brace: {}", line.trim());
    }

    Ok(program)
}
