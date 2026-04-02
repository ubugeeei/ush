use std::{fs, path::Path};

use anyhow::{Context, Result};
use ush_compiler::UshCompiler;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UshDiagnostic {
    pub line: usize,
    pub message: String,
}

pub fn check_file(path: &Path) -> Result<Vec<UshDiagnostic>> {
    let source =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(check_source(&source))
}

pub fn check_source(source: &str) -> Vec<UshDiagnostic> {
    match UshCompiler.compile_source(source) {
        Ok(_) => Vec::new(),
        Err(error) => vec![parse_diagnostic(&error.to_string())],
    }
}

fn parse_diagnostic(message: &str) -> UshDiagnostic {
    let trimmed = message.trim();
    let line = trimmed
        .split("line ")
        .nth(1)
        .and_then(|rest| rest.split(':').next())
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(1)
        .saturating_sub(1);
    let normalized = trimmed
        .split_once(": ")
        .map_or(trimmed, |(_, detail)| detail)
        .to_string();
    UshDiagnostic {
        line,
        message: normalized,
    }
}

#[cfg(test)]
mod tests {
    use super::check_source;

    #[test]
    fn returns_empty_diagnostics_for_valid_program() {
        assert!(check_source("print \"ok\"").is_empty());
    }

    #[test]
    fn extracts_line_information_from_compile_errors() {
        let diagnostics = check_source("let value = missing.await");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].line, 0);
        assert!(diagnostics[0].message.contains("missing"));
    }
}
