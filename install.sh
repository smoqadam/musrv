#!/usr/bin/env bash
set -euo pipefail

# Enable tracing if MUSRV_DEBUG=1
if [ "${MUSRV_DEBUG:-}" = "1" ]; then
  set -x
fi

VERSION="${MUSRV_VERSION:-latest}"
REPO="smoqadam/musrv"

# detect OS/arch
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
case "$OS-$ARCH" in
  linux-x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
  linux-armv7l) TARGET="armv7-unknown-linux-gnueabihf" ;;
  darwin-arm64)  TARGET="aarch64-apple-darwin" ;;
  *) echo "Unsupported OS/arch (supported: linux x86_64, linux armv7l, macOS arm64): $OS-$ARCH" >&2; exit 1 ;;
esac

echo "[musrv-install] repo=$REPO os=$OS arch=$ARCH target=$TARGET" >&2

if [ "$VERSION" = "latest" ]; then
  RELEASE_JSON="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest")"
  VERSION="$(printf "%s" "$RELEASE_JSON" | sed -n 's/.*"tag_name": *"\(.*\)".*/\1/p' | head -n1)"
else
  RELEASE_JSON="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/tags/$VERSION")"
fi

echo "[musrv-install] resolved version=$VERSION" >&2

# Pick an asset matching target and common archive extensions
URL="$(
  printf "%s" "$RELEASE_JSON" \
  | sed -nE 's/.*"browser_download_url": "([^"]+)".*/\1/p' \
  | grep "$TARGET" \
  | grep -E '\.(tar\.(gz|xz)|zip)$' \
  | head -n1
)"
echo "[musrv-install] asset url=${URL:-<none>}" >&2
if [ -z "$URL" ]; then
  echo "No release asset found for target $TARGET in $VERSION" >&2
  echo "Available assets:" >&2
  printf "%s\n" "$RELEASE_JSON" | sed -nE 's/.*"browser_download_url": "([^"]+)".*/\1/p' >&2
  exit 1
fi

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT
echo "[musrv-install] downloading $URL" >&2
case "$URL" in
  *.tar.gz) curl -fL "$URL" | tar -xz -C "$TMPDIR" ;;
  *.tar.xz) curl -fL "$URL" | tar -xJ -C "$TMPDIR" ;;
  *.zip)
    curl -fL -o "$TMPDIR/musrv.zip" "$URL"
    unzip -q "$TMPDIR/musrv.zip" -d "$TMPDIR"
    ;;
  *) echo "Unknown archive type for $URL" >&2; exit 1 ;;
esac
mkdir -p /usr/local/bin
install -m755 "$TMPDIR/musrv" /usr/local/bin/musrv
echo "Installed musrv $VERSION to /usr/local/bin"
