# Task Specification: vtt-mcp-csh.3.4
## Add get_last_transcription tool

**Parent:** vtt-mcp-csh.3 (Implement MCP Server with Transcription Tools)
**Estimate:** ~1h
**Status:** COMPLETE

### Description
Store recent transcriptions in state. Implement get_last_transcription tool. Return text, timestamp, confidence. Add transcript history management.

### Implementation Status

#### ✅ Completed:
- [x] Added global transcription history storage (`transcription_history`)
- [x] Implemented `get_last_transcription` tool with optional session_id parameter
- [x] Store transcriptions from `transcribe_clip` in global history
- [x] Store transcriptions from `stop_listening` in both session and global history
- [x] Return structured JSON with text, timestamp, confidence, and metadata
- [x] Limited history to 100 most recent transcriptions
- [x] Added `transcription_timestamp` to `SessionState`

### Implementation Details

#### Global Transcription History
- **Storage**: Thread-safe `Vec<HistoryEntry>` wrapped in `Arc<Mutex<>>`
- **Ordering**: Most recent first (inserted at index 0)
- **Capacity**: Limited to 100 entries (auto-trimmed)
- **Scope**: Server-wide, includes all transcriptions regardless of session

#### History Entry Structure
```rust
struct HistoryEntry {
    session_id: Uuid,           // Session that produced the transcription
    timestamp: DateTime<Utc>,   // When the transcription was completed
    config: WhisperConfig,      // Model configuration used
    transcription: TranscriptionResult,  // The transcription data
}
```

#### Tool Implemented

##### `get_last_transcription`
**Parameters:**
- `session_id`: Optional - Get transcription from specific session (if not provided, returns most recent across all sessions)

**Returns (when session_id provided):**
```json
{
  "session_id": "uuid-string",
  "timestamp": "2025-12-23T...",
  "text": "Transcribed text here...",
  "confidence": null,
  "start_ms": 0,
  "end_ms": 5000,
  "model_path": "models/ggml-base.bin",
  "language": null
}
```

**Returns (when session_id NOT provided):**
```json
{
  "session_id": "uuid-of-most-recent",
  "timestamp": "2025-12-23T...",
  "text": "Most recent transcription...",
  "confidence": null,
  "start_ms": 0,
  "end_ms": 3000,
  "model_path": "models/ggml-base.bin",
  "language": "en"
}
```

**Error Responses:**
- Invalid session ID format → `invalid_params`
- Session not found → `invalid_params`
- Session has no transcription → `invalid_params`
- No transcriptions available (global) → `internal_error`

### Integration with Existing Tools

#### `transcribe_clip`
- Now stores transcription in global history
- No session association (session_id = new random UUID)
- Enables retrieval of most recent one-shot transcription

#### `stop_listening`
- Stores transcription in both session state and global history
- Preserves session_id for correlation
- Maintains existing behavior

### Files Modified
- `crates/vtt-mcp/src/server.rs` - Added:
  - `transcription_history` field to `VttMcpServer`
  - `HistoryEntry` struct
  - `transcription_timestamp` field to `SessionState`
  - `GetLastTranscriptionParams` struct
  - `LastTranscriptionResult` struct
  - `get_last_transcription` tool method
  - `store_transcription_in_history` helper method
  - Calls to store transcriptions in `transcribe_clip` and `stop_listening`

### Test Results
```
running 3 tests
test tests::test_version ... ok
test server::tests::test_session_status_display ... ok
test server::tests::test_server_creation ... ok

test result: ok. 3 passed; 0 failed; 0 ignored
```

### Usage Examples

#### Example 1: Get most recent transcription (any session)
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_last_transcription","arguments":{}}}' | cargo run --package vtt-mcp
```

#### Example 2: Get transcription from specific session
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_last_transcription","arguments":{"session_id":"550e8400-e29b-41d4-a716-446655440000"}}}' | cargo run --package vtt-mcp
```

### Notes
- Transcription history is in-memory only (not persisted across restarts)
- 100-entry limit prevents unbounded memory growth
- `transcribe_clip` generates a random UUID for history entries (no actual session)
- Sessions remain in `sessions` map after transcription for historical retrieval
- History includes both one-shot transcriptions and session-based transcriptions

### Status
✅ COMPLETE - Transcription history management and get_last_transcription tool implemented successfully.
