use lsp_types::{
    Diagnostic, DiagnosticSeverity, Position, Range, SemanticToken, SemanticTokens, TextEdit,
};
use ush_tooling::{SemanticToken as UshToken, UshDiagnostic};

pub fn diagnostics(source: &str, items: &[UshDiagnostic]) -> Vec<Diagnostic> {
    items
        .iter()
        .map(|item| Diagnostic {
            range: line_range(source, item.line),
            severity: Some(DiagnosticSeverity::ERROR),
            message: item.message.clone(),
            ..Diagnostic::default()
        })
        .collect()
}

pub fn full_document_edit(source: &str, formatted: &str) -> TextEdit {
    TextEdit {
        range: Range {
            start: Position::new(0, 0),
            end: end_position(source),
        },
        new_text: formatted.to_string(),
    }
}

pub fn semantic_tokens(items: &[UshToken]) -> SemanticTokens {
    let mut prev_line = 0u32;
    let mut prev_start = 0u32;
    let mut data = Vec::new();

    for item in items {
        let delta_line = item.line - prev_line;
        let delta_start = if delta_line == 0 {
            item.start - prev_start
        } else {
            item.start
        };
        data.push(SemanticToken {
            delta_line,
            delta_start,
            length: item.length,
            token_type: item.kind.index(),
            token_modifiers_bitset: 0,
        });
        prev_line = item.line;
        prev_start = item.start;
    }

    SemanticTokens {
        result_id: None,
        data,
    }
}

fn line_range(source: &str, line: usize) -> Range {
    let text = source.lines().nth(line).unwrap_or("");
    Range {
        start: Position::new(line as u32, 0),
        end: Position::new(line as u32, text.chars().count() as u32),
    }
}

fn end_position(source: &str) -> Position {
    let mut line = 0u32;
    let mut col = 0u32;
    for ch in source.chars() {
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    Position::new(line, col)
}

#[cfg(test)]
mod tests {
    use super::{diagnostics, full_document_edit};

    #[test]
    fn creates_a_full_document_edit() {
        let edit = full_document_edit("print \"a\"\n", "print \"b\"\n");
        assert_eq!(edit.range.end.line, 1);
        assert_eq!(edit.new_text, "print \"b\"\n");
    }

    #[test]
    fn maps_diagnostics_to_ranges() {
        let diagnostics = diagnostics(
            "print \"ok\"\nprint missing\n",
            &[ush_tooling::UshDiagnostic {
                line: 1,
                message: "unknown variable".to_string(),
            }],
        );

        assert_eq!(diagnostics[0].range.start.line, 1);
    }
}
