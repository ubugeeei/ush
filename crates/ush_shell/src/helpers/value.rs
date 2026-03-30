use anyhow::Result;
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub enum ValueStream {
    #[default]
    Empty,
    Text(String),
    Lines(Vec<String>),
    Json(Value),
}

impl ValueStream {
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    pub fn to_text(&self) -> Result<String> {
        match self {
            Self::Empty => Ok(String::new()),
            Self::Text(text) => Ok(text.clone()),
            Self::Lines(lines) => Ok(lines.join("\n") + if lines.is_empty() { "" } else { "\n" }),
            Self::Json(value) => Ok(serde_json::to_string_pretty(value)? + "\n"),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.to_text()?.into_bytes())
    }

    pub fn into_lines(self) -> Result<Vec<String>> {
        match self {
            Self::Empty => Ok(Vec::new()),
            Self::Text(text) => Ok(text.lines().map(ToString::to_string).collect()),
            Self::Lines(lines) => Ok(lines),
            Self::Json(value) => match value {
                Value::Array(items) => Ok(items
                    .into_iter()
                    .map(|item| match item {
                        Value::String(value) => value,
                        other => other.to_string(),
                    })
                    .collect()),
                other => Ok(vec![other.to_string()]),
            },
        }
    }
}
