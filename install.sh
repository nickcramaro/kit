#!/bin/bash
set -e

REPO="nickcramaro/kit"
INSTALL_DIR="$HOME/.kit/bin"

echo "Installing kit..."

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
    darwin) OS="darwin" ;;
    linux) OS="linux" ;;
    *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
    x86_64) ARCH="x64" ;;
    amd64) ARCH="x64" ;;
    arm64) ARCH="arm64" ;;
    aarch64) ARCH="arm64" ;;
    *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

BINARY_NAME="kit-${OS}-${ARCH}"

# Get latest release
LATEST=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" 2>/dev/null | grep '"tag_name"' | cut -d'"' -f4 || echo "")

if [ -n "$LATEST" ]; then
    echo "Downloading $BINARY_NAME ($LATEST)..."
    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST/$BINARY_NAME.tar.gz"

    TEMP_DIR=$(mktemp -d)
    trap "rm -rf $TEMP_DIR" EXIT

    if curl -fsSL "$DOWNLOAD_URL" -o "$TEMP_DIR/kit.tar.gz"; then
        mkdir -p "$INSTALL_DIR"
        tar -xzf "$TEMP_DIR/kit.tar.gz" -C "$INSTALL_DIR"
        chmod +x "$INSTALL_DIR/kit"
    else
        echo "Failed to download binary, falling back to building from source..."
        LATEST=""
    fi
fi

# Fall back to building from source
if [ -z "$LATEST" ]; then
    echo "No release found, building from source..."

    if ! command -v cargo &> /dev/null; then
        echo "Error: cargo is required. Install Rust from https://rustup.rs"
        exit 1
    fi

    if ! command -v git &> /dev/null; then
        echo "Error: git is required"
        exit 1
    fi

    TEMP_DIR=$(mktemp -d)
    trap "rm -rf $TEMP_DIR" EXIT

    git clone --depth 1 "https://github.com/$REPO.git" "$TEMP_DIR/kit"
    cd "$TEMP_DIR/kit"
    cargo build --release --quiet

    mkdir -p "$INSTALL_DIR"
    cp target/release/kit "$INSTALL_DIR/"
fi

# Run setup
echo ""
"$INSTALL_DIR/kit" setup

echo ""
echo "Installation complete!"
echo "Restart your shell or run: source ~/.zshrc"
