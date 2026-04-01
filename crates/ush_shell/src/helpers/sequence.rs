use anyhow::Result;

use super::{HelperKind, ValueStream};

#[derive(Debug, Clone, Copy)]
pub(super) enum SequenceOp {
    Car,
    Cdr,
    Take(usize),
    Drop(usize),
    Nth(usize),
}

#[derive(Debug, Clone, Copy)]
pub(super) enum Field {
    First,
    Second,
}

pub(super) fn parse_sequence_helper(raw: &str) -> Option<Result<HelperKind>> {
    match raw {
        "car" | "head" => return Some(Ok(HelperKind::Sequence(SequenceOp::Car))),
        "cdr" | "tail" => return Some(Ok(HelperKind::Sequence(SequenceOp::Cdr))),
        "fst" => return Some(Ok(HelperKind::Field(Field::First))),
        "snd" => return Some(Ok(HelperKind::Field(Field::Second))),
        _ => {}
    }

    let (name, count) = parse_count_helper(raw)?;
    let kind = match name {
        "take" => HelperKind::Sequence(SequenceOp::Take(count)),
        "drop" => HelperKind::Sequence(SequenceOp::Drop(count)),
        "nth" => HelperKind::Sequence(SequenceOp::Nth(count)),
        _ => return None,
    };
    Some(Ok(kind))
}

pub(super) fn apply_sequence_op(input: ValueStream, op: &SequenceOp) -> Result<ValueStream> {
    let mut lines = input.into_lines()?;
    let output = match *op {
        SequenceOp::Car => lines.into_iter().take(1).collect(),
        SequenceOp::Cdr => {
            if !lines.is_empty() {
                lines.remove(0);
            }
            lines
        }
        SequenceOp::Take(count) => lines.into_iter().take(count).collect(),
        SequenceOp::Drop(count) => lines.into_iter().skip(count).collect(),
        SequenceOp::Nth(index) => lines
            .into_iter()
            .nth(index)
            .map(|line| vec![line])
            .unwrap_or_default(),
    };
    Ok(if output.is_empty() {
        ValueStream::Empty
    } else {
        ValueStream::Lines(output)
    })
}

fn parse_count_helper(raw: &str) -> Option<(&str, usize)> {
    let open = raw.find('(')?;
    let close = raw.rfind(')')?;
    if close <= open {
        return None;
    }
    let name = raw[..open].trim();
    let count = raw[open + 1..close].trim().parse().ok()?;
    Some((name, count))
}

#[cfg(test)]
mod tests {
    use super::{HelperKind, SequenceOp, ValueStream, apply_sequence_op, parse_sequence_helper};

    #[test]
    fn car_returns_first_line() {
        let output =
            apply_sequence_op(ValueStream::Text("a\nb\n".into()), &SequenceOp::Car).expect("car");
        assert_eq!(output.to_text().expect("text"), "a\n");
    }

    #[test]
    fn cdr_returns_remaining_lines() {
        let output = apply_sequence_op(ValueStream::Text("a\nb\nc\n".into()), &SequenceOp::Cdr)
            .expect("cdr");
        assert_eq!(output.to_text().expect("text"), "b\nc\n");
    }

    #[test]
    fn take_drop_and_nth_are_supported() {
        let take = apply_sequence_op(ValueStream::Text("a\nb\nc\n".into()), &SequenceOp::Take(2))
            .expect("take");
        let drop = apply_sequence_op(ValueStream::Text("a\nb\nc\n".into()), &SequenceOp::Drop(1))
            .expect("drop");
        let nth = apply_sequence_op(ValueStream::Text("a\nb\nc\n".into()), &SequenceOp::Nth(1))
            .expect("nth");
        assert_eq!(take.to_text().expect("text"), "a\nb\n");
        assert_eq!(drop.to_text().expect("text"), "b\nc\n");
        assert_eq!(nth.to_text().expect("text"), "b\n");
    }

    #[test]
    fn parses_aliases_and_counted_helpers() {
        let head = parse_sequence_helper("head")
            .expect("helper")
            .expect("parse");
        let tail = parse_sequence_helper("tail")
            .expect("helper")
            .expect("parse");
        let fst = parse_sequence_helper("fst")
            .expect("helper")
            .expect("parse");
        let snd = parse_sequence_helper("snd")
            .expect("helper")
            .expect("parse");
        let take = parse_sequence_helper("take(2)")
            .expect("helper")
            .expect("parse");
        assert!(matches!(head, HelperKind::Sequence(SequenceOp::Car)));
        assert!(matches!(tail, HelperKind::Sequence(SequenceOp::Cdr)));
        assert!(matches!(fst, HelperKind::Field(super::Field::First)));
        assert!(matches!(snd, HelperKind::Field(super::Field::Second)));
        assert!(matches!(take, HelperKind::Sequence(SequenceOp::Take(2))));
    }
}
