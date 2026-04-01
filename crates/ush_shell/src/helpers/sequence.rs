use anyhow::Result;

use super::{ValueStream, flat_lambda::FlatTransform};

pub(super) fn car(input: ValueStream) -> Result<ValueStream> {
    Ok(input
        .into_lines()?
        .into_iter()
        .next()
        .map(|line| ValueStream::Lines(vec![line]))
        .unwrap_or_default())
}

pub(super) fn cdr(input: ValueStream) -> Result<ValueStream> {
    let mut lines = input.into_lines()?;
    if lines.is_empty() {
        return Ok(ValueStream::Empty);
    }
    lines.remove(0);
    Ok(if lines.is_empty() {
        ValueStream::Empty
    } else {
        ValueStream::Lines(lines)
    })
}

pub(super) fn flat(input: ValueStream, transform: &FlatTransform) -> Result<ValueStream> {
    let mut lines = input.into_lines()?;
    let Some(head) = lines.first().cloned() else {
        return Ok(ValueStream::Empty);
    };
    let rest = if lines.len() > 1 {
        lines.split_off(1)
    } else {
        Vec::new()
    };
    let output = transform.apply(&head, &rest);
    Ok(if output.is_empty() {
        ValueStream::Empty
    } else {
        ValueStream::Lines(output)
    })
}

#[cfg(test)]
mod tests {
    use super::{ValueStream, car, cdr, flat};
    use crate::helpers::flat_lambda::parse_flat_lambda;

    #[test]
    fn car_returns_first_line() {
        let output = car(ValueStream::Text("a\nb\n".into())).expect("car");
        assert_eq!(output.to_text().expect("text"), "a\n");
    }

    #[test]
    fn cdr_returns_remaining_lines() {
        let output = cdr(ValueStream::Text("a\nb\nc\n".into())).expect("cdr");
        assert_eq!(output.to_text().expect("text"), "b\nc\n");
    }

    #[test]
    fn flat_splices_rest_into_the_output() {
        let transform = parse_flat_lambda(r#"\head, rest -> [head, "tail", rest]"#).expect("parse");
        let output = flat(ValueStream::Text("a\nb\nc\n".into()), &transform).expect("flat");
        assert_eq!(output.to_text().expect("text"), "a\ntail\nb\nc\n");
    }
}
