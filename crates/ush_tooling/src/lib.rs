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

mod diagnostic;
mod format;
mod semantic;
mod token;

pub use self::{
    diagnostic::{UshDiagnostic, check_file, check_source},
    format::format_source,
    semantic::semantic_tokens,
    token::{SemanticToken, SemanticTokenKind, semantic_token_legend},
};
