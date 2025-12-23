### /home/dikini/Projects/vtt-mcp/specs/task-csh.1.4.md
```markdown
1: # Task: vtt-mcp-csh.1.4 - Integrate Silero VAD
2: 
3: **Status**: ✅ Complete  
4: **Started**: 2025-12-23  
5: **Estimated**: ~2h
6: 
7: ## Description
8: 
9: Add silero-vad-rs dependency. Implement VAD (Voice Activity Detection) module in `vtt-core/src/vad/`. Test speech/silence detection on sample audio. Configure sensitivity thresholds.
10: 
11: ## Dependencies
12: 
13: - `vtt-mcp-csh.1` - Project Setup and Audio Capture Pipeline (audio capture must be working)
14: 
15: ## Implementation Plan
16: 
17: ### 1. Add Dependency
18: - Add `silero-vad = { version = "0.1" }` to `crates/vtt-core/Cargo.toml`
19: 
20: ### 2. Create VAD Module
21: - Create `crates/vtt-core/src/vad/mod.rs`
22: - Create `crates/vtt-core/src/vad/detector.rs` - VAD detector wrapper
23: - Export `pub mod vad;` from `crates/vtt-core/src/lib.rs`
24: 
25: ### 3. Implementation
26: - Wrap Silero VAD model
27: - Provide simple API: `process_audio(buffer: &[f32]) -> bool` (returns true if speech detected)
28: - Handle model loading and initialization
29: - Configure threshold/sensitivity parameters
30: 
31: ### 4. Testing
32: - Create unit tests in `crates/vtt-core/src/vad/tests.rs`
33: - Test with sample audio (speech vs silence)
34: - Verify threshold tuning works correctly
35: 
36: ### 5. CLI Integration
37: - Add `vtt-cli vad test` command for manual testing
38: - Test VAD on recorded audio files
39: 
40: ## Acceptance Criteria
41: 
42: - [ ] silero-vad-rs dependency added and builds
43: - [ ] VAD module created with simple API
44: - [ ] Can detect speech vs silence on sample audio
45: - [ ] Sensitivity thresholds configurable
46: - [ ] Unit tests pass
47: 
48: ## Notes
49: 
50: - Silero VAD is a lightweight neural network model for voice activity detection
51: - Returns probability of speech presence (0.0-1.0)
52: - Typical threshold: 0.5 (adjust based on environment)
53: - Works well with 16kHz or 8kHz audio (we have 48kHz, may need resampling)
```
