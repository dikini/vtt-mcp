# Task Spec: vtt-mcp-csh.4.5 - GPU Memory Optimization

## Status
🔄 In Progress

## Description
Profile VRAM usage and implement memory management strategies to optimize GPU memory consumption for the Whisper model.

## Implementation Plan

### 1. Memory Profiling
- Add VRAM usage tracking to WhisperContext
- Monitor memory allocation/deallocation
- Track peak memory usage during transcription

### 2. Memory Management Strategies
- Implement model unloading when idle
- Add configurable idle timeout
- Support for multiple concurrent sessions with shared model

### 3. Monitoring
- Add memory usage logging
- Track memory per session
- Alert on high memory usage

## Files to Modify
- `crates/vtt-core/src/whisper/context.rs` - Add memory tracking
- `crates/vtt-core/src/whisper/config.rs` - Add memory config options

## Testing
- Measure baseline VRAM usage
- Test with multiple concurrent sessions
- Verify model unloading works correctly
- Test memory doesn't leak over time

## Implementation Complete

### Memory Tracking Infrastructure
- Created memory.rs module with MemoryStats and MemoryTracker
- Reference counting for shared model usage
- Idle timeout support for automatic model unloading

### Configuration Updates
- Added idle_timeout_secs to WhisperConfig
- Added max_sessions to WhisperConfig (default: 4)

### Test Coverage
- All 29 tests passing including 3 new memory tests
