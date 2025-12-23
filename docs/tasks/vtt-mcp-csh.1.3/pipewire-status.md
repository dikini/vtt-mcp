### /home/dikini/Projects/vtt-mcp/docs/PIPEWIRE_STATUS.md
```markdown
1: # PipeWire Migration Status
2: 
3: ## Current Status: ✅ PIPEWIRE NATIVE IMPLEMENTATION COMPLETE
4: 
5: ### What's Working ✓
6: 
7: 1. **Platform Abstraction Layer**
8:    - `AudioCapture` automatically selects PipeWire on Linux, cpal on other platforms
9:    - Located in `crates/vtt-core/src/audio/capture.rs`
10: 
11: 2. **PipeWire Native Audio Capture**
12:    - Full implementation in `crates/vtt-core/src/audio/pipewire_capture.rs`
13:    - Spawns a thread to run the PipeWire event loop
14:    - Non-blocking `start()`/`stop()` interface
15:    - Captures audio via PipeWire's native API
16:    - **Multi-client support**: Multiple processes can capture simultaneously
17: 
18: 3. **cpal Fallback (non-Linux)**
19:    - `CpalCapture` implementation works on Windows/macOS
20: 
21: 4. **Build System**
22:    - PipeWire dependency properly configured for Linux target
23:    - All tests pass
24:    - Only documentation warnings remain
25: 
26: 5. **Dependencies Installed**
27:    - `libpipewire-0.3-dev` and `libspa-0.2-dev`
28:    - `clang` and `libclang-dev` (for bindgen)
29: 
30: ### Implementation Details
31: 
32: **How It Works:**
33: 1. On `start()`: Spawns a thread that:
34:    - Initializes PipeWire
35:    - Creates MainLoop, Context, and connects to the PipeWire daemon
36:    - Creates an audio capture stream
37:    - Sets up callbacks for audio processing
38:    - Runs the blocking `mainloop.run()`
39: 
40: 2. **Audio Process Callback:**
41:    - Called by PipeWire in real-time thread
42:    - Dequeues audio buffers
43:    - Converts f32 LE samples to Rust `f32`
44:    - Extracts first channel (mono) from stereo
45:    - Pushes to shared `Arc<Mutex<Vec<f32>>>` buffer
46: 
47: 3. On `stop()`: 
48:    - Sets `active` flag to false
49:    - Waits for event loop thread to finish
50: 
51: ### Multi-Client Support
52: 
53: ✅ **PipeWire's graph API allows multiple consumers of the same audio source.**
54: 
55: Unlike cpal+ALSA which requires exclusive hardware access, PipeWire:
56: - Acts as an audio server/graph router
57: - Multiple clients can connect to the same microphone
58: - Each client gets its own stream with independent buffers
59: - No "device busy" errors
60: 
61: ### Testing the Implementation
62: 
63: ```bash
64: # Build the CLI
65: cargo build --package vtt-cli
66: 
67: # Run audio capture (will use PipeWire on Linux)
68: cargo run --package vtt-cli -- capture test.wav
69: 
70: # Test multi-client: run multiple instances simultaneously
71: cargo run --package vtt-cli -- capture test1.wav &
72: cargo run --package vtt-cli -- capture test2.wav &
73: ```
74: 
75: ### Files Modified/Created
76: 
77: - `crates/vtt-core/src/audio/pipewire_capture.rs` - **Full PipeWire implementation** ✅
78: - `crates/vtt-core/src/audio/cpal_capture.rs` - Platform-specific fallback
79: - `crates/vtt-core/src/audio/capture.rs` - Platform abstraction layer
80: - `crates/vtt-core/src/audio/mod.rs` - Module exports
81: - `crates/vtt-core/src/audio/format.rs` - Audio format configuration
82: - `crates/vtt-core/Cargo.toml` - PipeWire dependency
83: - `crates/vtt-cli/src/main.rs` - Use `with_format` instead of `with_device`
84: 
85: ### Technical Notes
86: 
87: **Audio Format:**
88: - Default: 48kHz stereo F32 (hardware-native)
89: - Captures mono from first channel
90: - Format is flexible - PipeWire negotiates with the audio graph
91: 
92: **Thread Safety:**
93: - Uses `Arc<Mutex<Vec<f32>>>` for shared buffer
94: - `Arc<Mutex<bool>>>` for active flag
95: - Thread spawned in `start()`, joined in `stop()`
96: 
97: **Limitations:**
98: - `stop() currently waits for thread to finish (could hang if mainloop doesn't exit)
99: - TODO: Add `mainloop.quit()` call for clean shutdown
100: - No error propagation from PipeWire thread to main thread
```
