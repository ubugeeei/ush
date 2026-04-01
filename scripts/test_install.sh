#!/bin/sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
INSTALL_SH="$ROOT_DIR/install.sh"

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "test_install.sh: $1 is required" >&2
    exit 1
  fi
}

detect_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux) os="unknown-linux-gnu" ;;
    Darwin) os="apple-darwin" ;;
    *) echo "test_install.sh: unsupported os: $os" >&2; exit 1 ;;
  esac

  case "$arch" in
    x86_64 | amd64) arch="x86_64" ;;
    arm64 | aarch64) arch="aarch64" ;;
    *) echo "test_install.sh: unsupported arch: $arch" >&2; exit 1 ;;
  esac

  printf '%s-%s\n' "$arch" "$os"
}

make_archive() {
  tmpdir="$1"
  target="$2"
  pkg="$tmpdir/ush-$target"

  mkdir -p "$pkg"
  printf '#!/bin/sh\necho ush-test\n' > "$pkg/ush"
  printf '#!/bin/sh\necho ush-lsp-test\n' > "$pkg/ush_lsp"
  chmod +x "$pkg/ush" "$pkg/ush_lsp"
  tar -czf "$tmpdir/ush.tar.gz" -C "$tmpdir" "ush-$target"
}

assert_contains() {
  haystack="$1"
  needle="$2"

  case "$haystack" in
    *"$needle"*) ;;
    *)
      echo "test_install.sh: expected output to contain: $needle" >&2
      printf '%s\n' "$haystack" >&2
      exit 1
      ;;
  esac
}

test_existing_path_dir() {
  tmpdir="$(mktemp -d)"

  target="$(detect_target)"
  make_archive "$tmpdir" "$target"

  home_dir="$tmpdir/home"
  mkdir -p "$home_dir/bin"
  output="$(
    HOME="$home_dir" \
    PATH="$home_dir/bin:/usr/bin:/bin" \
    SHELL=/bin/zsh \
    USH_DOWNLOAD_URL="file://$tmpdir/ush.tar.gz" \
    sh "$INSTALL_SH" 2>&1
  )"

  [ -x "$home_dir/bin/ush" ]
  [ -x "$home_dir/bin/ush_lsp" ]
  [ ! -e "$home_dir/.zshrc" ]
  assert_contains "$output" "ush is already on PATH"
  rm -rf "$tmpdir"
}

test_auto_path_update() {
  tmpdir="$(mktemp -d)"

  target="$(detect_target)"
  make_archive "$tmpdir" "$target"

  home_dir="$tmpdir/home"
  mkdir -p "$home_dir"
  output="$(
    HOME="$home_dir" \
    PATH="/usr/bin:/bin" \
    SHELL=/bin/zsh \
    USH_DOWNLOAD_URL="file://$tmpdir/ush.tar.gz" \
    sh "$INSTALL_SH" 2>&1
  )"

  [ -x "$home_dir/.local/bin/ush" ]
  [ -x "$home_dir/.local/bin/ush_lsp" ]
  grep -F 'export PATH="$HOME/.local/bin:$PATH"' "$home_dir/.zshrc" >/dev/null
  assert_contains "$output" "added $home_dir/.local/bin to PATH"
  rm -rf "$tmpdir"
}

test_auto_path_disabled() {
  tmpdir="$(mktemp -d)"

  target="$(detect_target)"
  make_archive "$tmpdir" "$target"

  home_dir="$tmpdir/home"
  mkdir -p "$home_dir"
  output="$(
    HOME="$home_dir" \
    PATH="/usr/bin:/bin" \
    SHELL=/bin/zsh \
    USH_AUTO_PATH=0 \
    USH_DOWNLOAD_URL="file://$tmpdir/ush.tar.gz" \
    sh "$INSTALL_SH" 2>&1
  )"

  [ -x "$home_dir/.local/bin/ush" ]
  [ ! -e "$home_dir/.zshrc" ]
  assert_contains "$output" 'export PATH="'
  rm -rf "$tmpdir"
}

test_cli_flags() {
  tmpdir="$(mktemp -d)"

  target="$(detect_target)"
  make_archive "$tmpdir" "$target"

  home_dir="$tmpdir/home"
  bin_dir="$home_dir/tools/bin"
  mkdir -p "$home_dir"
  output="$(
    HOME="$home_dir" \
    PATH="/usr/bin:/bin" \
    SHELL=/bin/zsh \
    sh "$INSTALL_SH" \
      --bin-dir "$bin_dir" \
      --no-modify-path \
      --download-url "file://$tmpdir/ush.tar.gz" 2>&1
  )"

  [ -x "$bin_dir/ush" ]
  [ -x "$bin_dir/ush_lsp" ]
  [ ! -e "$home_dir/.zshrc" ]
  assert_contains "$output" "ush installed to $bin_dir/ush"
  rm -rf "$tmpdir"
}

main() {
  need_cmd mktemp
  need_cmd tar
  need_cmd install
  test_existing_path_dir
  test_auto_path_update
  test_auto_path_disabled
  test_cli_flags
}

main "$@"
