# Changelog

All notable changes to `ush` will be documented in this file.

The format is based on [Keep a Changelog][keepachangelog], and this
project adheres to [Semantic Versioning][semver].

[keepachangelog]: https://keepachangelog.com/en/1.1.0/
[semver]: https://semver.org/spec/v2.0.0.html

## [Unreleased]

## [0.7.0] — 2026-05-19

A production-readiness release. Everything below was previously
tracked under `[Unreleased]`.

### Added

- Started this changelog. Older releases are summarised below from the
  GitHub release notes; future releases should land their entries here
  first under `[Unreleased]` and then be cut into a version section.
- `SECURITY.md` with a private-disclosure policy.
- `CONTRIBUTING.md` describing the local-CI flow and project layout,
  plus a `scripts/preflight.sh` one-command CI mirror.
- Issue and pull-request templates under `.github/`, plus a
  `CODEOWNERS` file.
- `docs/release-process.md` documenting the pre-flight checklist,
  the matrix of published artefacts, and the rollback procedure.
- Root-level `.editorconfig` pins charset / EOL / indent.
- `Shellcheck` CI workflow lints `install.sh` and every
  `scripts/*.sh`.
- `Benchmark` CI job tracks parser and full `.ush → sh` compile-time
  benchmarks against the `main` baseline (on `gh-pages`) and fails
  PRs on >25% regression.
- `Deny` CI job runs `cargo-deny` for license / source / advisory /
  bans enforcement.
- `CodeQL` and `Gitleaks` workflows add SAST and secret-scanning to
  every PR and to a weekly cron.
- Compiler now enforces `match` exhaustiveness during the effects
  pass: missing variants on enums, missing arms on `Bool` and `Unit`,
  and literal-only matches on `String` / `Int` / tuples / lists are
  now rejected with a clear diagnostic instead of compiling to a
  silently-fall-through shell branch.
- New CI gate: every `examples/*.ush` is type-checked through
  `ush check` *and* `ush format --check`.
- README links the CI / Shellcheck / Dependencies workflow badges
  and the MIT shield.
- `.dockerignore` keeps the docker build context lean; the docker
  image now runs as the non-root `ush` user (uid 1000).
- Linux `aarch64` release archive (`ush-aarch64-unknown-linux-gnu.tar.gz`).
- `ush_lsp` gains `--version` / `-V` and `--help` / `-h` flags.
- `ush` gains a user-facing panic hook that prints a triage-ready
  message instead of a raw rustc panic.

### Changed

- Workspace declares `rust-version = "1.88"` as the MSRV (bumped
  during the 0.7.0 cycle to match what `criterion` and `home`
  require).
- CI matrixes `clippy` and `test` jobs across Ubuntu and macOS, adds
  an MSRV gate, runs `cargo audit` and `cargo deny`, gates rustdoc
  on warnings, and runs the `no_std` test suite (not just `cargo
  check`).
- Workflow `concurrency.group` is now keyed on `run_id` for `main`
  pushes so successive merges don't cancel each other.
- Shell signal helpers use `sigaction(2)` instead of `signal(3)`, and
  the SIGCONT pgid path rejects PIDs that do not fit `pid_t`.
- The compiler refuses to silently fall back when `canonicalize()`
  fails; the codegen invariant for control-flow statements is now
  an error rather than `unreachable!()`.
- The formatter no longer mis-parses `#[attr]` as a line comment,
  fixing un-indented function bodies after parameter attributes.
- `install.sh` hardens its trust surface: `umask 077`, best-effort
  `set -o pipefail`, no more clobber of the caller's `$TMPDIR`, and
  `curl` / `wget` flags pin HTTPS + TLS 1.2 with retries on remote
  URLs (local `file://` URLs in CI smoke-tests bypass these flags).
- Release-binary profile now strips debuginfo, runs thin-LTO with a
  single codegen unit, and uses `panic = "abort"`. The Linux/macOS
  release archives drop from ~5.6 MB to ~2.9 MB for `ush` and from
  ~2.7 MB to ~1.3 MB for `ush_lsp`.
- Workspace-wide lints deny `todo!()`, `dbg!()`, `unimplemented!()`,
  and `unused_must_use`.
- Cargo manifests carry full metadata (`description`, `repository`,
  `homepage`, `readme`, `keywords`, `categories`, `authors`). Apps
  are `publish = false`.
- `ush --help` and `ush_lsp --help` now document every flag and
  subcommand.

## [0.6.1] — 2026-05-17

Maintenance release. See the
[GitHub release notes](https://github.com/ubugeeei/ush/releases/tag/v0.6.1)
for the full diff.

## Older releases

For 0.6.0 and earlier, refer to the
[GitHub releases page](https://github.com/ubugeeei/ush/releases).

[Unreleased]: https://github.com/ubugeeei/ush/compare/v0.7.0...HEAD
[0.7.0]: https://github.com/ubugeeei/ush/releases/tag/v0.7.0
[0.6.1]: https://github.com/ubugeeei/ush/releases/tag/v0.6.1
