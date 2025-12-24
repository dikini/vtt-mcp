#!/bin/bash
# vtt-mcp Installation Script
# Automates setup of the Voice-to-Text MCP server

set -e

# Options
DEV=false
SYSTEMD=false
SKIP_MODEL=false
MODEL="base"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --help) grep '^#' "$0" | tail -n +2 | sed 's/^# //; s/^#//'; exit 0 ;;
        --dev) DEV=true; shift ;;
        --systemd) SYSTEMD=true; shift ;;
        --skip-model) SKIP_MODEL=true; shift ;;
        --model) MODEL="$2"; shift 2 ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║            VTT-MCP Installation Script                         ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

# Check Rust
echo "[*] Checking Rust installation..."
if ! command -v rustc &> /dev/null; then
    echo "[!] Rust not found. Install from https://rustup.rs"
    exit 1
fi
echo "[✓] Rust $(rustc --version | awk '{print $2}') found"

# Check system dependencies
echo "[*] Checking system dependencies..."
OS=$(uname -s)
case "$OS" in
    Linux)
        if command -v pactl &> /dev/null; then
            echo "[✓] PipeWire/PulseAudio found"
        elif command -v aplay &> /dev/null; then
            echo "[✓] ALSA found"
        else
            echo "[!] No audio system found"
        fi
        
        if command -v nvidia-smi &> /dev/null; then
            echo "[✓] NVIDIA GPU detected"
            HAS_GPU=true
        else
            echo "[i] No GPU - using CPU mode"
            HAS_GPU=false
        fi
        ;;
    Darwin)
        echo "[✓] macOS with CoreAudio"
        ;;
esac

# Create directories
echo "[*] Creating directories..."
mkdir -p models
mkdir -p "$HOME/.config/vtt-mcp"
echo "[✓] Directories created"

# Create default config
if [ ! -f "$HOME/.config/vtt-mcp/config.toml" ]; then
    cat > "$HOME/.config/vtt-mcp/config.toml" <<EOF
# VTT-MCP Configuration
[model]
model_path = "./models/ggml-base.bin"
n_threads = 0
use_gpu = true

[transcription]
required_sample_rate = 16000
language = ""
EOF
    echo "[✓] Config created"
fi

# Download model
if [ "$SKIP_MODEL" = false ]; then
    echo "[*] Downloading Whisper $MODEL model..."
    MODEL_FILE="ggml-$MODEL.bin"
    MODEL_URL="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/$MODEL_FILE"
    
    if [ -f "models/$MODEL_FILE" ]; then
        echo "[i] Model already exists, skipping download"
    else
        if command -v wget &> /dev/null; then
            wget --progress=bar:force "$MODEL_URL" -O "models/$MODEL_FILE"
        elif command -v curl &> /dev/null; then
            curl -L --progress-bar "$MODEL_URL" -o "models/$MODEL_FILE"
        else
            echo "[!] Neither wget nor curl found"
            exit 1
        fi
        echo "[✓] Model downloaded"
    fi
fi

# Build project
echo "[*] Building vtt-mcp..."
BUILD_FLAGS="--release"
if [ "$HAS_GPU" = true ]; then
    echo "[i] Building with CUDA support"
    BUILD_FLAGS="$BUILD_FLAGS --features cuda"
fi

if [ "$DEV" = true ]; then
    cargo build --all $BUILD_FLAGS
else
    cargo build --package vtt-cli --package vtt-mcp $BUILD_FLAGS
fi
echo "[✓] Build complete"

# Install systemd service
if [ "$SYSTEMD" = true ]; then
    echo "[*] Installing systemd service..."
    if command -v systemctl &> /dev/null; then
        mkdir -p "$HOME/.config/systemd/user"
        cat > "$HOME/.config/systemd/user/vtt-mcp.service" <<EOF
[Unit]
Description=VTT-MCP Voice-to-Text MCP Server
After=network.target

[Service]
Type=simple
ExecStart=$PWD/target/release/vtt-mcp
WorkingDirectory=$PWD
Restart=on-failure

[Install]
WantedBy=default.target
EOF
        systemctl --user daemon-reload
        systemctl --user enable vtt-mcp.service
        echo "[✓] Systemd service installed"
        echo "    Start with: systemctl --user start vtt-mcp"
    else
        echo "[i] systemctl not found, skipping systemd"
    fi
fi

# Summary
echo ""
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║              Installation Complete!                           ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""
echo "Quick start:"
echo "  ./target/release/vtt-cli --duration 5"
echo ""
echo "Documentation:"
echo "  - docs/SETUP.md"
echo "  - README.md"
echo ""
