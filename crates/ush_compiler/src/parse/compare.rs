use super::super::ast::CompareOp;

pub(super) fn split_compare(source: &str) -> Option<(&str, CompareOp, &str)> {
    let bytes = source.as_bytes();
    let (mut single, mut double, mut paren, mut brace) = (false, false, 0usize, 0usize);
    let mut index = 0usize;

    while index < bytes.len() {
        let ch = bytes[index] as char;
        match ch {
            '\'' if !double => single = !single,
            '"' if !single => double = !double,
            '(' if !single && !double => paren += 1,
            ')' if !single && !double && paren > 0 => paren -= 1,
            '{' if !single && !double => brace += 1,
            '}' if !single && !double && brace > 0 => brace -= 1,
            _ if single || double || paren > 0 || brace > 0 => {}
            _ => {
                if let Some((width, op)) = compare_op(&bytes[index..]) {
                    return Some((source[..index].trim(), op, source[index + width..].trim()));
                }
            }
        }
        index += ch.len_utf8();
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
