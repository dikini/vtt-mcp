# VTT-MCP Troubleshooting Guide

Solutions to common issues when using the VTT-MCP server.

## Table of Contents

- [Installation Issues](#installation-issues)
- [Audio Issues](#audio-issues)
- [Transcription Issues](#transcription-issues)
- [Performance Issues](#performance-issues)
- [Language Issues](#language-issues)
- [GPU Issues](#gpu-issues)
- [MCP Connection Issues](#mcp-connection-issues)
- [FAQ](#faq)

---

## Installation Issues

### Rust Version Too Old

**Error:** `error: package `vtt-mcp` requires Rust >= 1.70`

**Solution:**
```bash
# Update Rust via rustup
rustup update stable

# Verify version
rustc --version  # Should be 1.70+
```

---

### Build Errors - Missing Dependencies

**Error:** `error: linking with `cc` failed: exit code: 1`

**Solution:**
```bash
# Install build essentials (Linux)
sudo apt install build-essential pkg-config  # Debian/Ubuntu
sudo pacman -S base-devel  # Arch

# Install clang (macOS)
xcode-select --install
```

---

### Whisper.cpp Compilation Error

**Error:** `CMake Error: Could not find CMAKE_ROOT`

**Solution:**
```bash
# Install CMake
sudo apt install cmake  # Debian/Ubuntu
brew install cmake  # macOS

# Clean and rebuild
cargo clean
cargo build --release
```

---

## Audio Issues

### "No audio device found"

**Symptoms:**
- `DeviceNotFound` error
- `No default input device available`

**Linux Solutions:**
```bash
# 1. Check if PipeWire is running
pactl info

# 2. Start PipeWire if not running
systemctl --user start pipewire pipewire-pulse

# 3. Verify microphone
pactl list sources | grep -A2 "Name:"

# 4. Check permissions
groups | grep audio  # Should include 'audio'
sudo usermod -a -G audio $USER  # Add if missing

# 5. Restart audio system
systemctl --user restart pipewire wireplumber
```

**macOS Solutions:**
```bash
# 1. Check System Preferences → Sound → Input
# 2. Ensure microphone is selected
# 3. Check microphone permissions
# System Preferences → Security & Privacy → Privacy → Microphone
```

**Windows Solutions:**
```powershell
# 1. Check Sound Settings → Input
# 2. Ensure microphone is enabled
# 3. Test microphone in Settings
```

---

### "Permission denied" on Audio Device

**Error:** `Permission denied (audio device)`

**Solution (Linux):**
```bash
# Add user to audio group
sudo usermod -a -G audio $USER

# Log out and log back in, or run:
newgrp audio

# Verify
groups
```

---

### Audio Recording But No Transcription

**Symptoms:**
- Recording succeeds
- Transcription returns empty or partial text

**Diagnosis:**
```bash
# Test audio capture
cargo run --release --package vtt-cli -- --duration 5 --save-audio test.wav

# Play back recording
aplay test.wav  # Linux
afplay test.wav  # macOS
```

**Solutions:**
1. **Too quiet**: Speak louder or closer to microphone
2. **Wrong device**: Select correct audio device
   ```typescript
   const devices = await mcp.callTool("list_audio_devices", {});
   await mcp.callTool("configure_audio", {
     default_device: devices.available_devices[2].name  // Use device 2
   });
   ```
3. **VAD threshold**: Adjust sensitivity
   ```typescript
   await mcp.callTool("configure_audio", {
     vad_config: { threshold: 0.005 }  // More sensitive
   });
   ```

---

## Transcription Issues

### "Model file not found"

**Error:** `Model file not found: models/ggml-base.bin`

**Solution:**
```bash
# Create models directory
mkdir -p models

# Download base model
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin \
  -O models/ggml-base.bin

# Verify
ls -lh models/
```

---

### Poor Transcription Accuracy

**Symptoms:**
- Many transcription errors
- Wrong words detected

**Solutions:**

1. **Use larger model**:
   ```bash
   wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin \
     -O models/ggml-small.bin
   ```

2. **Specify language**:
   ```typescript
   await mcp.callTool("transcribe_clip", {
     audio_file: "/path/to/audio.wav",
     language: "en"  // Specify for better accuracy
   });
   ```

3. **Improve audio quality**:
   - Use better microphone
   - Reduce background noise
   - Speak clearly

4. **Enable GPU** (if available):
   ```bash
   cargo build --release --features cuda
   ```

---

### Wrong Language Detected

**Symptoms:**
- Transcription in wrong language
- Mixed languages in result

**Solutions:**

1. **Specify language explicitly**:
   ```typescript
   const result = await mcp.callTool("transcribe_clip", {
     audio_file: "/path/to/audio.wav",
     language: "es"  // Spanish
   });
   ```

2. **Use language-specific model** (if available):
   ```bash
   # Download Chinese-specific model
   wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base-zh.bin \
     -O models/ggml-base-zh.bin
   ```

---

## Performance Issues

### Slow Transcription (>5s for 5s audio)

**Symptoms:**
- High latency
- Slow processing

**Solutions:**

1. **Enable GPU acceleration**:
   ```bash
   # Install CUDA (NVIDIA)
   sudo apt install nvidia-cuda-toolkit

   # Rebuild with CUDA
   cargo build --release --features cuda
   ```

2. **Use smaller model**:
   ```bash
   # Download tiny model (75MB)
   wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin \
     -O models/ggml-tiny.bin

   # Use in code
   await mcp.callTool("transcribe_clip", {
     audio_file: "/path/to/audio.wav",
     model_path: "models/ggml-tiny.bin"
   });
   ```

3. **Increase thread count**:
   ```toml
   # In ~/.config/vtt-mcp/config.toml
   [whisper]
   threads = 8  # Increase from default 4
   ```

4. **Reduce recording duration**:
   ```typescript
   // Transcribe shorter clips
   await mcp.callTool("transcribe_clip", {
     audio_file: "/path/to/audio.wav"  // Shorter file
   });
   ```

---

### High Memory Usage (>1GB)

**Symptoms:**
- System slows down
- OOM errors

**Solutions:**

1. **Use smaller model**:
   - Tiny: ~75MB
   - Base: ~140MB
   - Small: ~460MB

2. **Limit concurrent sessions**:
   ```toml
   # In ~/.config/vtt-mcp/config.toml
   [whisper.memory]
   max_sessions = 2  # Reduce from default 10
   ```

3. **Enable idle timeout**:
   ```toml
   [whisper.memory]
   idle_timeout_secs = 60  # Unload after 60 seconds
   ```

---

## Language Issues

### Language Not Supported

**Symptoms:**
- `Invalid language code` error
- Language not in list

**Solution:**

Check supported languages:
```typescript
const result = await mcp.callTool("list_languages", {});
console.log(result.languages.map(l => l.code));
```

Supported: auto, en, es, fr, de, it, pt, zh, ja, ko, ru, ar, hi

---

### Poor Accuracy in Specific Language

**Solutions:**

1. **Specify language explicitly**:
   ```typescript
   await mcp.callTool("transcribe_clip", {
     audio_file: "/path/to/audio.wav",
     language: "ja"  // Japanese
   });
   ```

2. **Use larger model** (better for all languages):
   ```bash
   wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin \
     -O models/ggml-medium.bin
   ```

---

## GPU Issues

### CUDA Not Detected

**Symptoms:**
- GPU acceleration not working
- `GPU not available` message

**Diagnosis:**
```bash
# Check NVIDIA driver
nvidia-smi

# Check CUDA
nvcc --version
```

**Solutions:**

1. **Install NVIDIA drivers**:
   ```bash
   sudo apt install nvidia-driver-535  # Ubuntu/Debian
   ```

2. **Install CUDA toolkit**:
   ```bash
   sudo apt install nvidia-cuda-toolkit
   ```

3. **Rebuild with CUDA**:
   ```bash
   cargo clean
   cargo build --release --features cuda
   ```

---

### GPU Out of Memory

**Error:** `CUDA out of memory`

**Solutions:**

1. **Use smaller model**:
   ```bash
   # Base model (140MB) instead of Large (3GB)
   ```

2. **Reduce concurrent sessions**:
   ```toml
   [whisper.memory]
   max_sessions = 1
   ```

3. **Disable GPU** (use CPU):
   ```bash
   export VTT_NO_GPU=1
   ```

---

## MCP Connection Issues

### Server Not Responding

**Symptoms:**
- Timeout errors
- No response from tools

**Diagnosis:**
```bash
# Check if server is running
ps aux | grep vtt-mcp

# Test server directly
cargo run --release --package vtt-mcp
```

**Solutions:**

1. **Restart server**:
   ```bash
   # Kill existing server
   pkill vtt-mcp

   # Start fresh
   cargo run --release --package vtt-mcp
   ```

2. **Check logs**:
   ```bash
   # Enable debug logging
   RUST_LOG=debug cargo run --release --package vtt-mcp
   ```

3. **Verify stdio transport**:
   - Ensure no other process is using stdin/stdout
   - Check for buffering issues

---

### Tool Not Found

**Error:** `Tool 'transcribe_clip' not found`

**Solutions:**

1. **Check server version**:
   ```bash
   cargo run --release --package vtt-mcp -- --version
   ```

2. **List available tools**:
   ```typescript
   const tools = await mcp.listTools();
   console.log(tools.map(t => t.name));
   ```

3. **Update to latest version**:
   ```bash
   git pull
   cargo build --release
   ```

---

## FAQ

### Q: Can I use VTT-MCP offline?

**A:** Yes! VTT-MCP runs entirely locally. Just download the Whisper model once, and you can transcribe without internet.

---

### Q: What's the best model for accuracy?

**A:** The `large-v3` model provides the best accuracy but requires more CPU/GPU power. For most use cases, `base` or `small` models offer a good balance.

---

### Q: Can I transcribe multiple languages in one file?

**A:** Set `language: "auto"` to enable automatic language detection. However, accuracy is best when you specify the primary language.

---

### Q: How do I improve transcription in noisy environments?

**A:** 
- Use a noise-canceling microphone
- Adjust VAD threshold: `configure_audio({ vad_config: { threshold: 0.02 }})`
- Move closer to the microphone
- Use the `small` or `medium` model for better noise handling

---

### Q: Can I use VTT-MCP with other AI assistants?

**A:** Yes! VTT-MCP implements the standard MCP protocol and should work with any MCP-compatible client, not just Goose.

---

### Q: How do I reduce transcription latency?

**A:**
- Enable GPU acceleration (3-5x speedup)
- Use the `tiny` model (fastest, ~2x quicker than base)
- Increase thread count in config
- Use shorter audio clips

---

### Q: Is my audio data sent to the cloud?

**A:** No. All processing happens locally on your machine. No data is sent to external services.

---

### Q: Can I save transcriptions to a file?

**A:** Yes! Use the transcript history resource:
```typescript
const history = await mcp.readResource("transcript://history");
fs.writeFileSync("transcripts.json", history);
```

---

### Q: How do I switch between audio devices?

**A:**
```typescript
// List devices
const devices = await mcp.callTool("list_audio_devices", {});

// Set default device
await mcp.callTool("configure_audio", {
  default_device: "Device Name Here"
});
```

---

## Getting Help

If you're still experiencing issues:

1. **Check logs**: `RUST_LOG=debug cargo run --release --package vtt-mcp`
2. **Search issues**: [GitHub Issues](https://github.com/your-repo/vtt-mcp/issues)
3. **Create issue**: Include error messages, OS, and hardware specs
4. **Documentation**: [README](../README.md) | [API Reference](API.md) | [Architecture](ARCHITECTURE.md)

---

## Diagnostic Commands

```bash
# System info
uname -a
rustc --version
cargo --version

# Audio (Linux)
pactl info
pactl list sources

# Audio (macOS)
system_profiler SPAudioDataType

# GPU (NVIDIA)
nvidia-smi

# GPU (AMD)
rocm-smi

# Disk space
df -h

# Memory
free -h  # Linux
vm_stat  # macOS
```
