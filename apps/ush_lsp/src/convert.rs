use std::collections::HashMap;

use lsp_types::{
    CompletionItem as LspCompletionItem, CompletionItemKind, Diagnostic, DiagnosticSeverity,
    DocumentHighlight, DocumentHighlightKind, DocumentSymbol as LspDocumentSymbol,
    FoldingRange as LspFoldingRange, FoldingRangeKind, Hover as LspHover, HoverContents,
    MarkupContent, MarkupKind, Position, Range, SemanticToken, SemanticTokens,
    SymbolKind as LspSymbolKind, TextEdit, Uri, WorkspaceEdit,
};
use ush_tooling::{
    CompletionItem as UshCompletionItem, CompletionKind as UshCompletionKind,
    DocumentSymbol as UshDocumentSymbol, FoldingRange as UshFoldingRange, Highlight, HighlightKind,
    Hover as UshHover, Reference, SemanticToken as UshToken, SymbolKind as UshSymbolKind,
    UshDiagnostic,
};

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

pub fn document_highlights(items: &[Highlight]) -> Vec<DocumentHighlight> {
    items
        .iter()
        .map(|item| DocumentHighlight {
            range: Range {
                start: Position::new(item.line, item.start),
                end: Position::new(item.line, item.start + item.length),
            },
            kind: Some(match item.kind {
                HighlightKind::Write => DocumentHighlightKind::WRITE,
                HighlightKind::Read => DocumentHighlightKind::READ,
                HighlightKind::Text => DocumentHighlightKind::TEXT,
            }),
        })
        .collect()
}

pub fn range_of_reference(reference: &Reference) -> Range {
    Range {
        start: Position::new(reference.line, reference.start),
        end: Position::new(reference.line, reference.start + reference.length),
    }
}

pub fn rename_workspace_edit(uri: &Uri, locations: &[Reference], new_name: &str) -> WorkspaceEdit {
    // lsp_types' `Uri` has interior mutability (cached parse state)
    // so `HashMap<Uri, _>` trips clippy's `mutable_key_type`. The
    // entry is correctness-safe here — we never mutate the `Uri`
    // after inserting — but suppress the lint at the use-site
    // rather than reaching into a different container.
    #[allow(clippy::mutable_key_type)]
    let mut changes: HashMap<Uri, Vec<TextEdit>> = HashMap::new();
    let edits: Vec<TextEdit> = locations
        .iter()
        .map(|reference| TextEdit {
            range: range_of_reference(reference),
            new_text: new_name.to_string(),
        })
        .collect();
    changes.insert(uri.clone(), edits);
    WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    }
}

pub fn hover(item: UshHover) -> LspHover {
    LspHover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: item.contents,
        }),
        range: Some(Range {
            start: Position::new(item.line, item.start),
            end: Position::new(item.line, item.start + item.length),
        }),
    }
}

pub fn folding_ranges(items: &[UshFoldingRange]) -> Vec<LspFoldingRange> {
    items
        .iter()
        .map(|item| LspFoldingRange {
            start_line: item.start_line,
            start_character: None,
            end_line: item.end_line,
            end_character: None,
            kind: Some(FoldingRangeKind::Region),
            collapsed_text: None,
        })
        .collect()
}

pub fn completion_items(items: &[UshCompletionItem]) -> Vec<LspCompletionItem> {
    items
        .iter()
        .map(|item| LspCompletionItem {
            label: item.label.clone(),
            kind: Some(match item.kind {
                UshCompletionKind::Keyword => CompletionItemKind::KEYWORD,
                UshCompletionKind::Variable => CompletionItemKind::VARIABLE,
                UshCompletionKind::Function => CompletionItemKind::FUNCTION,
                UshCompletionKind::Type => CompletionItemKind::CLASS,
            }),
            detail: if item.detail.is_empty() {
                None
            } else {
                Some(item.detail.clone())
            },
            ..LspCompletionItem::default()
        })
        .collect()
}

pub fn document_symbols(items: &[UshDocumentSymbol]) -> Vec<LspDocumentSymbol> {
    items
        .iter()
        .map(|item| {
            let range = Range {
                start: Position::new(item.line, item.start),
                end: Position::new(item.line, item.start + item.length),
            };
            #[allow(deprecated)]
            LspDocumentSymbol {
                name: item.name.clone(),
                detail: None,
                kind: match item.kind {
                    UshSymbolKind::Function => LspSymbolKind::FUNCTION,
                    UshSymbolKind::Enum => LspSymbolKind::ENUM,
                    UshSymbolKind::Trait => LspSymbolKind::INTERFACE,
                    UshSymbolKind::Type => LspSymbolKind::STRUCT,
                    UshSymbolKind::Variable => LspSymbolKind::VARIABLE,
                    UshSymbolKind::Alias => LspSymbolKind::CONSTANT,
                },
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: None,
            }
        })
        .collect()
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
