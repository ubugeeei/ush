#!/bin/sh
set -eu

REPO="${USH_REPO:-${UBSH_REPO:-https://github.com/ubugeeei/ubshell.git}}"
BRANCH="${USH_BRANCH:-${UBSH_BRANCH:-main}}"
PREFIX="${USH_PREFIX:-${UBSH_PREFIX:-$HOME/.local}}"
TMPDIR="$(mktemp -d)"

cleanup() {
  rm -rf "$TMPDIR"
}

trap cleanup EXIT INT TERM

if ! command -v git >/dev/null 2>&1; then
  echo "install.sh: git is required" >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "install.sh: cargo is required" >&2
  exit 1
fi

git clone --depth 1 --branch "$BRANCH" "$REPO" "$TMPDIR/repo"
cargo build --release -p ush --manifest-path "$TMPDIR/repo/Cargo.toml"
mkdir -p "$PREFIX/bin"
install -m 755 "$TMPDIR/repo/target/release/ush" "$PREFIX/bin/ush"

echo "ush installed to $PREFIX/bin/ush"
