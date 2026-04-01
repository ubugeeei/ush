use anyhow::{Result, anyhow, bail};

use super::lambda_syntax::{block_body, parse_list_literal, parse_string_literal};

#[derive(Debug, Clone)]
pub(super) enum FlatPart {
    Head,
    Rest,
    Literal(String),
}

#[derive(Debug, Clone)]
pub(super) struct FlatTransform {
    parts: Vec<FlatPart>,
}

impl FlatTransform {
    pub(super) fn apply(&self, head: &str, rest: &[String]) -> Vec<String> {
        let mut output = Vec::new();
        for part in &self.parts {
            match part {
                FlatPart::Head => output.push(head.to_string()),
                FlatPart::Rest => output.extend(rest.iter().cloned()),
                FlatPart::Literal(value) => output.push(value.clone()),
            }
        }
        output
    }
}

pub(super) fn parse_flat_lambda(source: &str) -> Result<FlatTransform> {
    let (signature, body) = source
        .split_once("->")
        .ok_or_else(|| anyhow!("expected `\\head, rest -> [...]` lambda"))?;
    let args = parse_args(signature)?;
    let parts = parse_list_literal(block_body(body.trim()))?
        .into_iter()
        .map(|item| parse_part(&item, args[0], args[1]))
        .collect::<Result<Vec<_>>>()?;
    Ok(FlatTransform { parts })
}

fn parse_args(signature: &str) -> Result<[&str; 2]> {
    let args = signature
        .trim()
        .strip_prefix('\\')
        .ok_or_else(|| anyhow!("expected helper lambda like `\\head, rest -> [...]`"))?
        .split(',')
        .map(str::trim)
        .filter(|arg| !arg.is_empty())
        .collect::<Vec<_>>();
    if args.len() != 2 {
        bail!("flat helper lambdas require exactly two arguments");
    }
    Ok([args[0], args[1]])
}

fn parse_part(source: &str, head: &str, rest: &str) -> Result<FlatPart> {
    if source == head {
        return Ok(FlatPart::Head);
    }
    if source == rest {
        return Ok(FlatPart::Rest);
    }
    if let Some(value) = parse_string_literal(source) {
        return Ok(FlatPart::Literal(value));
    }
    bail!("unsupported flat lambda item: {source}")
}

#[cfg(test)]
mod tests {
    use super::parse_flat_lambda;

    #[test]
    fn parses_head_rest_list_lambda() {
        let transform = parse_flat_lambda(r#"\head, rest -> [head, rest, "done"]"#).expect("parse");
        assert_eq!(
            transform.apply("a", &["b".into(), "c".into()]),
            vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "done".to_string()
            ]
        );
    }

    #[test]
    fn parses_block_bodies() {
        let transform = parse_flat_lambda(r#"\head, rest -> { [rest, head] }"#).expect("parse");
        assert_eq!(
            transform.apply("a", &["b".into(), "c".into()]),
            vec!["b".to_string(), "c".to_string(), "a".to_string()]
        );
    }
}
