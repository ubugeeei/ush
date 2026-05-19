//! LSP `textDocument/documentHighlight` provider.
//!
//! Given a cursor position (zero-based line + character column),
//! returns every occurrence of the same identifier in the document
//! so the editor can underline / box them. We piggy-back on the
//! semantic-token tokenizer so the definition of "an identifier"
//! lines up with what is already highlighted as a variable /
//! function / type / property.

use crate::semantic::semantic_tokens;
use crate::token::{SemanticToken, SemanticTokenKind};

/// Where in the document a highlight should be drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Highlight {
    pub line: u32,
    pub start: u32,
    pub length: u32,
    pub kind: HighlightKind,
}

/// Matches the three roles LSP's `DocumentHighlightKind` distinguishes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightKind {
    /// The token that produces a binding (e.g. the `name` in `let name = …`).
    Write,
    /// A subsequent read of that binding.
    Read,
    /// Same name but a different category (function call, type ref, …).
    Text,
}

/// Token kinds we are willing to highlight occurrences for. Strings,
/// numbers, comments, keywords, and decorators do not get
/// "occurrence" highlighting because that is the job of the syntax
/// highlighter, not of `documentHighlight`.
fn is_identifier_like(kind: SemanticTokenKind) -> bool {
    matches!(
        kind,
        SemanticTokenKind::Variable
            | SemanticTokenKind::Function
            | SemanticTokenKind::Type
            | SemanticTokenKind::Property
    )
}

/// Compute the highlights for the identifier under the cursor.
///
/// Returns an empty `Vec` when the cursor is not on an identifier or
/// the identifier appears only once.
pub fn document_highlights(source: &str, line: u32, character: u32) -> Vec<Highlight> {
    let tokens = semantic_tokens(source);
    let cursor = match cursor_token(&tokens, line, character) {
        Some(token) => token,
        None => return Vec::new(),
    };
    if !is_identifier_like(cursor.kind) {
        return Vec::new();
    }
    let lines: Vec<&str> = source.lines().collect();
    let needle = token_text(&lines, cursor);
    if needle.is_empty() {
        return Vec::new();
    }

    tokens
        .iter()
        .filter(|token| is_identifier_like(token.kind))
        .filter(|token| token_text(&lines, **token) == needle)
        .map(|token| Highlight {
            line: token.line,
            start: token.start,
            length: token.length,
            kind: classify(*token, cursor),
        })
        .collect()
}

fn cursor_token(tokens: &[SemanticToken], line: u32, character: u32) -> Option<SemanticToken> {
    tokens
        .iter()
        .find(|token| {
            token.line == line && token.start <= character && character < token.start + token.length
        })
        .copied()
}

fn token_text(lines: &[&str], token: SemanticToken) -> String {
    lines
        .get(token.line as usize)
        .and_then(|line| {
            let start = token.start as usize;
            let end = start + token.length as usize;
            line.get(start..end).map(str::to_string)
        })
        .unwrap_or_default()
}

fn classify(token: SemanticToken, cursor: SemanticToken) -> HighlightKind {
    if token == cursor {
        // The token under the cursor itself is described as Text;
        // the editor will use a neutral background.
        return HighlightKind::Text;
    }
    match token.kind {
        SemanticTokenKind::Variable => HighlightKind::Read,
        SemanticTokenKind::Function | SemanticTokenKind::Type | SemanticTokenKind::Property => {
            HighlightKind::Text
        }
        _ => HighlightKind::Text,
    }
}

#[cfg(test)]
mod tests {
    use super::{HighlightKind, document_highlights};

    #[test]
    fn highlights_repeated_variable() {
        let source = "let greeting = \"hi\"\nprint greeting\nprint greeting\n";
        // cursor on the `greeting` in `let greeting`
        let highlights = document_highlights(source, 0, 5);
        assert_eq!(highlights.len(), 3);
        for h in &highlights {
            assert_eq!(h.length, "greeting".len() as u32);
        }
        assert!(
            highlights
                .iter()
                .any(|h| h.kind == HighlightKind::Text && h.line == 0)
        );
    }

    #[test]
    fn keyword_does_not_highlight() {
        let source = "let a = 1\nlet b = 2\n";
        let highlights = document_highlights(source, 0, 0); // cursor on `let`
        assert!(
            highlights.is_empty(),
            "keyword `let` must not produce highlights, got: {highlights:?}",
        );
    }

    #[test]
    fn cursor_in_whitespace_returns_nothing() {
        let source = "let x = 1\n";
        let highlights = document_highlights(source, 0, 3); // between `let` and `x`
        assert!(highlights.is_empty());
    }

    #[test]
    fn single_occurrence_still_highlights_itself() {
        let source = "let only_one = 1\n";
        let highlights = document_highlights(source, 0, 4);
        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].kind, HighlightKind::Text);
    }
}
