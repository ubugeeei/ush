mod lambda;
mod value;

use anyhow::{Result, anyhow};
use serde_json::Value;

pub use value::ValueStream;

use self::lambda::{Predicate, Transform, apply_transform, parse_lambda_helper};

#[derive(Debug, Clone)]
pub struct HelperInvocation {
    kind: HelperKind,
}

#[derive(Debug, Clone)]
enum HelperKind {
    Length,
    Lines,
    Json,
    Map(Transform),
    Each(Transform),
    Filter(Predicate),
    Any(Predicate),
    Some(Predicate),
}

impl HelperInvocation {
    pub fn parse(raw: &str) -> Option<Result<Self>> {
        let trimmed = raw.trim();
        let kind = match trimmed {
            "length" => Some(Ok(HelperKind::Length)),
            "lines" => Some(Ok(HelperKind::Lines)),
            "json" => Some(Ok(HelperKind::Json)),
            _ => parse_lambda_helper(trimmed),
        }?;
        Some(kind.map(|kind| Self { kind }))
    }

    pub fn execute(&self, input: ValueStream) -> Result<(ValueStream, i32)> {
        match &self.kind {
            HelperKind::Length => Ok((ValueStream::Text(format!("{}\n", stream_len(input))), 0)),
            HelperKind::Lines => Ok((ValueStream::Lines(input.into_lines()?), 0)),
            HelperKind::Json => {
                let text = input.to_text()?;
                let json = serde_json::from_str::<Value>(&text)
                    .map_err(|source| anyhow!("failed to parse json from pipeline: {source}"))?;
                Ok((ValueStream::Json(json), 0))
            }
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

#[cfg(test)]
mod tests {
    use super::{HelperInvocation, ValueStream};

    #[test]
    fn length_counts_lines() {
        let helper = HelperInvocation::parse("length")
            .expect("helper")
            .expect("parse");
        let (output, _) = helper
            .execute(ValueStream::Text("a\nb\n".to_string()))
            .expect("execute");
        assert_eq!(output.to_text().expect("text"), "2\n");
    }
}
