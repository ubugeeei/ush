#!/bin/sh
set -eu

REPO_INPUT="${USH_REPO:-${UBSH_REPO:-ubugeeei/ush}}"
VERSION="${USH_VERSION:-${UBSH_VERSION:-latest}}"
PREFIX="${USH_PREFIX:-${UBSH_PREFIX:-$HOME/.local}}"
BIN_DIR_INPUT="${USH_BIN_DIR:-${UBSH_BIN_DIR:-}}"
DOWNLOAD_URL="${USH_DOWNLOAD_URL:-${UBSH_DOWNLOAD_URL:-}}"
CHECKSUM_URL="${USH_CHECKSUM_URL:-${UBSH_CHECKSUM_URL:-}}"
AUTO_PATH="${USH_AUTO_PATH:-${UBSH_AUTO_PATH:-1}}"
TMPDIR="$(mktemp -d)"

cleanup() {
  rm -rf "$TMPDIR"
}

trap cleanup EXIT INT TERM

usage() {
  cat <<'EOF'
usage: install.sh [--version VERSION] [--bin-dir DIR] [--prefix DIR] [--download-url URL] [--checksum-url URL] [--no-modify-path]

options:
  --version VERSION     install a specific release tag such as v0.5.4
  --bin-dir DIR         install ush into DIR
  --prefix DIR          install ush into DIR/bin
  --download-url URL    override the release archive URL
  --checksum-url URL    override the sha256sums.txt URL
  --no-modify-path      do not edit shell rc files
  -h, --help            show this help
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --version)
      [ "$#" -ge 2 ] || { echo "install.sh: --version requires a value" >&2; exit 1; }
      VERSION="$2"
      shift 2
      ;;
    --bin-dir)
      [ "$#" -ge 2 ] || { echo "install.sh: --bin-dir requires a value" >&2; exit 1; }
      BIN_DIR_INPUT="$2"
      shift 2
      ;;
    --prefix)
      [ "$#" -ge 2 ] || { echo "install.sh: --prefix requires a value" >&2; exit 1; }
      PREFIX="$2"
      shift 2
      ;;
    --download-url)
      [ "$#" -ge 2 ] || { echo "install.sh: --download-url requires a value" >&2; exit 1; }
      DOWNLOAD_URL="$2"
      shift 2
      ;;
    --checksum-url)
      [ "$#" -ge 2 ] || { echo "install.sh: --checksum-url requires a value" >&2; exit 1; }
      CHECKSUM_URL="$2"
      shift 2
      ;;
    --no-modify-path)
      AUTO_PATH=0
      shift
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *)
      echo "install.sh: unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

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

path_contains() {
  case ":$PATH:" in
    *":$1:"*) return 0 ;;
    *) return 1 ;;
  esac
}

pick_bin_dir() {
  if [ -n "$BIN_DIR_INPUT" ]; then
    printf '%s\n' "$BIN_DIR_INPUT"
    return
  fi

  if [ "${USH_PREFIX+x}" = "x" ] || [ "${UBSH_PREFIX+x}" = "x" ]; then
    printf '%s/bin\n' "$PREFIX"
    return
  fi

  for candidate in "$HOME/.local/bin" "$HOME/bin"; do
    if path_contains "$candidate"; then
      printf '%s\n' "$candidate"
      return
    fi
  done

  old_ifs="$IFS"
  IFS=":"
  for candidate in $PATH; do
    [ -n "$candidate" ] || continue
    case "$candidate" in
      "$HOME"/*)
        if [ -d "$candidate" ] && [ -w "$candidate" ]; then
          printf '%s\n' "$candidate"
          IFS="$old_ifs"
          return
        fi
        ;;
    esac
  done
  IFS="$old_ifs"

  printf '%s\n' "$HOME/.local/bin"
}

detect_profile() {
  shell_name="${SHELL##*/}"

  case "$shell_name" in
    zsh) printf '%s\n' "$HOME/.zshrc" ;;
    bash)
      if [ -f "$HOME/.bashrc" ] || [ ! -f "$HOME/.bash_profile" ]; then
        printf '%s\n' "$HOME/.bashrc"
      else
        printf '%s\n' "$HOME/.bash_profile"
      fi
      ;;
    ksh) printf '%s\n' "$HOME/.kshrc" ;;
    fish) printf '\n' ;;
    *) printf '%s\n' "$HOME/.profile" ;;
  esac
}

ensure_path() {
  bin_dir="$1"

  if path_contains "$bin_dir"; then
    printf 'ready\n'
    return
  fi

  if [ "$AUTO_PATH" = "0" ]; then
    printf 'manual\n'
    return
  fi

  profile="$(detect_profile)"
  if [ -z "$profile" ]; then
    printf 'manual\n'
    return
  fi

  display_dir="$bin_dir"
  case "$bin_dir" in
    "$HOME"/*) display_dir="\$HOME/${bin_dir#"$HOME"/}" ;;
  esac

  line="export PATH=\"$display_dir:\$PATH\""
  if [ -f "$profile" ] && grep -F "$line" "$profile" >/dev/null 2>&1; then
    printf '%s\n' "$profile"
    return
  fi

  touch "$profile"
  {
    printf '\n# Added by ush install.sh\n'
    printf '%s\n' "$line"
  } >> "$profile"
  printf '%s\n' "$profile"
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

build_checksum_url() {
  repo="$1"
  version="$2"

  if [ -n "$CHECKSUM_URL" ]; then
    printf '%s\n' "$CHECKSUM_URL"
    return
  fi

  if [ "$version" = "latest" ]; then
    printf 'https://github.com/%s/releases/latest/download/sha256sums.txt\n' "$repo"
    return
  fi

  printf 'https://github.com/%s/releases/download/%s/sha256sums.txt\n' "$repo" "$version"
}

sha256_tool() {
  if command -v sha256sum >/dev/null 2>&1; then
    printf 'sha256sum\n'
    return
  fi

  if command -v shasum >/dev/null 2>&1; then
    printf 'shasum -a 256\n'
    return
  fi

  if command -v openssl >/dev/null 2>&1; then
    printf 'openssl\n'
    return
  fi

  if command -v python3 >/dev/null 2>&1; then
    printf 'python3\n'
    return
  fi

  printf '\n'
}

compute_sha256() {
  tool="$(sha256_tool)"
  [ -n "$tool" ] || return 1

  case "$tool" in
    sha256sum)
      sha256sum "$1" | awk '{print $1}'
      ;;
    "shasum -a 256")
      shasum -a 256 "$1" | awk '{print $1}'
      ;;
    openssl)
      openssl dgst -sha256 "$1" | awk '{print $NF}'
      ;;
    python3)
      python3 - "$1" <<'PY'
import hashlib
import sys

digest = hashlib.sha256()
with open(sys.argv[1], "rb") as handle:
    for chunk in iter(lambda: handle.read(1024 * 1024), b""):
        digest.update(chunk)
print(digest.hexdigest())
PY
      ;;
  esac
}

verify_archive() {
  repo="$1"
  version="$2"
  asset="$3"
  archive="$4"

  tool="$(sha256_tool)"
  if [ -z "$tool" ]; then
    echo "install.sh: no supported sha256 tool found (sha256sum, shasum, openssl, python3); refusing to install without checksum verification" >&2
    exit 1
  fi

  sums_url="$(build_checksum_url "$repo" "$version")"
  sums_file="$TMPDIR/sha256sums.txt"

  echo "install.sh: downloading $sums_url" >&2
  fetch "$sums_url" "$sums_file"

  line="$(grep "[[:space:]]$asset\$" "$sums_file" || true)"
  if [ -z "$line" ]; then
    echo "install.sh: could not find checksum for $asset" >&2
    exit 1
  fi

  set -- $line
  expected="$1"
  actual="$(compute_sha256 "$archive")"

  if [ "$actual" != "$expected" ]; then
    echo "install.sh: checksum verification failed for $asset" >&2
    exit 1
  fi

  echo "install.sh: verified checksum for $asset" >&2
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
BIN_DIR="$(pick_bin_dir)"
ASSET="ush-$TARGET.tar.gz"
URL="$(build_download_url "$REPO" "$VERSION" "$TARGET")"
ARCHIVE="$TMPDIR/ush.tar.gz"
UNPACK_DIR="$TMPDIR/unpack"
PACKAGE_DIR="$UNPACK_DIR/ush-$TARGET"

echo "install.sh: downloading $URL" >&2
fetch "$URL" "$ARCHIVE"
verify_archive "$REPO" "$VERSION" "$ASSET" "$ARCHIVE"

mkdir -p "$UNPACK_DIR"
tar -xzf "$ARCHIVE" -C "$UNPACK_DIR"

if [ ! -f "$PACKAGE_DIR/ush" ] && [ -f "$UNPACK_DIR/ush" ]; then
  PACKAGE_DIR="$UNPACK_DIR"
fi

if [ ! -f "$PACKAGE_DIR/ush" ]; then
  echo "install.sh: release archive did not contain ush" >&2
  exit 1
fi

mkdir -p "$BIN_DIR"
install -m 755 "$PACKAGE_DIR/ush" "$BIN_DIR/ush"

if [ -f "$PACKAGE_DIR/ush_lsp" ]; then
  install -m 755 "$PACKAGE_DIR/ush_lsp" "$BIN_DIR/ush_lsp"
fi

PATH_STATUS="$(ensure_path "$BIN_DIR")"

echo "ush installed to $BIN_DIR/ush"
if [ -f "$PACKAGE_DIR/ush_lsp" ]; then
  echo "ush_lsp installed to $BIN_DIR/ush_lsp"
fi

case "$PATH_STATUS" in
  ready)
    echo "ush is already on PATH. Try: ush --version"
    ;;
  manual)
    echo "add this to your shell config, then restart your shell:"
    echo "export PATH=\"$BIN_DIR:\$PATH\""
    ;;
  *)
    echo "added $BIN_DIR to PATH in $PATH_STATUS"
    echo "restart your shell or run: . \"$PATH_STATUS\""
    ;;
esac

if command -v "$BIN_DIR/ush" >/dev/null 2>&1; then
  echo "quick check: $BIN_DIR/ush --version"
fi
