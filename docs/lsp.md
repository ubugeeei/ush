# ush_lsp

`ush_lsp` is a stdio Language Server Protocol server for `.ush` files.
Release archives and the install script place it next to `ush`, so an editor can
usually invoke `ush_lsp` directly after installation.

## Capabilities

Implemented LSP methods, with the engine that backs each one
living in [`crates/ush_tooling`](../crates/ush_tooling):

| LSP method | Provides | Engine |
| --- | --- | --- |
| `textDocument/didOpen` / `didChange` / `didSave` | Full-document sync; triggers `publishDiagnostics`. | – |
| `textDocument/publishDiagnostics` | Compile errors from `ush check`. | `diagnostic::check_source` |
| `textDocument/formatting` | In-place format; idempotent rewriter. | `format::format_source` |
| `textDocument/semanticTokens/full` | Syntax highlighting (keyword / string / number / comment / variable / function / type / property / operator / decorator). | `semantic::semantic_tokens` |
| `textDocument/documentHighlight` | Every occurrence of the identifier under the cursor. | `highlight::document_highlights` |
| `textDocument/documentSymbol` | Outline of top-level `fn` / `enum` / `trait` / `type` / `let` / `alias`. | `symbol::document_symbols` |
| `textDocument/foldingRange` | `{ … }` block folding, ignoring braces inside strings / comments / `#[…]` attributes. | `folding::folding_ranges` |
| `textDocument/hover` | Markdown tooltip: keyword help, or "role + declaring line" for an identifier. | `hover::hover` |
| `textDocument/completion` | Every `.ush` keyword + every identifier the semantic tokenizer has classified in the open document. | `completion::completions` |
| `textDocument/definition` | First occurrence of the identifier under the cursor (no scope analysis yet). | `references::definition` |
| `textDocument/references` | Every occurrence of the identifier under the cursor. | `references::references` |
| `textDocument/prepareRename` | Range to highlight before the rename popup. | `references::prepare_rename` |
| `textDocument/rename` | `WorkspaceEdit` with one `TextEdit` per occurrence. Rejects new names that are not valid `.ush` identifiers with an LSP error. | `references::rename_locations` |

Not yet implemented: `signatureHelp`, `codeAction`, `inlayHint`,
`callHierarchy`, `workspaceSymbol`. Most of these will land once
the compiler exposes typed information; the current providers are
all syntactic so they cannot answer "what type is this" without
re-running the effects pass.

## Invocation

For an installed release:

```bash
ush_lsp
```

From a source checkout:

```bash
cargo run -p ush_lsp
```

The server speaks LSP over stdin/stdout. Do not wrap it in `ush -c` or another
shell command that writes prompts or extra output to stdout.

## Neovim Example

```lua
vim.lsp.start({
  name = "ush",
  cmd = { "ush_lsp" },
  root_dir = vim.fs.root(0, { "Cargo.toml", ".git" }),
})
```

## VS Code-Compatible Clients

Configure a stdio language server command:

```json
{
  "command": "ush_lsp",
  "args": [],
  "filetypes": ["ush"]
}
```

If you installed from source and have not copied the binary onto `PATH`, use the
absolute path to `target/release/ush_lsp` or run it through your editor's normal
Cargo task integration.
