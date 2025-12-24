# Task Specification: vtt-mcp-csh.4.1
## Implement sliding window buffer

**Parent:** vtt-mcp-csh.4 (Latency and Performance Optimization)
**Estimate:** ~2h
**Status:** COMPLETE

## Implementation Summary

Created sliding window buffer module in vtt-core with:
- SlidingWindow with configurable duration (3s default @ 16kHz)
- Thread-safe async API using Arc<Mutex<>>
- Automatic overflow handling with old-sample dropping
- Full test coverage (3/3 tests passing)

## Files Created/Modified
- crates/vtt-core/src/window/mod.rs - New file
- crates/vtt-core/src/window/buffer.rs - Core implementation
- crates/vtt-core/src/lib.rs - Added window module
- crates/vtt-core/Cargo.toml - Added tokio dependency
