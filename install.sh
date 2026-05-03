#!/usr/bin/env bash
set -e

BINARY_NAME="cs"
INSTALL_DIR="/usr/local/bin"

echo "==> Downloading $BINARY_NAME..."

ARCH=$(uname -m)
OS=$(uname -s | tr '[:upper:]' '[:lower:]')

case "$OS" in
  linux)  PLATFORM="linux" ;;
  darwin) PLATFORM="darwin" ;;
  *)
    echo "Unsupported OS: $OS"
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64)   TARGET="x86_64-${PLATFORM}" ;;
  aarch64|arm64) TARGET="aarch64-${PLATFORM}" ;;
  *)
    echo "Unsupported architecture: $ARCH"
    exit 1
    ;;
esac

ASSET_NAME="cs-${TARGET}"
VERSION="${VERSION:-$(curl -s https://api.github.com/repos/dougmaitelli/cs/releases/latest | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')}"

if [ -z "$VERSION" ]; then
  echo "Could not determine latest version."
  exit 1
fi

DOWNLOAD_URL="https://github.com/dougmaitelli/cs/releases/download/${VERSION}/${ASSET_NAME}"

TMPFILE=$(mktemp)

echo "==> Fetching $DOWNLOAD_URL ..."
curl -fsSL "$DOWNLOAD_URL" -o "$TMPFILE"

echo "==> Installing $BINARY_NAME to $INSTALL_DIR..."

if [ "$(id -u)" -ne 0 ]; then
  echo "==> Re-running with sudo to install to $INSTALL_DIR..."
  sudo mv "$TMPFILE" "$INSTALL_DIR/$BINARY_NAME"
else
  mv "$TMPFILE" "$INSTALL_DIR/$BINARY_NAME"
fi

chmod +x "$INSTALL_DIR/$BINARY_NAME"

echo "==> $BINARY_NAME installed successfully (version $VERSION)!"
echo "==> Run '$BINARY_NAME --help' to get started."
