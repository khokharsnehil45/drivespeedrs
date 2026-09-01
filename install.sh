#!/usr/bin/env bash
set -e

# Target GitHub Repository (update with your GitHub username)
REPO="khokharsnehil45/drivespeedrs"
BINARY_NAME="drivespeedrs"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

echo "⚡ Installing DriveSpeed RS..."

# Detect OS and Architecture
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
    x86_64|amd64)
        TARGET_ARCH="x86_64"
        ;;
    aarch64|arm64)
        TARGET_ARCH="aarch64"
        ;;
    *)
        echo "❌ Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

case "$OS" in
    linux)
        TARGET_OS="unknown-linux-gnu"
        ;;
    darwin)
        TARGET_OS="apple-darwin"
        ;;
    *)
        echo "❌ Unsupported OS: $OS"
        exit 1
        ;;
esac

ASSET_NAME="${BINARY_NAME}-${TARGET_ARCH}-${TARGET_OS}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ASSET_NAME}"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

echo "⬇️ Downloading ${BINARY_NAME} for ${OS}/${ARCH}..."
curl -sSL "$DOWNLOAD_URL" -o "$TMP_DIR/$ASSET_NAME"

tar -xzf "$TMP_DIR/$ASSET_NAME" -C "$TMP_DIR"

if [ -w "$INSTALL_DIR" ]; then
    mv "$TMP_DIR/$BINARY_NAME" "$INSTALL_DIR/"
else
    echo "🔑 Elevating permissions with sudo to install into $INSTALL_DIR..."
    sudo mv "$TMP_DIR/$BINARY_NAME" "$INSTALL_DIR/"
fi

chmod +x "$INSTALL_DIR/$BINARY_NAME"

echo "✅ Successfully installed drivespeedrs to $INSTALL_DIR/$BINARY_NAME!"
echo "🚀 Run it anytime by typing: drivespeedrs"
