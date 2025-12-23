# PipeWire Migration Status

## Current Status: ✅ PIPEWIRE NATIVE IMPLEMENTATION COMPLETE

### What's Working ✓

1. **Platform Abstraction Layer**
   - `AudioCapture` automatically selects PipeWire on Linux, cpal on other platforms
   - Located in `crates/vtt-core/src/audio/capture.rs`

2. **PipeWire Native Audio Capture**
   - Full implementation in `crates/vtt-core/src/audio/pipewire_capture.rs`
   - Spawns a thread to run the PipeWire event loop
   - Non-blocking `start()`/`stop()` interface
   - Captures audio via PipeWire's native API
   - **Multi-client support**: Multiple processes can capture simultaneously

3. **cpal Fallback (non-Linux)**
   - `CpalCapture` implementation works on Windows/macOS

4. **Build System**
   - PipeWire dependency properly configured for Linux target
   - All tests pass
   - Only documentation warnings remain

5. **Dependencies Installed**
   - `libpipewire-0.3-dev` and `libspa-0.2-dev`
   - `clang` and `libclang-dev` (for bindgen)

### Implementation Details

**How It Works:**
1. On `start()`: Spawns a thread that:
   - Initializes PipeWire
   - Creates MainLoop, Context, and connects to the PipeWire daemon
   - Creates an audio capture stream
   - Sets up callbacks for audio processing
   - Runs the blocking `mainloop.run()`

2. **Audio Process Callback:**
   - Called by PipeWire in real-time thread
   - Dequeues audio buffers
   - Converts f32 LE samples to Rust `f32`
   - Extracts first channel (mono) from stereo
   - Pushes to shared `Arc<Mutex<Vec<f32>>>` buffer

3. On `stop()`: 
   - Sets `active` flag to false
   - Waits for event loop thread to finish

### Multi-Client Support

✅ **PipeWire's graph API allows multiple consumers of the same audio source.**

Unlike cpal+ALSA which requires exclusive hardware access, PipeWire:
- Acts as an audio server/graph router
- Multiple clients can connect to the same microphone
- Each client gets its own stream with independent buffers
- No "device busy" errors

### Testing the Implementation

```bash
# Build the CLI
cargo build --package vtt-cli

# Run audio capture (will use PipeWire on Linux)
cargo run --package vtt-cli -- capture test.wav

# Test multi-client: run multiple instances simultaneously
cargo run --package vtt-cli -- capture test1.wav &
cargo run --package vtt-cli -- capture test2.wav &
```

### Files Modified/Created

- `crates/vtt-core/src/audio/pipewire_capture.rs` - **Full PipeWire implementation** ✅
- `crates/vtt-core/src/audio/cpal_capture.rs` - Platform-specific fallback
- `crates/vtt-core/src/audio/capture.rs` - Platform abstraction layer
- `crates/vtt-core/src/audio/mod.rs` - Module exports
- `crates/vtt-core/src/audio/format.rs` - Audio format configuration
- `crates/vtt-core/Cargo.toml` - PipeWire dependency
- `crates/vtt-cli/src/main.rs` - Use `with_format` instead of `with_device`

### Technical Notes

**Audio Format:**
- Default: 48kHz stereo F32 (hardware-native)
- Captures mono from first channel
- Format is flexible - PipeWire negotiates with the audio graph

**Thread Safety:**
- Uses `Arc<Mutex<Vec<f32>>>` for shared buffer
- `Arc<Mutex<bool>>>` for active flag
- Thread spawned in `start()`, joined in `stop()`

**Limitations:**
- `stop() currently waits for thread to finish (could hang if mainloop doesn't exit)
- TODO: Add `mainloop.quit()` call for clean shutdown
- No error propagation from PipeWire thread to main thread
