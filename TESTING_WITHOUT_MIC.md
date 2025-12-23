# Testing VTT-MCP Without a Microphone

Since the microphone is unavailable, here are alternative testing approaches:

## 1. Test with Existing WAV Files

```bash
# Use a test WAV file
ffmpeg -f lavfi -i "sine=frequency=1000:duration=3" /tmp/test_1khz.wav

# TODO: Add file input support to vtt-cli
# This will be needed for the transcribe_clip tool (task vtt-mcp-csh.3.2)
```

## 2. Use a Null Device (for testing pipeline)

```bash
# Create a mock capture that generates silence
# This allows testing the rest of the pipeline

# In vtt-core, add a MockCapture device for testing
```

## 3. Use USB Microphone (if available)

```bash
# List all devices
arecord -l

# Use specific device
arecord -d 3 -f S16_LE -r 16000 -c 1 -D hw:2,0 /tmp/usb_test.wav
```

## 4. Remote Machine with Working Audio

```bash
# SSH to a machine with working microphone
ssh user@machine-with-mic

# Run tests there
cd /path/to/vtt-mcp
cargo run --bin vtt-cli
```

## 5. Test Core Functionality

Without audio capture, you can still test:

- ✅ Device enumeration (`list_devices()`)
- ✅ WAV writing (`write_wav()`)
- ✅ Format conversions
- ✅ Error handling
- ✅ Configuration system (when implemented)

## Current Task Status

| Task | Status | Notes |
|------|--------|-------|
| vtt-mcp-csh.1.3 | ✅ Complete | Implementation done, mic issue is environmental |
| vtt-mcp-csh.1.4 | 🔶 Next | Silero VAD - may need file input for testing |
| vtt-mcp-csh.2.1 | Ready | Whisper setup - doesn't need mic |
