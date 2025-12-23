# VTT-MCP Setup Guide

Complete setup instructions for the Voice-to-Text MCP server.

## Table of Contents

- [System Requirements](#system-requirements)
- [Quick Start](#quick-start)
- [Detailed Setup](#detailed-setup)
  - [Rust Installation](#rust-installation)
  - [Audio System Setup](#audio-system-setup)
  - [Model Download](#model-download)
  - [GPU Acceleration (Optional)](#gpu-acceleration-optional)
- [Building](#building)
- [Testing](#testing)
- [Troubleshooting](#troubleshooting)

---

## System Requirements

### Minimum Requirements
- **OS**: Linux (PipeWire preferred), macOS, or Windows
- **Rust**: 1.70 or later
- **RAM**: 2GB (4GB+ recommended)
- **Disk**: 500MB for code + 150MB-3GB for models
- **Audio**: Microphone or audio input device

### Recommended for GPU Acceleration
- **GPU**: NVIDIA GPU with CUDA support (GTX 1060 or better)
- **VRAM**: 4GB+ for base model, 10GB+ for large model
- **Driver**: NVIDIA drivers 470+ with CUDA toolkit 11+

---

## Quick Start

Get up and running in 5 minutes:

```bash
# 1. Clone and navigate
git clone <repository-url>
cd vtt-mcp

# 2. Download Whisper base model (142MB)
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin -O models/ggml-base.bin

# 3. Build the CLI tool
cargo build --release --package vtt-cli

# 4. Test recording and transcription
cargo run --release --package vtt-cli -- --duration 5
```

Speak into your microphone for 5 seconds, and see the transcription!

---

## Detailed Setup

### Rust Installation

#### On Linux (Debian/Ubuntu)
```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify installation
rustc --version  # Should be 1.70+
cargo --version
```

#### On macOS
```bash
# Install Homebrew if needed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install Rust
brew install rust
```

#### On Windows
Download and run rustup-init.exe from [https://rustup.rs](https://rustup.rs)

---

### Audio System Setup

#### Linux (PipeWire/ALSA)

**PipeWire** (recommended for modern Linux):
```bash
# Debian/Ubuntu
sudo apt install pipewire pipewire-audio-client-libraries libpipewire-0-3-dev

# Arch
sudo pacman -S pipewire pipewire-pulse

# Fedora (usually pre-installed)
sudo dnf install pipewire pipewire-pulseaudio
```

**ALSA** (fallback):
```bash
sudo apt install libasound2-dev libportaudio2  # Debian/Ubuntu
```

**Verify audio devices:**
```bash
cargo run --package vtt-cli -- --list-devices
```

#### macOS
CoreAudio is built into macOS. No additional setup needed.

#### Windows
WASAPI is supported by default. No additional setup needed.

---

### Model Download

Whisper models are downloaded separately from HuggingFace:

#### Base Model (142MB) - Recommended for Testing
```bash
mkdir -p models
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin -O models/ggml-base.bin
```

**Performance**: ~1-2s for 5s audio, good accuracy

#### Small Model (462MB)
```bash
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin -O models/ggml-small.bin
```

**Performance**: ~2-4s for 5s audio, better accuracy

#### Large Model (3GB+)
```bash
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin -O models/ggml-large-v3.bin
```

**Performance**: ~5-10s for 5s audio, best accuracy (requires GPU)

#### All Available Models
See [whisper.cpp models](https://github.com/ggerganov/whisper.cpp#models) for the complete list (tiny, base, small, medium, large-v1, large-v2, large-v3).

---

### GPU Acceleration (Optional)

For faster transcription, enable CUDA (NVIDIA) or ROCm (AMD).

#### NVIDIA CUDA

1. **Install NVIDIA drivers** (470+):
```bash
# Ubuntu/Debian
sudo apt install nvidia-driver-535

# Verify
nvidia-smi
```

2. **Install CUDA toolkit**:
```bash
# Ubuntu/Debian
sudo apt install nvidia-cuda-toolkit

# Or download from NVIDIA: https://developer.nvidia.com/cuda-downloads
```

3. **Build with CUDA feature**:
```bash
cargo build --release --package vtt-cli --features cuda
```

#### AMD ROCm (Experimental)

1. **Install ROCm toolkit** (follow [AMD ROCm install guide](https://rocm.docs.amd.com/en/latest/deploy/linux/quick_start.html))

2. **Build with ROCm feature**:
```bash
cargo build --release --package vtt-cli --features hipblas
```

**Note**: GPU support is optional. CPU-only mode works fine for testing.

---

## Building

### Build All Crates
```bash
cargo build --release
```

### Build CLI Tool Only
```bash
cargo build --release --package vtt-cli
```

### Build with GPU Support
```bash
# NVIDIA
cargo build --release --package vtt-cli --features cuda

# AMD
cargo build --release --package vtt-cli --features hipblas
```

### Output Locations
- CLI tool: `target/release/vtt-cli`
- Library: `target/release/libvtt_core.rlib`
- MCP server: `target/release/vtt-mcp` (coming in Phase 2)

---

## Testing

### 1. Verify Audio Capture

List available audio devices:
```bash
cargo run --release --package vtt-cli -- --list-devices
```

Expected output:
```
Available audio devices:
  [0] "alsa_output.pci-0000_00_1f.3.analog-stereo.monitor" (44100 Hz)
  [1] "alsa_input.pci-0000_00_1f.3.analog-stereo" (48000 Hz) ← Use this
```

### 2. Test Recording + Transcription

Record 5 seconds and transcribe:
```bash
cargo run --release --package vtt-cli -- --duration 5
```

Speak clearly during the recording. You should see:
```
Recording 5 seconds of audio...
Converting audio...
Transcribing with Whisper...

═══════════════════════════════════════════════════════════════
Transcription
═══════════════════════════════════════════════════════════════

hello world this is a test of speech recognition

Duration: 0.00s - 5.00s
═══════════════════════════════════════════════════════════════
```

### 3. Run Benchmark

Measure latency and performance:
```bash
# Using Rust benchmark
cargo run --release --example benchmark --package vtt-core

# Using shell script
./scripts/benchmark.sh
```

Expected results (CPU-only, base model):
- Cold start: 1-3s (model loading)
- Warm start: 500ms-2s per 5s clip
- Memory: 200-500MB

### 4. Test Accuracy

Run the CLI and speak known phrases:
```bash
cargo run --release --package vtt-cli -- --duration 5
```

**Test phrases**:
- "The quick brown fox jumps over the lazy dog"
- "One two three four five six seven eight nine ten"
- "Hello world, this is a test"

Compare output with expected text. Whisper base model should achieve:
- Clear speech: >95% accuracy (WER <5%)
- Noisy environment: 80-90% accuracy (WER 10-20%)

---

## Troubleshooting

### "Model not found" Error

**Problem**: `Error: Model file not found: models/ggml-base.bin`

**Solution**:
```bash
# Create models directory
mkdir -p models

# Download model
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin -O models/ggml-base.bin
```

---

### "No audio device found" Error

**Problem**: `Error: No default audio device available`

**Solution (Linux)**:
```bash
# Check if PipeWire is running
pactl info

# If not, start PipeWire
systemctl --user start pipewire pipewire-pulse

# Check microphone
pactl list sources | grep -A2 "Name:"
```

**Solution (macOS/Windows)**:
- Check System Settings → Privacy → Microphone
- Ensure microphone is not in use by another app

---

### CUDA Build Error

**Problem**: `CMake Error: CUDA Toolkit not found`

**Solution 1**: Install CUDA toolkit
```bash
sudo apt install nvidia-cuda-toolkit  # Ubuntu/Debian
```

**Solution 2**: Build without CUDA (CPU-only)
```bash
cargo build --release --package vtt-cli
# Don't use --features cuda
```

---

### Empty Transcription

**Problem**: Transcription returns empty text

**Possible causes**:
1. **Microphone not recording**: Check audio input device
2. **Too quiet**: Speak louder or closer to mic
3. **Wrong sample rate**: The resampling should handle this automatically

**Debug**:
```bash
# Save audio to file first
cargo run --release --package vtt-cli -- --duration 5 --save-audio test.wav

# Play back to verify recording
aplay test.wav  # Linux
afplay test.wav  # macOS
```

---

### High Latency (>5s)

**Problem**: Transcription takes too long

**Solutions**:
1. **Use smaller model**:
   ```bash
   cargo run --release --package vtt-cli -- --model models/ggml-tiny.bin
   ```

2. **Enable GPU acceleration**:
   ```bash
   cargo build --release --package vtt-cli --features cuda
   ```

3. **Reduce recording duration**:
   ```bash
   cargo run --release --package vtt-cli -- --duration 3
   ```

4. **Increase threads**:
   ```bash
   cargo run --release --package vtt-cli -- --threads 8
   ```

---

### Permission Denied on Audio Device

**Problem**: `Error: Permission denied (audio device)`

**Solution (Linux)**:
```bash
# Add user to audio group
sudo usermod -a -G audio $USER

# Log out and log back in, or run:
newgrp audio
```

---

## Next Steps

After successful setup:

1. **Explore CLI options**: `vtt-cli --help`
2. **Run benchmarks**: `./scripts/benchmark.sh`
3. **Read Phase 1 report**: `docs/PHASE1_REPORT.md` (after completion)
4. **Prepare for Phase 2**: MCP server integration

---

## Additional Resources

- [whisper.cpp GitHub](https://github.com/ggerganov/whisper.cpp)
- [OpenAI Whisper paper](https://arxiv.org/abs/2212.04356)
- [Project README](../README.md)
- [Project Plan](../PLAN.md)
