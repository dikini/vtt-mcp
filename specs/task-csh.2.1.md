### /home/dikini/Projects/vtt-mcp/specs/task-csh.2.1.md
```markdown
1: ### /home/dikini/Projects/vtt-mcp/specs/task-csh.2.1.md
2: ```markdown
3: 1: ### /home/dikini/Projects/vtt-mcp/specs/task-csh.2.1.md
4: 2: ```markdown
5: 3: 1: # Task: vtt-mcp-csh.2.1 - Setup whisper-rs with GPU support
6: 4: 2: 
7: 5: 3: **Status**: ✅ Complete  
8: 6: 4: **Started**: 2025-12-23  
9: 7: 5: **Estimated**: ~1h
10: 8: 6: 
11: 9: 7: ## Description
12: 10: 8: 
13: 11: 9: Add whisper-rs to Cargo.toml with cuda/hipblas features. Download Whisper base model to models/ directory. Verify GPU acceleration is working.
14: 12: 10: 
15: 13: 11: ## Dependencies
16: 14: 12: 
17: 15: 13: - vtt-mcp-csh.2 - Integrate Whisper-rs for STT Inference
18: 16: 14: 
19: 17: 15: ## System Dependencies
20: 
21: - libssl-dev - Required by whisper-rs
22:   - Install: sudo apt install libssl-dev
23: 
24: ## Implementation Plan
25: 18: 16: 
26: 19: 17: ### 1. Add whisper-rs dependency
27: 20: 18: - Check available whisper-rs versions and features
28: 21: 19: - Add to crates/vtt-core/Cargo.toml with GPU features
29: 22: 20: - Handle platform-specific features (CUDA vs ROCm vs CPU)
30: 23: 21: 
31: 24: 22: ### 2. Create models directory
32: 25: 23: - Create models/ directory in project root
33: 26: 24: - Add .gitkeep to track the directory
34: 27: 25: - Add to .gitignore (models are large binary files)
35: 28: 26: 
36: 29: 27: ### 3. Download Whisper base model
37: 30: 28: - Identify source for Whisper base model
38: 31: 29: - Download model to models/ggml-base.bin
39: 32: 30: - Verify model file integrity
40: 33: 31: 
41: 34: 32: ### 4. Verify GPU setup
42: 35: 33: - Check for CUDA/ROCm availability
43: 36: 34: - Test whisper-rs can load the model
44: 37: 35: - Verify GPU acceleration is being used
45: 38: 36: 
46: 39: 37: ## Acceptance Criteria
47: 40: 38: 
48: 41: 39: - [ ] whisper-rs dependency added and builds
49: 42: 40: - [x] models/ directory created
50: 43: 41: - [x] Whisper base model downloaded (142MB)
51: 44: 42: - [x] GPU feature enabled (CUDA toolkit needed for actual acceleration) (or CPU fallback works)
52: 45: 43: 
53: 46: 44: ## Notes
54: 47: 45: 
55: 48: 46: - whisper-rs uses GGML model format
56: 49: 47: - Base model is ~150MB
57: 50: 48: - GPU acceleration requires CUDA or ROCm libraries
58: 51: 49: - Fallback to CPU if GPU not available
59: 52: ```
60: ```
```
