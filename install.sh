#!/bin/sh
set -eu

REPO_INPUT="${USH_REPO:-${UBSH_REPO:-ubugeeei/ush}}"
VERSION="${USH_VERSION:-${UBSH_VERSION:-latest}}"
PREFIX="${USH_PREFIX:-${UBSH_PREFIX:-$HOME/.local}}"
DOWNLOAD_URL="${USH_DOWNLOAD_URL:-${UBSH_DOWNLOAD_URL:-}}"
TMPDIR="$(mktemp -d)"

cleanup() {
  rm -rf "$TMPDIR"
}

trap cleanup EXIT INT TERM

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "install.sh: $1 is required" >&2
    exit 1
  fi
}

normalize_repo() {
  printf '%s' "$1" | sed \
    -e 's#^https\{0,1\}://github\.com/##' \
    -e 's#^git@github\.com:##' \
    -e 's#\.git$##'
}

detect_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux) os="unknown-linux-gnu" ;;
    Darwin) os="apple-darwin" ;;
    *)
      echo "install.sh: unsupported operating system: $os" >&2
      echo "install.sh: build from source or use Docker/Nix on this platform" >&2
      exit 1
      ;;
  esac

  case "$arch" in
    x86_64 | amd64) arch="x86_64" ;;
    arm64 | aarch64) arch="aarch64" ;;
    *)
      echo "install.sh: unsupported architecture: $arch" >&2
      echo "install.sh: build from source on this platform" >&2
      exit 1
      ;;
  esac

  printf '%s-%s\n' "$arch" "$os"
}

fetch() {
  url="$1"
  output="$2"

  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$output"
    return
  fi

  if command -v wget >/dev/null 2>&1; then
    wget -qO "$output" "$url"
    return
  fi

  echo "install.sh: curl or wget is required" >&2
  exit 1
}

build_download_url() {
  repo="$1"
  version="$2"
  target="$3"
  asset="ush-$target.tar.gz"

  if [ -n "$DOWNLOAD_URL" ]; then
    printf '%s\n' "$DOWNLOAD_URL"
    return
  fi

  if [ "$version" = "latest" ]; then
    printf 'https://github.com/%s/releases/latest/download/%s\n' "$repo" "$asset"
    return
  fi

  printf 'https://github.com/%s/releases/download/%s/%s\n' "$repo" "$version" "$asset"
}

if [ "${USH_BRANCH+x}" = "x" ] || [ "${UBSH_BRANCH+x}" = "x" ]; then
  echo "install.sh: USH_BRANCH/UBSH_BRANCH is no longer supported" >&2
  echo "install.sh: use USH_VERSION or build from source for branch installs" >&2
  exit 1
fi

need_cmd tar
need_cmd install

REPO="$(normalize_repo "$REPO_INPUT")"
TARGET="$(detect_target)"
URL="$(build_download_url "$REPO" "$VERSION" "$TARGET")"
ARCHIVE="$TMPDIR/ush.tar.gz"
UNPACK_DIR="$TMPDIR/unpack"
PACKAGE_DIR="$UNPACK_DIR/ush-$TARGET"

echo "install.sh: downloading $URL" >&2
fetch "$URL" "$ARCHIVE"

mkdir -p "$UNPACK_DIR"
tar -xzf "$ARCHIVE" -C "$UNPACK_DIR"

if [ ! -f "$PACKAGE_DIR/ush" ] && [ -f "$UNPACK_DIR/ush" ]; then
  PACKAGE_DIR="$UNPACK_DIR"
fi

if [ ! -f "$PACKAGE_DIR/ush" ]; then
  echo "install.sh: release archive did not contain ush" >&2
  exit 1
fi

mkdir -p "$PREFIX/bin"
install -m 755 "$PACKAGE_DIR/ush" "$PREFIX/bin/ush"

if [ -f "$PACKAGE_DIR/ush_lsp" ]; then
  install -m 755 "$PACKAGE_DIR/ush_lsp" "$PREFIX/bin/ush_lsp"
fi

echo "ush installed to $PREFIX/bin/ush"
if [ -f "$PACKAGE_DIR/ush_lsp" ]; then
  echo "ush_lsp installed to $PREFIX/bin/ush_lsp"
fi
