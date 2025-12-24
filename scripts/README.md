# VTT-MCP Installation Scripts

This directory contains installation scripts for vtt-mcp.

## Scripts

### install.sh
Comprehensive installation script with dependency checks and options.

**Usage:**
```bash
./scripts/install.sh [OPTIONS]
```

**Options:**
- `--help` - Show help message
- `--dev` - Install development tools and run tests
- `--systemd` - Install systemd user service
- `--skip-model` - Skip model download
- `--model MODEL` - Specify model size (base, small, medium, large-v3)

**Examples:**
```bash
# Standard installation
./scripts/install.sh

# Installation with systemd service
./scripts/install.sh --systemd

# Development installation with tests
./scripts/install.sh --dev

# Use small model instead of base
./scripts/install.sh --model small
```

**What it does:**
1. Checks Rust installation
2. Detects system dependencies (audio, GPU)
3. Creates directory structure
4. Downloads Whisper model (142MB base model by default)
5. Builds vtt-cli and vtt-mcp with GPU support if detected
6. Optionally installs systemd user service
7. Displays quick start instructions

### quick-install.sh
Minimal installation script for advanced users.

**Usage:**
```bash
./scripts/quick-install.sh
```

**What it does:**
- Creates directories
- Downloads base model
- builds CLI and MCP server
- No dependency checks or extras

### download-whisper-model.sh
Script to download Whisper models separately.

**Usage:**
```bash
./scripts/download-whisper-model.sh [MODEL_SIZE]
```

### benchmark.sh
Run performance benchmarks on the installation.

**Usage:**
```bash
./scripts/benchmark.sh
```

## Configuration

After installation, configuration is stored in:
- `~/.config/vtt-mcp/config.toml` - User configuration
- `./models/` - Whisper model files

## Model Sizes

Available Whisper models:
- **tiny** (~75MB) - Fastest, lower accuracy
- **base** (~142MB) - Good balance (default)
- **small** (~462MB) - Better accuracy
- **medium** (~1.5GB) - Even better
- **large-v3** (~3GB) - Best accuracy, requires GPU

## Troubleshooting

### "Rust not found"
Install Rust from https://rustup.rs:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### "No audio system found"
Install PipeWire (Linux):
```bash
sudo apt install pipewire pipewire-audio-client-libraries
```

### Build errors
If CUDA build fails, build without GPU:
```bash
cargo build --release --package vtt-cli
```

## Systemd Service

When installed with `--systemd`, the service can be managed with:
```bash
systemctl --user start vtt-mcp    # Start service
systemctl --user stop vtt-mcp     # Stop service
systemctl --user status vtt-mcp   # Check status
journalctl --user -u vtt-mcp -f   # View logs
```
