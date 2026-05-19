use super::super::ast::CompareOp;
use crate::scan::{ScanState, advance};

pub(super) fn split_compare(source: &str) -> Option<(&str, CompareOp, &str)> {
    let bytes = source.as_bytes();
    let mut state = ScanState::default();
    let mut index = 0usize;

    while index < bytes.len() {
        if state.top_level()
            && let Some((width, op)) = compare_op(&bytes[index..])
        {
            return Some((source[..index].trim(), op, source[index + width..].trim()));
        }
        index = advance(source, index, &mut state);
    }

    None
}

fn compare_op(source: &[u8]) -> Option<(usize, CompareOp)> {
    match source {
        [b'=', b'=', ..] => Some((2, CompareOp::Eq)),
        [b'!', b'=', ..] => Some((2, CompareOp::Ne)),
        [b'<', b'=', ..] => Some((2, CompareOp::Le)),
        [b'>', b'=', ..] => Some((2, CompareOp::Ge)),
        [b'<', ..] => Some((1, CompareOp::Lt)),
        [b'>', ..] => Some((1, CompareOp::Gt)),
        _ => None,
    }
}
