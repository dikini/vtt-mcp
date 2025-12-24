# Task Spec: vtt-mcp-csh.4.4 - Optimize Buffer and Inference Timing

## Status
🔄 In Progress

## Description
Profile latency bottlenecks in the transcription pipeline. Tune buffer sizes (1-5s range) and inference intervals (250-1000ms) to optimize for <1s latency target while maintaining accuracy.

## Implementation Progress

### 1. Profiling Infrastructure ✅
- Created profile module with Timer, ProfileData, Timing, and TimingStats
- Instrumented SlidingWindow operations with timing
- Added timing measurements to buffer push and get operations

### 2. Remaining Work
- Add instrumentation to IncrementalTranscriber
- Create benchmark suite for timing measurements
- Test various configurations
- Document optimal settings

## Files Modified
- crates/vtt-core/src/profile/mod.rs (NEW)
- crates/vtt-core/src/window/buffer.rs
- crates/vtt-core/src/incremental/transcriber.rs
- crates/vtt-core/src/lib.rs

## Testing
- All 27 tests passing
