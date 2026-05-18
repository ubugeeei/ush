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
-->

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --locked --workspace --all-targets --no-deps -- -D warnings`
- [ ] `cargo test --locked --workspace`
- [ ] `cargo test --locked --workspace --release` (if behaviour-changing)
- [ ] `cargo check --locked -p ush_compiler --no-default-features` (if touching the compiler)
- [ ] `sh scripts/check_rs_line_limit.sh`
- [ ] `sh scripts/test_install.sh` (if touching the installer)
