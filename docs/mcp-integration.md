# VTT MCP Server Integration Guide

> **Status:** ✅ Implementation Complete (2025-12-24)  
> **Build:** Compiling successfully with all 6 MCP tools integrated  
> **Testing:** Ready for end-to-end testing with MCP clients

## Overview

The VTT (Voice-to-Text) MCP server provides speech-to-text functionality via the Model Context Protocol (MCP). This guide covers integration with AI assistants like Goose.

### /home/dikini/Projects/vtt-mcp/./docs/mcp-integration.md
```markdown
1: # VTT MCP Server Integration Guide
2: 
3: ## Overview
4: 
5: The VTT (Voice-to-Text) MCP server provides speech-to-text functionality via the Model Context Protocol (MCP). This guide covers integration with AI assistants like Goose.
6: 
7: ## Architecture
8: 
9: ```
10: ┌─────────────────┐     MCP Protocol      ┌──────────────────┐
11: │   Goose/AI      │ ◄────────────────────► │   VTT MCP Server │
12: │   Assistant     │      (stdio/JSON-RPC)  │   (vtt-mcp)      │
13: └─────────────────┘                        └────────┬─────────┘
14:                                                    │
15:                                                    ▼
16:                                           ┌─────────────────┐
17:                                           │   vtt-core      │
18:                                           │   (Whisper)     │
19:                                           └─────────────────┘
20: ```
21: 
22: ## Configuration
23: 
24: ### Environment Variables
25: 
26: | Variable | Description | Default |
27: |----------|-------------|---------|
28: | `WHISPER_MODEL` | Path to Whisper model file | `models/ggml-base.bin` |
29: | `WHISPER_THREADS` | Number of CPU threads | `<physical cores>` |
30: | `WHISPER_USE_GPU` | Enable GPU acceleration | `true` |
31: | `RUST_LOG` | Log level filter | `info,vtt_mcp=debug` |
32: 
33: ### Server Startup
34: 
35: ```bash
36: # Start server with default settings
37: cargo run --package vtt-mcp
38: 
39: # Start with custom model
40: WHISPER_MODEL=models/ggml-tiny.bin cargo run --package vtt-mcp
41: 
42: # Start with logging
43: RUST_LOG=debug cargo run --package vtt-mcp
44: ```
45: 
46: ## MCP Tools
47: 
48: ### 1. transcribe_clip
49: 
50: One-shot transcription from a WAV audio file.
51: 
52: **Parameters:**
53: ```json
54: {
55:   "audio_file": "/path/to/audio.wav",
56:   "model_path": "models/ggml-base.bin",  // optional
57:   "use_gpu": true,                       // optional
58:   "threads": 4                           // optional
59: }
60: ```
61: 
62: **Response:**
63: ```json
64: {
65:   "text": "Transcribed text here...",
66:   "confidence": 0.95,
67:   "start_ms": 0,
68:   "end_ms": 5230
69: }
70: ```
71: 
72: ### 2. start_listening
73: 
74: Start an audio capture session for continuous recording.
75: 
76: **Parameters:**
77: ```json
78: {
79:   "model_path": "models/ggml-base.bin",  // optional
80:   "language": "en",                      // optional
81:   "use_gpu": true,                       // optional
82:   "threads": 4,                          // optional
83:   "device_name": "Built-in Audio"        // optional (future)
84: }
85: ```
86: 
87: **Response:**
88: ```json
89: {
90:   "session_id": "550e8400-e29b-41d4-a716-446655440000",
91:   "status": "listening",
92:   "start_time": "2025-12-24T01:00:00Z",
93:   "model_path": "models/ggml-base.bin",
94:   "language": "en",
95:   "use_gpu": true
96: }
97: ```
98: 
99: ### 3. stop_listening
100: 
101: Stop an active listening session and optionally transcribe.
102: 
103: **Parameters:**
104: ```json
105: {
106:   "session_id": "550e8400-e29b-41d4-a716-446655440000",
107:   "transcribe": true
108: }
109: ```
110: 
111: **Response:**
112: ```json
113: {
114:   "session_id": "550e8400-e29b-41d4-a716-446655440000",
115:   "status": "transcribed",
116:   "duration_ms": 5000,
117:   "samples_captured": 80000,
118:   "transcription": {
119:     "text": "Transcribed audio...",
120:     "confidence": 0.92,
121:     "start_ms": 0,
122:     "end_ms": 5000
123:   },
124:   "error": null
125: }
126: ```
127: 
128: ### 4. get_last_transcription
129: 
130: Retrieve the most recent transcription or a session-specific one.
131: 
132: **Parameters (most recent):**
133: ```json
134: {}
135: ```
136: 
137: **Parameters (specific session):**
138: ```json
139: {
140:   "session_id": "550e8400-e29b-41d4-a716-446655440000"
141: }
142: ```
143: 
144: **Response:**
145: ```json
146: {
147:   "session_id": "550e8400-e29b-41d4-a716-446655440000",
148:   "timestamp": "2025-12-24T01:00:05Z",
149:   "text": "Transcribed text...",
150:   "confidence": 0.92,
151:   "start_ms": 0,
152:   "end_ms": 5000,
153:   "model_path": "models/ggml-base.bin",
154:   "language": "en"
155: }
156: ```
157: 
158: ### 5. list_audio_devices
159: 
160: Enumerate available audio input devices.
161: 
162: **Parameters:**
163: ```json
164: {}
165: ```
166: 
167: **Response:**
168: ```json
169: {
170:   "devices": [
171:     {
172:       "name": "Built-in Audio",
173:       "is_default": true
174:     },
175:     {
176:       "name": "USB Microphone",
177:       "is_default": false
178:     }
179:   ],
180:   "default_device": "Built-in Audio"
181: }
182: ```
183: 
184: ### 6. configure_audio
185: 
186: Configure audio device and VAD settings.
187: 
188: **Parameters:**
189: ```json
190: {
191:   "device_name": "USB Microphone",      // optional
192:   "vad_sensitivity": 0.01              // optional, 0.0-1.0
193: }
194: ```
195: 
196: **Response:**
197: ```json
198: {
199:   "default_device": "USB Microphone",
200:   "vad_config": {
201:     "energy_threshold": 0.01,
202:     "speech_frames_threshold": 3,
203:     "silence_frames_threshold": 10,
204:     "min_speech_duration": 30
205:   },
206:   "available_devices": [
207:     {
208:       "name": "Built-in Audio",
209:       "is_default": false
210:     },
211:     {
212:       "name": "USB Microphone",
213:       "is_default": true
214:     }
215:   ]
216: }
217: ```
218: 
219: ## Error Handling
220: 
221: ### Error Codes
222: 
223: | Code | Description |
224: |------|-------------|
225: | `INVALID_PARAMS` | Invalid or missing parameters |
226: | `DEVICE_NOT_FOUND` | Audio device not found |
227: | `NO_AUDIO_DATA` | No audio data available |
228: | `MODEL_ERROR` | Whisper model error |
229: | `SESSION_ERROR` | Session management error |
230: | `AUDIO_ERROR` | Audio capture error |
231: | `TRANSCRIPTION_ERROR` | Transcription failed |
232: | `IO_ERROR` | File I/O error |
233: | `INTERNAL_ERROR` | Internal server error |
234: 
235: ### Error Response Format
236: 
237: ```json
238: {
239:   "error": {
240:     "code": -32602,
241:     "message": "Invalid params: Audio file not found: /path/to/file.wav",
242:     "data": null
243:   },
244:   "id": 1
245: }
246: ```
247: 
248: ## Integration with Goose
249: 
250: ### MCP Client Configuration
251: 
252: Goose can connect to the VTT MCP server via stdio:
253: 
254: ```toml
255: # goose config
256: [mcp.vtt-server]
257: command = "cargo run --package vtt-mcp"
258: args = []
259: env = { WHISPER_MODEL = "models/ggml-base.bin" }
260: ```
261: 
262: ### Example Usage
263: 
264: ```python
265: # Pseudo-code for Goose integration
266: from goose import MCPClient
267: 
268: client = MCPClient("vtt-server")
269: 
270: # Start listening
271: session = client.call_tool("start_listening", {
272:     "language": "en"
273: })
274: session_id = session["session_id"]
275: 
276: # ... wait for speech ...
277: 
278: # Stop and transcribe
279: result = client.call_tool("stop_listening", {
280:     "session_id": session_id,
281:     "transcribe": True
282: })
283: 
284: text = result["transcription"]["text"]
285: print(f"Transcribed: {text}")
286: ```
287: 
288: ## Testing
289: 
290: ### Unit Tests
291: 
292: ```bash
293: # Run all tests
294: cargo test --package vtt-mcp
295: 
296: # Run integration tests
297: cargo test --package vtt-mcp --test integration_tests
298: 
299: # Run with output
300: cargo test --package vtt-mcp -- --nocapture
301: ```
302: 
303: ### Manual Testing
304: 
305: ```bash
306: # List tools
307: echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | cargo run --package vtt-mcp
308: 
309: # Call a tool
310: echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"list_audio_devices","arguments":{}}}' | cargo run --package vtt-mcp
311: ```
312: 
313: ## Performance Considerations
314: 
315: - **Latency**: Target <1s from audio end to transcription
316: - **Memory**: Whisper model ~1GB for base model
317: - **CPU**: 4+ threads recommended for real-time
318: - **GPU**: Recommended for faster transcription
319: 
320: ## Troubleshooting
321: 
322: ### Common Issues
323: 
324: 1. **Model not found**
325:    - Ensure `WHISPER_MODEL` points to valid model file
326:    - Download models from [Whisper.cpp](https://github.com/ggerganov/whisper.cpp)
327: 
328: 2. **No audio devices**
329:    - Check system audio configuration
330:    - Verify microphone permissions
331: 
332: 3. **Transcription errors**
333:    - Check audio quality and sample rate
334:    - Ensure 16kHz mono format for best results
335: 
336: 4. **GPU errors**
337:    - Set `WHISPER_USE_GPU=false` to use CPU
338:    - Verify CUDA installation
339: 
340: ## Development Status
341: 
342: ### Completed (vtt-mcp-csh.3.7)
343: - ✅ All 6 MCP tools implemented
344: - ✅ Session management
345: - ✅ Transcription history
346: - ✅ Audio configuration
347: - ✅ Error handling
348: - ✅ Integration tests
349: 
350: ### Pending
351: - ⏳ MCP protocol trait implementation (rmcp::Service)
352: - ⏳ Tool registration with rmcp macros
353: - ⏳ Resource support for live streaming
354: - ⏳ Incremental transcription
355: 
356: ## Resources
357: 
358: - [MCP Specification](https://modelcontextprotocol.io)
359: - [Whisper.cpp](https://github.com/ggerganov/whisper.cpp)
360: - [Goose Documentation](https://goose.dev)
```
