# ush_lsp

`ush_lsp` is a stdio Language Server Protocol server for `.ush` files.
Release archives and the install script place it next to `ush`, so an editor can
usually invoke `ush_lsp` directly after installation.

## Capabilities

- full-document sync
- diagnostics from `ush check`
- document formatting
- semantic tokens

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
