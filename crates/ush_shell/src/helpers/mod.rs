mod browser;
mod lambda;
mod lambda_syntax;
mod value;

use anyhow::Result;
use serde_json::Value;

pub use value::ValueStream;

use self::lambda::{Predicate, Transform, apply_transform, parse_lambda_helper};

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
            "len" | "length" => Some(Ok(HelperKind::Len)),
            "lines" => Some(Ok(HelperKind::Lines)),
            "json" => Some(Ok(HelperKind::Json)),
            "xml" => Some(Ok(HelperKind::Xml)),
            "html" => Some(Ok(HelperKind::Html)),
            _ => parse_lambda_helper(trimmed),
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
    fn len_counts_lines() {
        let helper = HelperInvocation::parse("len")
            .expect("helper")
            .expect("parse");
        let (output, _) = helper
            .execute(ValueStream::Text("a\nb\n".to_string()))
            .expect("execute");
        assert_eq!(output.to_text().expect("text"), "2\n");
    }

    #[test]
    fn length_remains_as_compat_alias() {
        let helper = HelperInvocation::parse("length")
            .expect("helper")
            .expect("parse");
        let (output, _) = helper
            .execute(ValueStream::Text("a\nb\n".to_string()))
            .expect("execute");
        assert_eq!(output.to_text().expect("text"), "2\n");
    }

    #[test]
    fn html_helper_is_recognized() {
        let helper = HelperInvocation::parse("html")
            .expect("helper")
            .expect("parse");
        assert!(matches!(helper.kind, super::HelperKind::Html));
    }

    #[test]
    fn xml_helper_is_recognized() {
        let helper = HelperInvocation::parse("xml")
            .expect("helper")
            .expect("parse");
        assert!(matches!(helper.kind, super::HelperKind::Xml));
    }

    #[test]
    fn backslash_lambda_supports_custom_arg_names_and_block_bodies() {
        let helper = HelperInvocation::parse(r#"map(\line -> { upper(line) })"#)
            .expect("helper")
            .expect("parse");
        let (output, _) = helper
            .execute(ValueStream::Text("ush\n".to_string()))
            .expect("execute");
        assert_eq!(output.to_text().expect("text"), "USH\n");
    }

    #[test]
    fn zero_arg_block_lambda_can_produce_constants() {
        let helper = HelperInvocation::parse(r#"map(\-> { "ok" })"#)
            .expect("helper")
            .expect("parse");
        let (output, _) = helper
            .execute(ValueStream::Text("a\nb\n".to_string()))
            .expect("execute");
        assert_eq!(output.to_text().expect("text"), "ok\nok\n");
    }
}
