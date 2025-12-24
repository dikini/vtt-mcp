#!/bin/bash
# Quick install for vtt-mcp (advanced users)
set -e

echo "Installing vtt-mcp..."

# Setup
mkdir -p models
mkdir -p "$HOME/.config/vtt-mcp"

# Download model
if [ ! -f "models/ggml-base.bin" ]; then
    echo "Downloading Whisper base model (142MB)..."
    wget -q --show-progress \
        https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin \
        -O models/ggml-base.bin
fi

# Build
echo "Building..."
cargo build --release --package vtt-cli --package vtt-mcp

echo ""
echo "✓ Installation complete!"
echo ""
echo "Test with:"
echo "  ./target/release/vtt-cli --duration 5"
