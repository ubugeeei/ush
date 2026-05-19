//! Editor-side helpers for `.ush` files.
//!
//! Three independent passes that the LSP, the `ush` CLI subcommands
//! (`ush format`, `ush check`), and the formatter dogfood gate
//! consume:
//!
//! - [`format_source`] — the formatter (idempotent rewriter).
//! - [`check_file`] / [`check_source`] — diagnostic extraction
//!   for `ush check` and LSP `textDocument/publishDiagnostics`.
//! - [`semantic_tokens`] + [`semantic_token_legend`] — the
//!   `textDocument/semanticTokens` payload.

mod completion;
mod diagnostic;
mod folding;
mod format;
mod highlight;
mod semantic;
mod symbol;
mod token;

pub use self::{
    completion::{CompletionItem, CompletionKind, completions},
    diagnostic::{UshDiagnostic, check_file, check_source},
    folding::{FoldingRange, folding_ranges},
    format::format_source,
    highlight::{Highlight, HighlightKind, document_highlights},
    semantic::semantic_tokens,
    symbol::{DocumentSymbol, SymbolKind, document_symbols},
    token::{SemanticToken, SemanticTokenKind, semantic_token_legend},
};
