# Task Specification: vtt-mcp-csh.3.3
## Implement start_listening/stop_listening

**Parent:** vtt-mcp-csh.3 (Implement MCP Server with Transcription Tools)
**Estimate:** ~2h
**Status:** COMPLETE

### Description
Add session management with UUIDs. Implement start_listening tool with model/language params. Add stop_listening with session_id param. Return session state.

### Implementation Status

#### ✅ Completed:
- [x] Added session state management with UUID-based session tracking
- [x] Implemented `start_listening` tool with optional model_path, language, use_gpu, threads, and device_name parameters
- [x] Implemented `stop_listening` tool with session_id and optional transcribe parameter
- [x] Added thread-safe session storage using `Arc<Mutex<HashMap<Uuid, SessionState>>>`
- [x] Integrated audio capture via `AudioCapture`
- [x] Automatic transcription on stop (when transcribe=true)
- [x] Session status tracking (listening, stopped, transcribed, error)
- [x] Return structured JSON responses with session state

### Implementation Details

#### Session Management
- **Session ID**: UUID-based unique identifiers
- **Session State**: Stored in thread-safe `HashMap`
- **Status Lifecycle**: 
  1. `listening` - Active recording
  2. `stopped` - Recording stopped, awaiting transcription
  3. `transcribed` - Transcription complete
  4. `error` - An error occurred

#### Tools Implemented

##### `start_listening`
**Parameters:**
- `model_path`: Optional - Path to Whisper model (default: from env or "models/ggml-base.bin")
- `language`: Optional - Language code (default: auto-detect)
- `use_gpu`: Optional - GPU acceleration (default: true)
- `threads`: Optional - Thread count (default: CPU count)
- `device_name`: Optional - Audio device name (reserved for future use)

**Returns:**
```json
{
  "session_id": "uuid-string",
  "status": "listening",
  "start_time": "2025-12-23T...",
  "model_path": "models/ggml-base.bin",
  "language": null,
  "use_gpu": true
}
```

##### `stop_listening`
**Parameters:**
- `session_id`: Required - UUID from start_listening
- `transcribe`: Optional - Whether to transcribe (default: true)

**Returns:**
```json
{
  "session_id": "uuid-string",
  "status": "transcribed",
  "duration_ms": 5000,
  "samples_captured": 80000,
  "transcription": {
    "text": "Transcribed text here...",
    "confidence": null,
    "start_ms": 0,
    "end_ms": 5000
  },
  "error": null
}
```

### Additional Changes Made

#### vtt-core Changes
- Added `#[derive(Clone)]` to `WhisperConfig`
- Added `#[derive(Debug, Clone)]` to `AudioCapture`
- Implemented manual `Clone` for `PipeWireCapture` (shares buffer/active state)
- Implemented manual `Clone` for `CpalCapture` (shares buffer, drops stream)

#### Files Modified
- `crates/vtt-core/src/whisper/config.rs` - Added Clone derive
- `crates/vtt-core/src/audio/capture.rs` - Added Debug/Clone derives
- `crates/vtt-core/src/audio/pipewire_capture.rs` - Manual Clone impl
- `crates/vtt-core/src/audio/cpal_capture.rs` - Manual Clone impl
- `crates/vtt-mcp/src/server.rs` - Added start_listening/stop_listening tools

### Test Results
```
running 3 tests
test tests::test_version ... ok
test server::tests::test_session_status_display ... ok
test server::tests::test_server_creation ... ok

test result: ok. 3 passed; 0 failed; 0 ignored
```

### Usage Example
```bash
# Start listening
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"start_listening","arguments":{"model_path":"models/ggml-base.bin","use_gpu":true}}}' | cargo run --package vtt-mcp

# Wait for audio capture...

# Stop listening and transcribe
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"stop_listening","arguments":{"session_id":"uuid-from-start","transcribe":true}}}' | cargo run --package vtt-mcp
```

### Notes
- Sessions are kept in memory after stop for potential retrieval (for future `get_last_transcription` tool)
- Audio capture runs on a separate thread (via PipeWire/cpal)
- The `device_name` parameter is reserved for future device selection support
- Session IDs are standard UUIDs (version 4, random)

### Status
✅ COMPLETE - Session management and listening tools implemented successfully.
