#!/bin/sh
# scripts/preflight.sh — run every CI gate that lives in this repo,
# in the same order CI runs them, so a contributor can catch a CI
# failure before pushing.
#
# Each section prints a banner so the failing one is easy to spot
# even in plain `sh -e` output.

set -eu

banner() {
  printf '\n=== %s ===\n' "$1"
}

banner "rustfmt (stable)"
cargo +stable fmt --all --check

banner "clippy"
cargo clippy --locked --workspace --all-targets --no-deps -- -D warnings

banner "workspace tests"
cargo test --locked --workspace

banner "release tests"
cargo test --locked --workspace --release

banner "no_std compiler core (check)"
cargo check --locked -p ush_compiler --no-default-features

banner "no_std compiler core (tests)"
cargo test --locked -p ush_compiler --no-default-features --lib

banner "rustdoc (warnings as errors)"
RUSTDOCFLAGS="-D warnings" cargo doc \
  --locked --workspace --no-deps --document-private-items

banner "parser bench (build only)"
cargo bench --locked -p ush_shell --bench parser --no-run

banner "compile bench (build only)"
cargo bench --locked -p ush_compiler --bench compile --no-run

banner "rust line-limit"
sh scripts/check_rs_line_limit.sh

banner "vendored rustyline drift"
sh scripts/check_rustyline_upstream.sh

banner "installer flow"
sh scripts/test_install.sh

if command -v shellcheck >/dev/null 2>&1; then
  banner "shellcheck"
  shellcheck --severity=warning install.sh scripts/*.sh
else
  banner "shellcheck (skipped — not installed)"
fi

banner "ush check / format on examples"
for example in examples/*.ush; do
  cargo run --locked -p ush --quiet -- check "$example"
  cargo run --locked -p ush --quiet -- format --check "$example"
done

banner "preflight passed"
