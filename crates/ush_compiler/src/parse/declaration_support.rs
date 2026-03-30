use anyhow::{Result, bail};

use super::super::util::is_identifier;
use super::SourceLine;
use crate::types::AstString as String;

pub(super) fn finish_block(
    lines: &[SourceLine<'_>],
    cursor: &mut usize,
    kind: &str,
) -> Result<bool> {
    if *cursor >= lines.len() {
        bail!("unterminated {kind}");
    }
    let trimmed = lines[*cursor].1.trim();
    let terminated = match trimmed {
        "}" => false,
        "};" => true,
        _ => bail!("unterminated {kind}"),
    };
    *cursor += 1;
    Ok(terminated)
}

pub(super) fn parse_name(source: &str) -> Result<String> {
    let trimmed = source.trim();
    if is_identifier(trimmed) {
        Ok(trimmed.into())
    } else {
        bail!("invalid identifier: {trimmed}")
    }
}
