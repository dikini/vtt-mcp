# Task Specification: vtt-mcp-csh.4.3
## Implement transcript://live resource

**Parent:** vtt-mcp-csh.4 (Latency and Performance Optimization)
**Estimate:** ~2h
**Status:** PENDING

### Description
Add MCP resource for real-time streaming. Implement subscription mechanism. Push incremental updates to clients. Handle client disconnection.

### Design

Created `crates/vtt-mcp/src/live.rs` module with:
- `LiveSession` - Manages subscriber list via broadcast channel
- `TranscriptUpdate` - Struct for live transcript updates with timestamps
- `LiveTranscriptionManager` - Manages multiple live sessions

### Implementation Notes

The live streaming infrastructure is ready but needs:
1. Integration with VttMcpServer (add `live_manager` field)
2. Resource handler implementation (list_resources, read_resource)
3. Session management in start_listening tool
4. Broadcast integration with incremental transcription

### Next Steps

To complete this task:
1. Add live_manager to VttMcpServer struct
2. Implement MCP resource handlers for transcript://live/{sessionId}
3. Integrate with IncrementalTranscriber to push updates
4. Add subscribe mechanism for clients
