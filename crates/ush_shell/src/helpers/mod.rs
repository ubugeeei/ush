mod browser;
mod flat_lambda;
mod lambda;
mod lambda_syntax;
mod sequence;
#[cfg(test)]
mod tests;
mod value;
mod zip;

use anyhow::Result;
use serde_json::Value;

pub use value::ValueStream;

use self::flat_lambda::FlatTransform;
use self::lambda::{Predicate, Transform, apply_transform, parse_lambda_helper};
use self::sequence::{Field, SequenceOp, apply_sequence_op, parse_sequence_helper};
use self::zip::{ZipSource, parse_zip_helper};

#[derive(Debug, Clone)]
pub struct HelperInvocation {
    kind: HelperKind,
}

#[derive(Debug, Clone)]
enum HelperKind {
    Len,
    Lines,
    Json,
    Xml,
    Html,
    Sequence(SequenceOp),
    Map(Transform),
    Each(Transform),
    Filter(Predicate),
    Any(Predicate),
    Some(Predicate),
    Flat(FlatTransform),
    Field(Field),
    Zip(ZipSource),
}

impl HelperInvocation {
    pub fn parse(raw: &str) -> Option<Result<Self>> {
        let trimmed = raw.trim();
        let kind = match trimmed {
            "len" | "length" => Some(Ok(HelperKind::Len)),
            "lines" => Some(Ok(HelperKind::Lines)),
            "json" => Some(Ok(HelperKind::Json)),
            "xml" => Some(Ok(HelperKind::Xml)),
            "html" => Some(Ok(HelperKind::Html)),
            _ => parse_sequence_helper(trimmed)
                .or_else(|| parse_zip_helper(trimmed))
                .or_else(|| parse_lambda_helper(trimmed)),
        }?;
        Some(kind.map(|kind| Self { kind }))
    }

    pub fn execute(&self, input: ValueStream) -> Result<(ValueStream, i32)> {
        match &self.kind {
            HelperKind::Len => Ok((ValueStream::Text(format!("{}\n", stream_len(input))), 0)),
            HelperKind::Lines => Ok((ValueStream::Lines(input.into_lines()?), 0)),
            HelperKind::Json => {
                let text = input.to_text()?;
                match serde_json::from_str::<Value>(&text) {
                    Ok(json) => Ok((ValueStream::Json(json), 0)),
                    Err(_) => {
                        browser::open_in_browser(&ValueStream::Text(text))?;
                        Ok((ValueStream::Empty, 0))
                    }
                }
            }
            HelperKind::Xml => {
                let text = input.to_text()?;
                match browser::format_xml(&text) {
                    Ok(xml) => Ok((ValueStream::Text(xml), 0)),
                    Err(_) => {
                        browser::open_in_browser(&ValueStream::Text(text))?;
                        Ok((ValueStream::Empty, 0))
                    }
                }
            }
            HelperKind::Html => {
                browser::open_in_browser(&input)?;
                Ok((ValueStream::Empty, 0))
            }
            HelperKind::Sequence(op) => Ok((apply_sequence_op(input, op)?, 0)),
            HelperKind::Map(transform) | HelperKind::Each(transform) => {
                let output = input
                    .into_lines()?
                    .into_iter()
                    .map(|line| apply_transform(transform, &line))
                    .collect::<Vec<_>>();
                Ok((ValueStream::Lines(output), 0))
            }
            HelperKind::Filter(predicate) => {
                let output = input
                    .into_lines()?
                    .into_iter()
                    .filter(|line| predicate.matches(line))
                    .collect::<Vec<_>>();
                Ok((ValueStream::Lines(output), 0))
            }
            HelperKind::Any(predicate) | HelperKind::Some(predicate) => {
                let matched = input
                    .into_lines()?
                    .iter()
                    .any(|line| predicate.matches(line));
                Ok((
                    ValueStream::Text(format!("{matched}\n")),
                    if matched { 0 } else { 1 },
                ))
            }
            HelperKind::Flat(transform) => Ok((flat(input, transform)?, 0)),
            HelperKind::Field(field) => Ok((project_field(input, *field)?, 0)),
            HelperKind::Zip(source) => Ok((source.apply(input)?, 0)),
        }
    }
}

fn stream_len(input: ValueStream) -> usize {
    match input {
        ValueStream::Empty => 0,
        ValueStream::Text(text) => text.lines().count(),
        ValueStream::Lines(lines) => lines.len(),
        ValueStream::Json(value) => match value {
            Value::Array(values) => values.len(),
            Value::Object(map) => map.len(),
            Value::String(text) => text.chars().count(),
            Value::Null => 0,
            _ => 1,
        },
    }
}

fn flat(input: ValueStream, transform: &FlatTransform) -> Result<ValueStream> {
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

fn project_field(input: ValueStream, field: Field) -> Result<ValueStream> {
    let output = input
        .into_lines()?
        .into_iter()
        .map(|line| match line.split_once('\t') {
            Some((left, right)) => match field {
                Field::First => left.to_string(),
                Field::Second => right.to_string(),
            },
            None => match field {
                Field::First => line,
                Field::Second => String::new(),
            },
        })
        .collect::<Vec<_>>();
    Ok(if output.is_empty() {
        ValueStream::Empty
    } else {
        ValueStream::Lines(output)
    })
}
