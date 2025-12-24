# Task Specification: vtt-mcp-csh.3.1
## Setup MCP Server Scaffold

**Parent:** vtt-mcp-csh.3 (Implement MCP Server with Transcription Tools)
**Estimate:** ~2h
**Status:** ✅ COMPLETE (2025-12-24)

### Description
Add rmcp SDK to vtt-mcp/Cargo.toml. Setup stdio transport. Implement basic MCP server struct with tool registration. Test with echo tool.

---

## Implementation Summary

### ✅ Completed (2025-12-24):

#### MCP Protocol Integration
- [x] Rust upgraded to 1.92.0 (from 1.86.0)
- [x] Dependencies added to Cargo.toml (rmcp 0.12, tokio, serde, tracing, uuid, chrono)
- [x] Basic server structure implemented with rmcp framework
- [x] `ServerHandler` trait implemented with `get_info()` method
- [x] stdio transport setup in main.rs using `.serve(stdio())`
- [x] `#[tool_router]` macro integration for all tool methods
- [x] Proper `Parameters<T>` wrapper for tool inputs
- [x] Error handling with `From<VttError> for rmcp::model::ErrorData`

#### Final Integration
- [x] All 6 MCP tools properly integrated with rmcp framework
- [x] Tool routing handled automatically by `#[tool_router]` macro
- [x] Server info and capabilities configured
- [x] Binary compiles successfully
- [x] All tests passing (3/3)

### Files Modified:
- `crates/vtt-mcp/src/server.rs` - Complete rewrite with rmcp integration
- `crates/vtt-mcp/src/error.rs` - Added rmcp ErrorData conversion
- `crates/vtt-mcp/src/main.rs` - stdio transport (already correct)
- `crates/vtt-mcp/Cargo.toml` - Dependencies (already correct)

### Next Steps:
See task vtt-mcp-csh.3.7 for MCP integration tests and full tool implementations.
