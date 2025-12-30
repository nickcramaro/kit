#!/bin/bash
set -e

REPO="nickcramaro/kit"
INSTALL_DIR="$HOME/.kit/bin"

echo "Installing kit..."

# Check for required tools
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo is required. Install Rust from https://rustup.rs"
    exit 1
fi

if ! command -v git &> /dev/null; then
    echo "Error: git is required"
    exit 1
fi

# Create temp directory
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

# Clone and build
echo "Cloning repository..."
git clone --depth 1 "https://github.com/$REPO.git" "$TEMP_DIR/kit"

echo "Building..."
cd "$TEMP_DIR/kit"
cargo build --release --quiet

# Install
echo "Installing to $INSTALL_DIR..."
mkdir -p "$INSTALL_DIR"
cp target/release/kit "$INSTALL_DIR/"

# Run setup
echo ""
"$INSTALL_DIR/kit" setup

echo ""
echo "Installation complete!"
echo "Restart your shell or run: source ~/.zshrc"
