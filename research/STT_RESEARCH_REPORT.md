
# Research Report: High-Accuracy Live STT in Rust for Linux

**Date:** 2025-12-22  
**Target:** Local Open Source Voice Assistant / Goose Voice Interface  
**Goal:** Maximum Accuracy, GPU Accelerated (CUDA/ROCm), Live Transcription on Linux (PipeWire).

---

## 1. Executive Summary
The most viable path for high-accuracy, low-latency live transcription in Rust on Linux is using **Whisper-rs** (bindings for `whisper.cpp`) coupled with **Silero VAD** for voice activity detection. While **Candle** (HuggingFace's native Rust ML framework) is a strong alternative, `whisper.cpp` currently offers more mature streaming optimizations and robust GPU acceleration (CUDA/ROCm) support that is easily toggled via feature flags.

---

## 2. Recommended Technical Stack

| Component | Recommendation | Why? |
| :--- | :--- | :--- |
| **STT Engine** | [whisper-rs](https://github.com/tazz4843/whisper-rs) | High performance, CUDA/ROCm support, battle-tested. |
| **ML Framework** | [whisper.cpp](https://github.com/ggerganov/whisper.cpp) | State-of-the-art C++ implementation used under the hood. |
| **Audio Capture** | [cpal](https://github.com/RustAudio/cpal) | Cross-platform, works seamlessly with PipeWire via Pulse/ALSA. |
| **VAD** | [silero-vad-rs](https://github.com/shihira/silero-vad-rs) | Fast, reliable, ONNX-based. Essential for "Live" feel. |
| **MCP SDK** | [rmcp](https://github.com/modelcontextprotocol/rust-sdk) | Official SDK for building Model Context Protocol servers. |

---

## 3. Detailed Architecture

### 3.1 Live Audio Pipeline
To achieve "live" transcription, the system should follow a Producer-Consumer pattern:

1.  **Producer (Audio Thread):** Captures raw PCM audio (f32, 16kHz, mono) using `cpal`.
2.  **VAD Filter:** Chunks are passed to Silero VAD. If no speech is detected, the audio is dropped to save GPU cycles.
3.  **Circular Buffer:** If speech is detected, audio is pushed into a sliding window buffer (e.g., 3-5 seconds long).
4.  **Inference (Worker Thread):** Periodically (every 500ms - 1s), the buffer is sent to the GPU for transcription.
5.  **Partial Results:** Whisper provides "segments." We stream these back to the UI/Client immediately.

### 3.2 GPU Acceleration
*   **CUDA (NVIDIA):** Enable via the `cuda` feature flag in `whisper-rs`. Requires `nvcc` and CUDA toolkits installed on the host.
*   **ROCm (AMD):** Enable via the `hipblas` feature flag. Specifically optimized for Linux.

---

## 4. Integration with Goose (MCP)

To make this available to Goose, we build an MCP Server that exposes a "Listen" tool or a subscription mechanism.

### Suggested MCP Tools:
*   `start_listening`: Opens the audio stream and starts pushing transcriptions to a log or specific resource.
*   `transcribe_clip`: A simple tool for push-to-talk.
*   `get_last_transcription`: Retrieves the most recent spoken command.

---

## 5. Implementation Roadmap (Phase 1)

### Step 1: Prototype Audio Loop
Implement a basic CLI that captures audio and runs VAD.
```rust
// Simplified snippet
let stream = device.build_input_stream(&config, move |data: &[f32], _| {
    if vad.is_speech(data) {
        buffer.extend(data);
    }
}, ...);
```

### Step 2: Integrate Whisper-rs
Add GPU support to `Cargo.toml`:
```toml
[dependencies]
whisper-rs = { version = "0.15", features = ["cuda"] }
```

### Step 3: MCP Server Wrapper
Wrap the transcription loop in the `rmcp` server handler.

---

## 6. Risks and Mitigations
*   **Latency:** Transcription of long buffers can lag. *Mitigation:* Use "incremental" transcription where you only re-process the last few hundred milliseconds.
*   **Dependency Hell:** CUDA/ROCm versions must match the `whisper.cpp` expectations. *Mitigation:* Use a well-defined `build.rs` or provide a Dockerized dev environment.
*   **Accuracy vs. Size:** `large-v3` is most accurate but heavy. *Mitigation:* Provide options to switch between `base`, `small`, and `large` models depending on VRAM availability.

---

**Artifacts created:**
*   `research/architecture_diagram.md` (Logical flow)
*   `research/dependency_guide.md` (Setup instructions for CUDA/ROCm)
