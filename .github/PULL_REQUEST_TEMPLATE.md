## Summary

<!--
What does this PR do, and why? One or two sentences. Use bullets if
there is more than one logical change, but please consider splitting
the PR if so.
-->

## Notes

<!--
Anything the reviewer should know up-front: design tradeoffs you
considered, why you picked this approach over alternatives, behaviour
that looks suspicious but is intentional, related issues, follow-ups
you plan to file.
-->

## Validation

<!--
Tick the boxes for what you actually ran locally. CI runs the same
gates, but landing PRs that have been validated locally is faster.
The one-line `sh scripts/preflight.sh` runs every gate below.
-->

- [ ] `cargo +stable fmt --all --check`
- [ ] `cargo clippy --locked --workspace --all-targets --no-deps -- -D warnings`
- [ ] `cargo test --locked --workspace`
- [ ] `cargo test --locked --workspace --release` (if behaviour-changing)
- [ ] `cargo check --locked -p ush_compiler --no-default-features` (if touching the compiler)
- [ ] `cargo test --locked -p ush_compiler --no-default-features --lib` (if touching the compiler)
- [ ] `RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --no-deps --document-private-items`
- [ ] `sh scripts/check_rs_line_limit.sh`
- [ ] `sh scripts/check_rustyline_upstream.sh` (if touching `vendor/`)
- [ ] `sh scripts/test_install.sh` (if touching the installer)
- [ ] `shellcheck --severity=warning install.sh scripts/*.sh` (if touching shell scripts)
- [ ] `ush check` + `ush format --check` over `examples/*.ush` (if touching the compiler or formatter)

## Security

<!--
Anything to flag for SECURITY.md / advisories? Examples:
- changes to install.sh / curl|sh path
- changes to signal handlers or process spawning
- changes to release pipeline / attestations
- new third-party action introduced (must be pinned to a commit SHA)
Leave blank if nothing applies.
-->
