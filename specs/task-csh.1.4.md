# Task Specification: vtt-mcp-csh.1.4
## Integrate Silero VAD

**Task ID**: vtt-mcp-csh.1.4  
**Parent Task**: vtt-mcp-csh.1 (Project Setup and Audio Capture Pipeline)  
**Status**: ✅ Complete  
**Started**: 2025-12-23  
**Estimated**: ~2h  

---

## 1. Overview

Add silero-vad-rs dependency. Implement VAD (Voice Activity Detection) module in `vtt-core/src/vad/`. Test speech/silence detection on sample audio. Configure sensitivity thresholds.

### Context
- **Prerequisite**: Task vtt-mcp-csh.1.3 (audio capture must be working)
- **Purpose**: Enable voice activity detection to identify when speech is present in audio streams

### Dependencies
- **vtt-mcp-csh.1** - Project Setup and Audio Capture Pipeline (audio capture must be working)

---

## 2. Implementation Plan

### 2.1 Add Dependency
- Add `silero-vad = { version = "0.1" }` to `crates/vtt-core/Cargo.toml`

### 2.2 Create VAD Module
- Create `crates/vtt-core/src/vad/mod.rs`
- Create `crates/vtt-core/src/vad/detector.rs` - VAD detector wrapper
- Export `pub mod vad;` from `crates/vtt-core/src/lib.rs`

### 2.3 Implementation
- Wrap Silero VAD model
- Provide simple API: `process_audio(buffer: &[f32]) -> bool` (returns true if speech detected)
- Handle model loading and initialization
- Configure threshold/sensitivity parameters

### 2.4 Testing
- Create unit tests in `crates/vtt-core/src/vad/tests.rs`
- Test with sample audio (speech vs silence)
- Verify threshold tuning works correctly

### 2.5 CLI Integration
- Add `vtt-cli vad test` command for manual testing
- Test VAD on recorded audio files

---

## 3. Acceptance Criteria

- [ ] silero-vad-rs dependency added and builds
- [ ] VAD module created with simple API
- [ ] Can detect speech vs silence on sample audio
- [ ] Sensitivity thresholds configurable
- [ ] Unit tests pass

---

## 4. Notes

- Silero VAD is a lightweight neural network model for voice activity detection
- Returns probability of speech presence (0.0-1.0)
- Typical threshold: 0.5 (adjust based on environment)
- Works well with 16kHz or 8kHz audio (we have 48kHz, may need resampling)
