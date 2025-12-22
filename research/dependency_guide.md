
# Dependency Guide for Rust STT

## System Dependencies
### Ubuntu/Debian
```bash
sudo apt install libasound2-dev libpipewire-0.3-dev libclang-dev
```

## GPU Support
### CUDA (NVIDIA)
Ensure `nvcc` is in your PATH.
```toml
whisper-rs = { version = "0.15", features = ["cuda"] }
```

### ROCm (AMD)
Ensure ROCm is installed.
```toml
whisper-rs = { version = "0.15", features = ["hipblas"] }
```

## Audio Capture (PipeWire)
CPAL will use PulseAudio or ALSA by default. To ensure it talks to PipeWire, you may need:
```bash
export CPAL_ASOUND_DEVICE="pipewire"
```
