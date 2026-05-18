# Changelog

All notable changes to `ush` will be documented in this file.

The format is based on [Keep a Changelog][keepachangelog], and this
project adheres to [Semantic Versioning][semver].

[keepachangelog]: https://keepachangelog.com/en/1.1.0/
[semver]: https://semver.org/spec/v2.0.0.html

## [Unreleased]

### Added

- Started this changelog. Older releases are summarised below from the
  GitHub release notes; future releases should land their entries here
  first under `[Unreleased]` and then be cut into a version section.
- `SECURITY.md` with a private-disclosure policy.
- `CONTRIBUTING.md` describing the local-CI flow and project layout.
- Issue and pull-request templates under `.github/`.
- Root-level `.editorconfig` pins charset / EOL / indent.
- `Shellcheck` CI workflow lints `install.sh` and every `scripts/*.sh`.
- `Benchmark` CI job tracks parser and full `.ush → sh` compile-time
  benchmarks against the `main` baseline (on `gh-pages`) and fails
  PRs on >25% regression.
- `Deny` CI job runs `cargo-deny` for license/source/advisory
  enforcement.
- Compiler now enforces `match` exhaustiveness during the effects
  pass: missing variants on enums, missing arms on `Bool` and `Unit`,
  and literal-only matches on `String`/`Int`/tuples/lists are now
  rejected with a clear diagnostic instead of compiling to a
  silently-fall-through shell branch.
- New CI gate: every `examples/*.ush` is type-checked through
  `ush check`, not just the two that had bespoke runners.
- README links the CI / Shellcheck / Dependencies workflow badges and
  the MIT shield.
- `.dockerignore` keeps the docker build context lean; the docker
  image now runs as the non-root `ush` user (uid 1000).

### Changed

- Workspace declares `rust-version = "1.85"` as the MSRV.
- CI matrixes `clippy` and `test` jobs across Ubuntu and macOS, adds
  an MSRV gate, and runs `cargo audit` (also on a weekly schedule).
- Shell signal helpers use `sigaction(2)` instead of `signal(3)`, and
  the SIGCONT pgid path rejects PIDs that do not fit `pid_t`.
- The compiler refuses to silently fall back when `canonicalize()`
  fails; the codegen invariant for control-flow statements is now an
  error rather than `unreachable!()`.
- `install.sh` hardens its trust surface: `umask 077`, best-effort
  `set -o pipefail`, no more clobber of the caller's `$TMPDIR`, and
  `curl`/`wget` flags pin HTTPS + TLS 1.2 with retries on remote
  URLs (local `file://` URLs in CI smoke-tests bypass these flags).
- Release-binary profile now strips debuginfo, runs thin-LTO with a
  single codegen unit, and uses `panic = "abort"`. The Linux/macOS
  release archives drop from ~5.6 MB to ~2.9 MB for `ush` and from
  ~2.7 MB to ~1.3 MB for `ush_lsp`.

## [0.6.1] — 2026-05-17

Maintenance release. See the
[GitHub release notes](https://github.com/ubugeeei/ush/releases/tag/v0.6.1)
for the full diff.

## Older releases

For 0.6.0 and earlier, refer to the
[GitHub releases page](https://github.com/ubugeeei/ush/releases).

[Unreleased]: https://github.com/ubugeeei/ush/compare/v0.6.1...HEAD
[0.6.1]: https://github.com/ubugeeei/ush/releases/tag/v0.6.1
