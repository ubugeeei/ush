//! Shared engine for `textDocument/references`, `textDocument/definition`,
//! and `textDocument/rename`.
//!
//! All three queries answer essentially the same question — "what
//! other occurrences of this identifier are in the file" — and rely
//! on the semantic-token tokenizer to decide what counts as an
//! identifier. There is no scope analysis: a `let x` inside a
//! function and a `let x` at top level look like the same name to
//! these queries. That is in line with the
//! [`document_highlights`](crate::document_highlights) provider,
//! and matches the level of precision callers can expect from a
//! pre-1.0 LSP.

use crate::semantic::semantic_tokens;
use crate::token::{SemanticToken, SemanticTokenKind};

/// One occurrence in the source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reference {
    pub line: u32,
    pub start: u32,
    pub length: u32,
}

/// Returns every identifier occurrence (including the one at the
/// cursor). When the cursor is not on an identifier, returns empty.
pub fn references(source: &str, line: u32, character: u32) -> Vec<Reference> {
    let tokens = semantic_tokens(source);
    let cursor = match cursor_token(&tokens, line, character) {
        Some(token) if is_identifier(token.kind) => token,
        _ => return Vec::new(),
    };
    let lines: Vec<&str> = source.lines().collect();
    let needle = token_text(&lines, cursor);
    if needle.is_empty() {
        return Vec::new();
    }
    tokens
        .iter()
        .filter(|t| is_identifier(t.kind))
        .filter(|t| token_text(&lines, **t) == needle)
        .map(|t| Reference {
            line: t.line,
            start: t.start,
            length: t.length,
        })
        .collect()
}

/// Returns the first occurrence of the identifier under the cursor.
/// Used for `textDocument/definition` — pre-1.0 the project does not
/// track scoped declarations, so "definition" is "first sighting".
pub fn definition(source: &str, line: u32, character: u32) -> Option<Reference> {
    references(source, line, character).into_iter().next()
}

/// Returns the same set as [`references`] for use when renaming.
/// The LSP layer is responsible for applying the new name to each
/// returned range. Returns [`Err`] if the new name is not a valid
/// `.ush` identifier; the LSP layer can surface that as a normal
/// rename error.
pub fn rename_locations(
    source: &str,
    line: u32,
    character: u32,
    new_name: &str,
) -> Result<Vec<Reference>, RenameError> {
    if !is_valid_identifier(new_name) {
        return Err(RenameError::InvalidName);
    }
    Ok(references(source, line, character))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenameError {
    InvalidName,
}

/// Validate the cursor for a rename, before the user types a new
/// name. Returns the range of the identifier under the cursor when
/// a rename is possible, or `None` when the position is not on a
/// renameable identifier (keyword / whitespace / string / comment).
///
/// LSP clients call this for `textDocument/prepareRename`; the
/// editor uses the returned range to highlight what's about to be
/// renamed and pre-fills the rename popup.
pub fn prepare_rename(source: &str, line: u32, character: u32) -> Option<Reference> {
    let tokens = semantic_tokens(source);
    let cursor = cursor_token(&tokens, line, character)?;
    if !is_identifier(cursor.kind) {
        return None;
    }
    Some(Reference {
        line: cursor.line,
        start: cursor.start,
        length: cursor.length,
    })
}

fn is_identifier(kind: SemanticTokenKind) -> bool {
    matches!(
        kind,
        SemanticTokenKind::Variable
            | SemanticTokenKind::Function
            | SemanticTokenKind::Type
            | SemanticTokenKind::Property
    )
}

fn is_valid_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) if first == '_' || first.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
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

#[cfg(test)]
mod tests {
    use super::{RenameError, definition, prepare_rename, references, rename_locations};

    #[test]
    fn prepare_rename_returns_identifier_range_under_cursor() {
        let source = "let value = 1\n";
        let range = prepare_rename(source, 0, 5).expect("rename `value`");
        assert_eq!(range.line, 0);
        assert_eq!(range.start, 4);
        assert_eq!(range.length, "value".len() as u32);
    }

    #[test]
    fn prepare_rename_returns_none_on_keyword() {
        // Cursor on `let`.
        assert!(prepare_rename("let x = 1\n", 0, 0).is_none());
    }

    #[test]
    fn references_return_every_occurrence_of_a_variable() {
        let source = "let v = 1\nprint v\nprint v\n";
        let refs = references(source, 0, 4);
        assert_eq!(refs.len(), 3);
    }

    #[test]
    fn definition_points_to_the_first_occurrence() {
        let source = "let v = 1\nprint v\n";
        let def = definition(source, 1, 6).expect("v has a definition");
        assert_eq!(def.line, 0);
    }

    #[test]
    fn references_on_keyword_return_nothing() {
        let source = "let a = 1\n";
        assert!(references(source, 0, 0).is_empty());
    }

    #[test]
    fn rename_returns_every_occurrence_when_new_name_is_valid() {
        let source = "let v = 1\nprint v\n";
        let locations = rename_locations(source, 0, 4, "value").expect("rename `v` to `value`");
        assert_eq!(locations.len(), 2);
    }

    #[test]
    fn rename_rejects_invalid_identifier() {
        let source = "let v = 1\n";
        let err = rename_locations(source, 0, 4, "1bad").unwrap_err();
        assert_eq!(err, RenameError::InvalidName);

        let err = rename_locations(source, 0, 4, "with space").unwrap_err();
        assert_eq!(err, RenameError::InvalidName);
    }
}
