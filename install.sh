#!/bin/bash

# assum installation script
# This script downloads and installs the pre-compiled binary for asum.
# It supports Linux (x86_64) and macOS (x86_64, aarch64/arm64).

set -e

REPO="tranthethang/ai-commit-summarizer"
APP_NAME="asum"
INSTALL_DIR="/usr/local/bin"

echo "--------------------------------------------------"
echo "🚀 Installing $APP_NAME..."
echo "--------------------------------------------------"

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Map architecture to Rust target names
if [ "$ARCH" = "x86_64" ] || [ "$ARCH" = "amd64" ]; then
    ARCH="x86_64"
elif [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
    ARCH="aarch64"
else
    echo "❌ Error: Unsupported architecture ($ARCH)."
    exit 1
fi

if [ "$OS" = "linux" ]; then
    if [ "$ARCH" = "x86_64" ]; then
        TARGET="x86_64-unknown-linux-gnu"
    else
        echo "❌ Error: Pre-compiled binaries for Linux $ARCH are not available yet."
        exit 1
    fi
elif [ "$OS" = "darwin" ]; then
    if [ "$ARCH" = "x86_64" ]; then
        TARGET="x86_64-apple-darwin"
    elif [ "$ARCH" = "aarch64" ]; then
        TARGET="aarch64-apple-darwin"
    fi
else
    echo "❌ Error: Unsupported operating system ($OS)."
    exit 1
fi

TAR_NAME="${APP_NAME}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/latest/${TAR_NAME}"
TEMP_DIR=$(mktemp -d)

# Download the archive
echo "Downloading $APP_NAME for $OS ($ARCH)..."
echo "URL: $DOWNLOAD_URL"
if ! curl -fsSL -o "$TEMP_DIR/$TAR_NAME" "$DOWNLOAD_URL"; then
    echo "❌ Error: Failed to download $APP_NAME. Please check your internet connection or try again later."
    rm -rf "$TEMP_DIR"
    exit 1
fi

# Extract the archive
echo "Extracting..."
tar -xzf "$TEMP_DIR/$TAR_NAME" -C "$TEMP_DIR"

# Install the binary
echo "Installing to $INSTALL_DIR (requires sudo access)..."
sudo cp "$TEMP_DIR/$APP_NAME" "$INSTALL_DIR/$APP_NAME"
sudo chmod +x "$INSTALL_DIR/$APP_NAME"

# Clean up
rm -rf "$TEMP_DIR"

echo "--------------------------------------------------"
echo "✅ SUCCESS: $APP_NAME has been installed successfully!"
echo "Location: $(which $APP_NAME)"
echo "Version:  $($APP_NAME --version 2>/dev/null || echo 'Unknown')"
echo "--------------------------------------------------"
