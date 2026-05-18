# Contributing to `ush`

Thanks for your interest in `ush`. The project is still pre-1.0 and
moves quickly; that means small, focused contributions are easier to
land than sweeping refactors.

## Quick start

```bash
git clone https://github.com/ubugeeei/ush
cd ush
cargo build                     # builds the workspace
cargo test --workspace          # runs the full suite
```

The workspace targets stable Rust. The current MSRV (minimum
supported Rust version) is declared in `Cargo.toml`'s
`[workspace.package]` as `rust-version`; it is bumped only in minor
releases.

## Run the same checks CI runs

Before pushing, please run the same gates that CI runs. They are
intentionally cheap:

```bash
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --no-deps -- -D warnings
cargo test --locked --workspace
cargo test --locked --workspace --release
cargo check --locked -p ush_compiler --no-default-features   # no_std core
sh scripts/check_rs_line_limit.sh                            # 250-line file cap
sh scripts/test_install.sh                                   # installer flow
```

Any of these failing in CI is a hard block. The line-limit script
caps individual Rust files at 250 lines (sources only, tests inside a
file are counted); split larger files into modules.

## Workspace layout

- `apps/ush` — CLI binary
- `apps/ush_lsp` — stdio LSP server
- `crates/ush_config` — config loading
- `crates/ush_compiler` — `.ush → sh` compiler (`no_std + alloc` capable)
- `crates/ush_shell` — interactive shell, parser, stylish I/O
- `crates/ush_tooling` — formatter, diagnostics, semantic tokens
- `vendor/rustyline` — vendored line-editor; do not edit in random
  PRs (see `vendor/rustyline/`'s upstream-sync notes)

## Filing an issue

Use the templates under `.github/ISSUE_TEMPLATE/` so the maintainer
can triage without a back-and-forth. For anything security-related,
follow [SECURITY.md](./SECURITY.md) instead of opening a public
issue.

## Opening a pull request

- Match the existing style: short summary, notes (any subtlety the
  reviewer needs to know), and a validation list (the commands you
  ran). `.github/PULL_REQUEST_TEMPLATE.md` pre-fills this.
- One topic per PR. If you noticed unrelated cleanups while you were
  there, please open a follow-up.
- Keep `Cargo.lock` up to date — CI uses `--locked` everywhere.
- New behaviour needs a test or an explicit "why a test would be
  too expensive" note.

## Commits

- Use the imperative present tense ("add", "fix", "refactor").
- Reference issues with `Closes #NN` or `Refs #NN`.
- Don't squash unrelated changes into one commit.
