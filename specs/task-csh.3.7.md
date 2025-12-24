# Task Specification: vtt-mcp-csh.3.7
## Write MCP Integration Tests

**Parent:** vtt-mcp-csh.3 (Implement MCP Server with Transcription Tools)
**Estimate:** ~2h
**Status:** IN PROGRESS

### Description
Create test script for MCP tool invocation. Test each tool with mock inputs. Verify error handling. Document integration with Goose.

### Background
The MCP server implementation was documented in task specs (3.1-3.5) but the actual `server.rs` implementation file is empty. The integration tests need to be created based on the documented tool specifications.

### MCP Tools to Test

Based on the completed task specs, the following MCP tools should be implemented:

1. **transcribe_clip** - One-shot transcription from WAV file
   - Parameters: audio_file, model_path, use_gpu, threads
   - Returns: text, confidence, start_ms, end_ms

2. **start_listening** - Start audio capture session
   - Parameters: model_path, language, use_gpu, threads, device_name
   - Returns: session_id, status, start_time, config

3. **stop_listening** - Stop audio capture and optionally transcribe
   - Parameters: session_id, transcribe
   - Returns: session_id, status, duration_ms, samples_captured, transcription

4. **get_last_transcription** - Retrieve recent transcription
   - Parameters: session_id (optional)
   - Returns: session_id, timestamp, text, confidence, metadata

5. **list_audio_devices** - Enumerate available audio devices
   - Parameters: none
   - Returns: devices array with name and is_default flags

6. **configure_audio** - Set audio configuration
   - Parameters: device_name, vad_sensitivity
   - Returns: default_device, vad_config, available_devices

### Implementation Plan

#### Phase 1: Create Test Infrastructure
- [ ] Create `crates/vtt-mcp/tests/` directory
- [ ] Add integration test dependencies to Cargo.toml
- [ ] Create test helper module for common test utilities
- [ ] Setup test fixtures (sample audio files, mock configs)

#### Phase 2: Write Unit Tests for Each Tool
- [ ] Test transcribe_clip with valid WAV file
- [ ] Test transcribe_clip with invalid file (error handling)
- [ ] Test start_listening creates valid session
- [ ] Test stop_listening with valid session_id
- [ ] Test stop_listening with invalid session_id (error handling)
- [ ] Test get_last_transcription returns most recent
- [ ] Test get_last_transcription with session_id filter
- [ ] Test list_audio_devices returns device list
- [ ] Test configure_audio updates config
- [ ] Test configure_audio with invalid device (error handling)

#### Phase 3: Integration Tests
- [ ] Test full workflow: start_listening → stop_listening → get_last_transcription
- [ ] Test multiple concurrent sessions
- [ ] Test transcription history persistence across sessions
- [ ] Test audio configuration affects new sessions
- [ ] Test error recovery (e.g., stop already stopped session)

#### Phase 4: Documentation
- [ ] Create `docs/mcp-integration.md` with Goose integration guide
- [ ] Document environment variables and configuration
- [ ] Add example MCP client interactions
- [ ] Document error codes and recovery strategies

### Test Coverage Goals
- All MCP tools invoked with valid parameters
- All error paths tested (invalid params, missing files, etc.)
- Edge cases covered (empty files, concurrent sessions, etc.)
- Integration workflows tested end-to-end

### Success Criteria
- [ ] All tests pass with mock audio data
- [ ] Error handling verified for all tools
- [ ] Integration documentation complete
- [ ] Tests can be run with `cargo test --package vtt-mcp`
- [ ] Goose integration guide published

### Notes
- Tests should use mock/small audio files to avoid long test times
- Some tests may be marked as `#[ignore]` if they require actual audio hardware
- Consider using `tempfile` crate for temporary test audio files
- Tests should be independent and can run in any order

## Implementation Status

### ✅ Completed:

- [x] Created `crates/vtt-mcp/tests/integration_tests.rs` with comprehensive tests
- [x] Implemented VttMcpServer with all 6 MCP tools:
  - transcribe_clip
  - start_listening
  - stop_listening
  - get_last_transcription
  - list_audio_devices
  - configure_audio
- [x] Tests for each tool with valid and invalid inputs
- [x] Error handling verification (invalid session IDs, missing files, etc.)
- [x] Session lifecycle tests (start → stop → retrieve)
- [x] Concurrent session handling tests
- [x] VAD configuration and clamping tests
- [x] Audio device validation tests

### Test Coverage

The integration tests cover:
1. **Server initialization** - Verify clean startup
2. **Audio device management** - List and configure devices
3. **Session lifecycle** - Create, stop, and manage sessions
4. **Error handling** - Invalid inputs, missing files, bad session IDs
5. **Concurrent operations** - Multiple active sessions
6. **History management** - Transcription history storage

### Running Tests

```bash
# Run all integration tests
cargo test --package vtt-mcp --test integration_tests

# Run specific test
cargo test --package vtt-mcp test_server_creation -- --exact

# Run with output
cargo test --package vtt-mcp -- --nocapture
```

### Notes

- The VttMcpServer implementation is complete but not yet integrated with rmcp protocol traits
- Tests verify core functionality independent of MCP protocol layer
- Some tests may be skipped on systems without audio hardware
- Full MCP integration requires implementing rmcp::Service trait

### Next Steps for Full MCP Integration

To complete the MCP server implementation:
1. Implement rmcp::Service trait for VttMcpServer
2. Register tools using rmcp's #[tool] macro
3. Handle MCP protocol messages (initialize, tools/list, tools/call)
4. Implement proper JSON-RPC response formatting
5. Add resource support for live transcription streaming

## Status
✅ IN PROGRESS - Core implementation and tests complete. MCP protocol integration pending.
