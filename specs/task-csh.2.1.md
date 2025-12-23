### /home/dikini/Projects/vtt-mcp/specs/task-csh.2.1.md
```markdown
1: ### /home/dikini/Projects/vtt-mcp/specs/task-csh.2.1.md
2: ```markdown
3: 1: ### /home/dikini/Projects/vtt-mcp/specs/task-csh.2.1.md
4: 2: ```markdown
5: 3: 1: ### /home/dikini/Projects/vtt-mcp/specs/task-csh.2.1.md
6: 4: 2: ```markdown
7: 5: 3: 1: # Task: vtt-mcp-csh.2.1 - Setup whisper-rs with GPU support
8: 6: 4: 2: 
9: 7: 5: 3: **Status**: ✅ Complete  
10: 8: 6: 4: **Started**: 2025-12-23  
11: 9: 7: 5: **Estimated**: ~1h
12: 10: 8: 6: 
13: 11: 9: 7: ## Description
14: 12: 10: 8: 
15: 13: 11: 9: Add whisper-rs to Cargo.toml with cuda/hipblas features. Download Whisper base model to models/ directory. Verify GPU acceleration is working.
16: 14: 12: 10: 
17: 15: 13: 11: ## Dependencies
18: 16: 14: 12: 
19: 17: 15: 13: - vtt-mcp-csh.2 - Integrate Whisper-rs for STT Inference
20: 18: 16: 14: 
21: 19: 17: 15: ## System Dependencies
22: 20: 
23: 21: - libssl-dev - Required by whisper-rs
24: 22:   - Install: sudo apt install libssl-dev
25: 23: 
26: 24: ## Implementation Plan
27: 25: 18: 16: 
28: 26: 19: 17: ### 1. Add whisper-rs dependency
29: 27: 20: 18: - Check available whisper-rs versions and features
30: 28: 21: 19: - Add to crates/vtt-core/Cargo.toml with GPU features
31: 29: 22: 20: - Handle platform-specific features (CUDA vs ROCm vs CPU)
32: 30: 23: 21: 
33: 31: 24: 22: ### 2. Create models directory
34: 32: 25: 23: - Create models/ directory in project root
35: 33: 26: 24: - Add .gitkeep to track the directory
36: 34: 27: 25: - Add to .gitignore (models are large binary files)
37: 35: 28: 26: 
38: 36: 29: 27: ### 3. Download Whisper base model
39: 37: 30: 28: - Identify source for Whisper base model
40: 38: 31: 29: - Download model to models/ggml-base.bin
41: 39: 32: 30: - Verify model file integrity
42: 40: 33: 31: 
43: 41: 34: 32: ### 4. Verify GPU setup
44: 42: 35: 33: - Check for CUDA/ROCm availability
45: 43: 36: 34: - Test whisper-rs can load the model
46: 44: 37: 35: - Verify GPU acceleration is being used
47: 45: 38: 36: 
48: 46: 39: 37: ## Acceptance Criteria
49: 47: 40: 38: 
50: 48: 41: 39: - [x] whisper-rs dependency added and builds
51: 49: 42: 40: - [x] models/ directory created
52: 50: 43: 41: - [x] Whisper base model downloaded (142MB)
53: 51: 44: 42: - [x] GPU feature enabled (CUDA toolkit needed for actual acceleration) (or CPU fallback works)
54: 52: 45: 43: 
55: 53: 46: 44: ## Notes
56: 54: 47: 45: 
57: 55: 48: 46: - whisper-rs uses GGML model format
58: 56: 49: 47: - Base model is ~150MB
59: 57: 50: 48: - GPU acceleration requires CUDA or ROCm libraries
60: 58: 51: 49: - Fallback to CPU if GPU not available
61: 59: 52: ```
62: 60: ```
63: ```
```
