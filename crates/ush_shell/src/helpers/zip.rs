use anyhow::{Result, anyhow, bail};

use super::HelperKind;
use super::ValueStream;
use super::lambda_syntax::{parse_list_literal, parse_string_literal};

#[derive(Debug, Clone)]
pub(super) struct ZipSource {
    lines: Vec<String>,
}

impl ZipSource {
    pub(super) fn apply(&self, input: ValueStream) -> Result<ValueStream> {
        let output = input
            .into_lines()?
            .into_iter()
            .zip(self.lines.iter())
            .map(|(left, right)| format!("{left}\t{right}"))
            .collect::<Vec<_>>();
        Ok(if output.is_empty() {
            ValueStream::Empty
        } else {
            ValueStream::Lines(output)
        })
    }
}

pub(super) fn parse_zip_helper(raw: &str) -> Option<Result<HelperKind>> {
    let open = raw.find('(')?;
    let close = raw.rfind(')')?;
    if close <= open {
        return Some(Err(anyhow!("invalid helper invocation: {raw}")));
    }

    let name = raw[..open].trim();
    if name != "fzip" {
        return None;
    }
    let inner = raw[open + 1..close].trim();
    Some(parse_zip_source(inner).map(HelperKind::Zip))
}

fn parse_zip_source(source: &str) -> Result<ZipSource> {
    if let Some(text) = parse_string_literal(source) {
        return Ok(ZipSource {
            lines: text.lines().map(ToString::to_string).collect(),
        });
    }

    let items = parse_list_literal(source)?
        .into_iter()
        .map(|item| {
            parse_string_literal(&item)
                .ok_or_else(|| anyhow!("fzip list items must be string literals, found {item}"))
        })
        .collect::<Result<Vec<_>>>()?;
    if items.is_empty() {
        bail!("fzip requires at least one right-hand value");
    }
    Ok(ZipSource { lines: items })
}

#[cfg(test)]
mod tests {
    use super::parse_zip_helper;
    use crate::helpers::ValueStream;

    #[test]
    fn zips_streams_from_list_literals() {
        let helper = parse_zip_helper(r#"fzip(["1", "2"])"#)
            .expect("helper")
            .expect("parse");
        let super::HelperKind::Zip(source) = helper else {
            panic!("expected zip helper");
        };
        let output = source
            .apply(ValueStream::Text("a\nb\n".into()))
            .expect("apply");
        assert_eq!(output.to_text().expect("text"), "a\t1\nb\t2\n");
    }
}
