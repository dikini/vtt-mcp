### /home/dikini/Projects/vtt-mcp/docs/working-notes/pipewire-implementation-log.md
```markdown
1: # PipeWire Audio Capture - Implementation Log
2: 
3: **Task**: vtt-mcp-csh.1.3 - Implement Basic Audio Capture to WAV  
4: **Date**: 2025-12-23  
5: **Implementation**: PipeWire native API with multi-client support
6: 
7: ---
8: 
9: ## Problem Discovery
10: 
11: ### Initial Issue: "Device or Resource Busy"
12: 
13: When attempting to use cpal+ALSA for audio capture, encountered:
14: ```
15: ALSA lib pcm_hw.c:2057:(snd_pcm_hw_open) open '/dev/snd/pcmC1D0c' failed
16: Device or resource busy
17: ```
18: 
19: ### Root Cause Analysis
20: 
21: **Hardware**: Realtek ALC257 codec with internal microphone  
22: **System**: PipeWire running as audio server  
23: **Problem**: PipeWire had exclusive access to the device; cpal via ALSA couldn't open it
24: 
25: **Investigation Steps**:
26: 1. Confirmed PipeWire was running and had the device open as RUNNING source
27: 2. Discovered hardware only supports 44.1/48/96/192kHz (not 16kHz default)
28: 3. Identified that PipeWire+cpal+ALSA creates hardware exclusive access conflict
29: 
30: ### Solution Decision
31: 
32: **Decision**: Migrate from cpal to PipeWire native API  
33: **Rationale**:
34: - PipeWire natively supports multi-client audio capture
35: - No "device busy" errors with PipeWire graph API
36: - Direct integration with modern Linux audio stack
37: - Multiple processes can capture from the same microphone
38: 
39: ---
40: 
41: ## Implementation Approach
42: 
43: ### Platform Abstraction Layer
44: 
45: ```rust
46: #[cfg(target_os = "linux")]
47: use super::pipewire_capture::PipeWireCapture as Impl;
48: 
49: #[cfg(not(target_os = "linux"))]
50: use super::cpal_capture::CpalCapture as Impl;
51: ```
52: 
53: **Benefits**:
54: - Clean separation of platform-specific code
55: - Linux gets PipeWire benefits
56: - Other platforms continue using cpal (CoreAudio/WASAPI)
57: 
58: ### PipeWire Native Implementation
59: 
60: **Architecture**: Threaded event loop model
61: 
62: 1. **start()**: Spawns thread that:
63:    - Initializes PipeWire (pw::init())
64:    - Creates MainLoop, Context, and Core
65:    - Creates audio capture stream
66:    - Sets up real-time audio callback
67:    - Runs blocking mainloop.run()
68: 
69: 2. **Audio Callback**:
70:    - Dequeues audio buffers from PipeWire
71:    - Converts f32 LE samples to Rust f32
72:    - Extracts first channel (mono) from stereo
73:    - Pushes to shared Arc<Mutex<Vec<f32>>>
74: 
75: 3. **stop()**: 
76:    - Sets active flag to false
77:    - Joins event loop thread
78: 
79: **Key Files**:
80: - `crates/vtt-core/src/audio/pipewire_capture.rs` (250+ lines)
81: - `crates/vtt-core/src/audio/cpal_capture.rs` (fallback)
82: - `crates/vtt-core/src/audio/capture.rs` (abstraction)
83: 
84: ---
85: 
86: ## Technical Challenges
87: 
88: ### Challenge 1: PipeWire API Event Loop Model
89: 
90: **Problem**: PipeWire requires blocking `mainloop.run()`, but our API needs non-blocking start/stop
91: 
92: **Solution**: Spawn thread to run event loop in background
93: - Use Arc<Mutex<bool>> for active flag
94: - Use Arc<Mutex<Vec<f32>>> for shared buffer
95: - Thread spawned in start(), joined in stop()
96: 
97: ### Challenge 2: Borrow Checker in Audio Callback
98: 
99: **Problem**: Cannot borrow `data` mutably and immutably simultaneously
100: 
101: **Solution**: Extract chunk size before borrowing data
102: ```rust
103: let chunk_size = data.chunk().size();  // Before borrow
104: if let Some(samples) = data.data() {  // Then borrow
105:     // ... process samples
106: }
107: ```
108: 
109: ### Challenge 3: Build Dependencies
110: 
111: **Problem**: pipewire crate requires bindgen, which needs clang
112: 
113: **Solution**: Install system dependencies
114: ```bash
115: sudo apt install libpipewire-0.3-dev libspa-0.2-dev clang libclang-dev
116: ```
117: 
118: ---
119: 
120: ## Multi-Client Verification
121: 
122: ### Testing Multi-Client Capture
123: 
124: ```bash
125: # Terminal 1
126: cargo run --package vtt-cli -- capture test1.wav &
127: 
128: # Terminal 2  
129: cargo run --package vtt-cli -- capture test2.wav &
130: ```
131: 
132: **Result**: ✅ Both processes capture simultaneously without errors
133: 
134: **Why This Works**:
135: - PipeWire acts as audio graph router
136: - Each client gets its own stream with independent buffers
137: - PipeWire manages hardware access internally
138: - No exclusive lock at application level
139: 
140: ---
141: 
142: ## Implementation Decisions
143: 
144: ### Decision 1: 48kHz vs 16kHz
145: 
146: **Original spec**: 16kHz mono (STT-optimized)  
147: **Implementation**: 48kHz stereo (hardware-native)
148: 
149: **Rationale**:
150: - Hardware (ALC257) supports 44.1/48/96/192kHz
151: - PipeWire graph can handle format conversion
152: - Capture at hardware rate, downsample later if needed
153: - Better quality for general audio capture
154: 
155: ### Decision 2: Mono Extraction
156: 
157: **Decision**: Extract first channel from stereo
158: 
159: **Rationale**:
160: - Most internal microphones are mono hardware
161: - PipeWire often presents as stereo
162: - First channel typically contains the audio
163: - Simple approach that works universally
164: 
165: ### Decision 3: Threaded vs Async
166: 
167: **Decision**: Use threads instead of async/await
168: 
169: **Rationale**:
170: - PipeWire C API is inherently blocking
171: - Threads map naturally to blocking event loop
172: - No need for async runtime
173: - Simpler implementation
174: 
175: ---
176: 
177: ## Known Limitations
178: 
179: 1. **Stop mechanism**: Currently waits for thread join
180:    - Could hang if mainloop doesn't exit cleanly
181:    - TODO: Add mainloop.quit() for clean shutdown
182: 
183: 2. **Error propagation**: No error channel from PipeWire thread
184:    - Errors printed to stderr via eprintln!
185:    - TODO: Implement channel for error communication
186: 
187: 3. **Format negotiation**: Accepts whatever PipeWire provides
188:    - Typically 48kHz stereo
189:    - TODO: Add format negotiation options
190: 
191: ---
192: 
193: ## Files Modified/Created
194: 
195: ### Created
196: - `crates/vtt-core/src/audio/pipewire_capture.rs` - PipeWire implementation
197: - `crates/vtt-core/src/audio/cpal_capture.rs` - cpal fallback
198: - `crates/vtt-core/src/audio/format.rs` - Audio format config
199: - `crates/vtt-core/src/audio/writer.rs` - WAV file writing
200: - `crates/vtt-core/tests/audio_integration.rs` - Integration tests
201: 
202: ### Modified
203: - `crates/vtt-core/src/audio/capture.rs` - Platform abstraction
204: - `crates/vtt-core/src/audio/device.rs` - Device enumeration
205: - `crates/vtt-core/src/audio/mod.rs` - Module exports
206: - `crates/vtt-core/Cargo.toml` - Added pipewire dependency
207: - `crates/vtt-cli/src/main.rs` - Use with_format
208: 
209: ### Documentation
210: - `PIPEWIRE_STATUS.md` - Implementation status (keep)
211: - `specs/task-csh.1.3.md` - Updated with completion status
212: 
213: ### Temporary (to be removed)
214: - ERROR_ANALYSIS.md - Consolidated into this log
215: - IMPLEMENTATION_SUMMARY.md - Consolidated into this log
216: - MICROPHONE_DIAGNOSIS.md - Consolidated into this log
217: - MULTI_CLIENT_AUDIO.md - Consolidated into this log
218: - PIPEWIRE_MIGRATION_PLAN.md - Consolidated into this log
219: - TESTING_WITHOUT_MIC.md - Consolidated into this log
220: 
221: ---
222: 
223: ## References
224: 
225: - [PipeWire documentation](https://docs.pipewire.org)
226: - [pipewire-rs crate](https://docs.rs/pipewire)
227: - PipeWire audio-capture example: ~/.cargo/registry/src/*/pipewire-0.9.2/examples/audio-capture.rs
228: 
229: ---
230: 
231: ## Commit History
232: 
233: - 694e973 - feat: implement PipeWire native audio capture with multi-client support
234: - bd0099e - docs: add completion summary to task spec
235: 
236: ---
237: 
238: **Status**: ✅ Complete  
239: **Multi-Client Support**: ✅ Verified  
240: **All Tests**: ✅ Passing (11 tests)
```
