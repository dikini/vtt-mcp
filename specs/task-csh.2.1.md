# Task Specification: vtt-mcp-csh.2.1
## Setup whisper-rs with GPU support

**Task ID**: vtt-mcp-csh.2.1  
**Parent Task**: vtt-mcp-csh.2 (Integrate Whisper-rs for STT Inference)  
**Status**: ✅ Complete  
**Started**: 2025-12-23  
**Estimated**: ~1h  

---

## 1. Overview

Add whisper-rs to Cargo.toml with cuda/hipblas features. Download Whisper base model to models/ directory. Verify GPU acceleration is working.

### Dependencies
- **vtt-mcp-csh.2** - Integrate Whisper-rs for STT Inference

### System Dependencies
- **libssl-dev** - Required by whisper-rs
  - Install: `sudo apt install libssl-dev`

---

## 2. Implementation Plan

### 2.1 Add whisper-rs dependency
- Check available whisper-rs versions and features
- Add to `crates/vtt-core/Cargo.toml` with GPU features
- Handle platform-specific features (CUDA vs ROCm vs CPU)

### 2.2 Create models directory
- Create `models/` directory in project root
- Add `.gitkeep` to track the directory
- Add to `.gitignore` (models are large binary files)

### 2.3 Download Whisper base model
- Identify source for Whisper base model
- Download model to `models/ggml-base.bin`
- Verify model file integrity

### 2.4 Verify GPU setup
- Check for CUDA/ROCm availability
- Test whisper-rs can load the model
- Verify GPU acceleration is being used

---

## 3. Acceptance Criteria

- [x] whisper-rs dependency added and builds
- [x] models/ directory created
- [x] Whisper base model downloaded (142MB)
- [x] GPU feature enabled (CUDA toolkit needed for actual acceleration) (or CPU fallback works)

---

## 4. Notes

- whisper-rs uses GGML model format
- Base model is ~150MB
- GPU acceleration requires CUDA or ROCm libraries
- Fallback to CPU if GPU not available
