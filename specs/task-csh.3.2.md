# Task: Implement transcribe_clip tool

## Description
Create MCP tool for one-shot transcription. Handle duration and model parameters. Wire up to vtt-core transcription engine. Return JSON with text and confidence. ~90min

**Status:** ✅ COMPLETE (2025-12-23) | ✅ MCP Integration Complete (2025-12-24)

---

## Implementation (COMPLETED 2025-12-23)

### What was implemented:

1. **Parameters Struct**
   - audio_file: Path to WAV audio file
   - model_path: Optional Whisper model path
   - use_gpu: Optional GPU acceleration flag
   - threads: Optional thread count

2. **Result Struct**
   - text: Transcribed text
   - confidence: Confidence score (currently None from whisper-rs)
   - start_ms: Start timestamp
   - end_ms: End timestamp

3. **Implementation Details**
   - Uses hound crate to read WAV files
   - Converts i16 samples to f32 normalized to [-1.0, 1.0]
   - Creates WhisperContext on each call (no caching yet)
   - Supports environment variables: WHISPER_MODEL, WHISPER_THREADS
   - Returns structured JSON using Json<TranscriptionResult> wrapper

4. **Error Handling**
   - File not found errors
   - WAV parsing errors
   - Whisper context creation errors
   - Transcription errors

### MCP Protocol Integration (2025-12-24):
- [x] Integrated with rmcp `#[tool_router]` macro
- [x] Uses `Parameters<TranscribeClipParams>` wrapper
- [x] Returns `Result<CallToolResult, McpError>`
- [x] Proper JSON schema generation via `JsonSchema` derive
- [x] Error conversion to `rmcp::model::ErrorData`

### Files Modified:
- crates/vtt-mcp/src/server.rs: Added transcribe_clip tool
- crates/vtt-mcp/Cargo.toml: Added hound dependency

---

## Dependencies Added:
```toml
[dependencies]
hound = "3.5"  # WAV file parsing
```
