# Voice-to-Text MCP Server: High-Level Plan

**Project:** vtt-mcp  
**Goal:** Build a voice-to-text MCP server to enable voice communication with Goose AI agent  
**Date:** 2025-12-22  
**Status:** Planning Phase

---

## 1. Project Overview

### 1.1 Vision
Create a high-accuracy, low-latency voice-to-text (STT) system that integrates with Goose via the Model Context Protocol (MCP), enabling users to interact with the Goose AI agent through voice commands on Linux.

### 1.2 Key Requirements
- **Accuracy:** High-quality transcription using state-of-the-art models (Whisper)
- **Performance:** GPU-accelerated (CUDA/ROCm) for real-time processing
- **Integration:** Native MCP server implementation in Rust
- **Platform:** Linux-first with PipeWire audio support
- **Latency:** Low enough for natural conversation (<1s delay)
- **Offline:** Fully local processing, no cloud dependencies

---

## 2. System Specifications

### 2.1 Technical Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| **Language** | Rust | Performance, safety, MCP SDK availability |
| **STT Engine** | whisper-rs | Bindings to whisper.cpp, proven accuracy |
| **ML Framework** | whisper.cpp | GPU-accelerated, optimized inference |
| **Audio Capture** | cpal | Cross-platform, PipeWire compatible |
| **VAD** | silero-vad-rs | Fast voice activity detection, reduces GPU load |
| **MCP Framework** | rmcp (rust-sdk) | Official Rust MCP SDK |
| **Async Runtime** | tokio | Standard for Rust async operations |

### 2.2 Model Selection
- **Primary:** Whisper large-v3 (highest accuracy, requires 10GB+ VRAM)
- **Alternative:** Whisper small/base (faster, 1-2GB VRAM)
- **Configuration:** Runtime model selection based on hardware

### 2.3 Audio Specifications
- **Sample Rate:** 16kHz (Whisper requirement)
- **Format:** 32-bit float PCM (f32)
- **Channels:** Mono
- **Buffer Size:** 3-5 seconds sliding window
- **VAD Threshold:** Configurable sensitivity

### 2.4 MCP Interface Design

#### Tools Exposed:
1. **start_listening**
   - Description: Begin continuous voice capture and transcription
   - Parameters: 
     - model: "base" | "small" | "large" (optional, default: "base")
     - language: ISO language code (optional, default: auto-detect)
   - Returns: session_id
   
2. **stop_listening**
   - Description: Stop active transcription session
   - Parameters: session_id
   - Returns: final transcript summary

3. **transcribe_clip**
   - Description: One-shot transcription (push-to-talk mode)
   - Parameters:
     - duration_seconds: float (default: 5.0)
     - model: "base" | "small" | "large" (optional)
   - Returns: transcribed text

4. **get_last_transcription**
   - Description: Retrieve most recent transcribed text
   - Parameters: none
   - Returns: text, timestamp, confidence

5. **configure_audio**
   - Description: Adjust audio input settings
   - Parameters:
     - device: audio device name (optional)
     - vad_sensitivity: 0.0-1.0 (optional)
   - Returns: current configuration

#### Resources Exposed:
1. **transcript://live**
   - Real-time streaming transcript updates
   - Subscribe mechanism for continuous updates

2. **transcript://history**
   - Historical transcription log
   - Paginated access to past transcriptions

---

## 3. Architecture

### 3.1 System Components

```
┌─────────────────────────────────────────────────────────────┐
│                        MCP Client (Goose)                   │
└────────────────────────┬────────────────────────────────────┘
                         │ MCP Protocol (stdio)
┌────────────────────────▼────────────────────────────────────┐
│                    MCP Server (vtt-mcp)                     │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              MCP Request Handler                     │   │
│  │  (start_listening, transcribe_clip, etc.)           │   │
│  └──────────────────┬───────────────────────────────────┘   │
│                     │                                        │
│  ┌──────────────────▼───────────────────────────────────┐   │
│  │           Transcription Engine                       │   │
│  │  ┌────────────┐  ┌──────────┐  ┌─────────────────┐  │   │
│  │  │ Audio      │→ │ VAD      │→ │ Whisper         │  │   │
│  │  │ Capture    │  │ Filter   │  │ Inference (GPU) │  │   │
│  │  │ (cpal)     │  │ (Silero) │  │ (whisper-rs)    │  │   │
│  │  └────────────┘  └──────────┘  └─────────────────┘  │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │         State Management                             │   │
│  │  - Session tracking                                  │   │
│  │  - Transcript history                                │   │
│  │  - Configuration                                     │   │
│  └──────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
                         │
         ┌───────────────┴───────────────┐
         │                               │
┌────────▼─────────┐          ┌──────────▼──────────┐
│  PipeWire/ALSA   │          │   GPU (CUDA/ROCm)   │
│  (Audio Input)   │          │   (Acceleration)    │
└──────────────────┘          └─────────────────────┘
```

### 3.2 Data Flow

#### Live Transcription Flow:
1. **Audio Thread:** Continuously captures audio from microphone via cpal
2. **VAD Stage:** Silero VAD filters out silence (saves 60-80% GPU cycles)
3. **Buffer Stage:** Speech segments accumulate in sliding window buffer
4. **Inference Stage:** Every 500ms-1s, buffer sent to Whisper on GPU
5. **Result Stage:** Text segments streamed back via MCP resources
6. **Client Stage:** Goose receives and processes transcribed commands

#### Push-to-Talk Flow:
1. MCP tool invoked with duration parameter
2. Record fixed-duration audio clip
3. Process entire clip through Whisper
4. Return complete transcription
5. No streaming, simpler implementation

### 3.3 Threading Model

```
┌─────────────────┐
│  Main Thread    │  MCP Server event loop
│  (stdio I/O)    │  
└────────┬────────┘
         │
┌────────▼────────┐
│  Handler Thread │  Processes MCP tool requests
│  (tokio)        │  Manages session state
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
┌───▼──┐  ┌──▼────┐
│Audio │  │Inference│
│Thread│  │Thread   │
│(cpal)│  │(GPU)    │
└──────┘  └─────────┘
```

---

## 4. Implementation Phases

### Phase 1: Foundation (Week 1)
**Goal:** Basic infrastructure and proof of concept

#### Milestones:
- [x] Research completed (STT options, architecture)
- [ ] Project structure setup
  - Cargo workspace with multiple crates
  - Basic dependency configuration
- [ ] Audio capture working
  - cpal integration
  - Raw audio recording to file
  - PipeWire compatibility verified
- [ ] VAD integration
  - Silero VAD working
  - Speech/silence detection validated
- [ ] Basic Whisper inference
  - Load model
  - Transcribe pre-recorded audio file
  - Verify GPU acceleration

**Deliverables:**
- CLI tool that records audio and transcribes to stdout
- Documentation of setup steps
- Performance benchmarks (latency, accuracy)

---

### Phase 2: MCP Integration (Week 2)
**Goal:** Transform POC into functional MCP server

#### Milestones:
- [ ] MCP server scaffold
  - rmcp SDK integration
  - stdio transport setup
  - Basic tool registration
- [ ] Implement core tools
  - `transcribe_clip` (simplest, for testing)
  - `start_listening` / `stop_listening`
  - `get_last_transcription`
- [ ] State management
  - Session tracking
  - Transcript history storage
- [ ] Error handling
  - Graceful failure modes
  - Logging infrastructure

**Deliverables:**
- Functional MCP server binary
- Test script for MCP tool invocation
- Integration guide for Goose

---

### Phase 3: Real-Time Streaming (Week 3)
**Goal:** Low-latency continuous transcription

#### Milestones:
- [ ] Sliding window buffer implementation
- [ ] Incremental transcription
  - Partial result streaming
  - Duplicate suppression
- [ ] MCP resource implementation
  - `transcript://live` streaming resource
  - Subscription mechanism
- [ ] Latency optimization
  - Buffer size tuning
  - Inference interval optimization
  - GPU memory management

**Deliverables:**
- Real-time transcription with <1s latency
- Streaming resource for Goose
- Performance profiling report

---

### Phase 4: Robustness & Features (Week 4)
**Goal:** Production-ready quality

#### Milestones:
- [ ] Configuration system
  - TOML/YAML config file
  - Runtime model switching
  - Audio device selection
- [ ] Advanced features
  - Multi-language support
  - Speaker diarization (if needed)
  - Punctuation restoration
- [ ] Testing
  - Unit tests for core components
  - Integration tests with mock MCP client
  - End-to-end tests with Goose
- [ ] Documentation
  - API documentation
  - User guide
  - Troubleshooting guide

**Deliverables:**
- Production-ready MCP server
- Comprehensive test suite (>80% coverage)
- Complete documentation

---

### Phase 5: Polish & Distribution (Week 5)
**Goal:** Easy installation and deployment

#### Milestones:
- [ ] Installation automation
  - GPU detection and configuration
  - Model download automation
  - Dependency checking
- [ ] Packaging
  - .deb package for Debian/Ubuntu
  - AUR package for Arch
  - Flatpak (optional)
- [ ] CI/CD pipeline
  - Automated builds
  - Release artifacts
- [ ] Performance optimization
  - Profile-guided optimization
  - Memory usage reduction

**Deliverables:**
- Installation packages
- Release on GitHub/crates.io
- Performance comparison with alternatives

---

## 5. Technical Implementation Details

### 5.1 Project Structure

```
vtt-mcp/
├── Cargo.toml              # Workspace definition
├── crates/
│   ├── vtt-core/           # Core transcription engine
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── audio/      # Audio capture (cpal)
│   │       ├── vad/        # Voice activity detection
│   │       ├── whisper/    # Whisper inference wrapper
│   │       └── buffer/     # Sliding window buffer
│   ├── vtt-mcp/            # MCP server implementation
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── server.rs   # MCP server logic
│   │       ├── tools/      # MCP tool implementations
│   │       ├── resources/  # MCP resource implementations
│   │       └── state.rs    # Session state management
│   └── vtt-cli/            # Standalone CLI (for testing)
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
├── models/                 # Whisper model files (gitignored)
├── config/                 # Configuration templates
│   └── vtt-mcp.toml
├── tests/                  # Integration tests
├── docs/                   # Documentation
├── research/               # Research artifacts (existing)
└── scripts/                # Build and setup scripts
```

### 5.2 Key Dependencies

```toml
[dependencies]
# Core
whisper-rs = { version = "0.15", features = ["cuda"] }  # or "hipblas" for AMD
cpal = "0.15"
silero-vad = "0.1"

# MCP
mcp-core = "0.1"  # Official MCP Rust SDK
tokio = { version = "1", features = ["full"] }

# Utilities
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = "0.3"

# Configuration
config = "0.14"
dirs = "5"

# Audio processing
rubato = "0.14"  # Resampling if needed
```

### 5.3 Configuration Schema

```toml
[audio]
sample_rate = 16000
buffer_size = 1024
device = "default"  # or specific device name

[vad]
enabled = true
sensitivity = 0.5  # 0.0 = least sensitive, 1.0 = most sensitive
min_speech_duration_ms = 250
min_silence_duration_ms = 500

[whisper]
model = "base"  # base | small | large
model_path = "~/.local/share/vtt-mcp/models"
language = "auto"  # or specific ISO code
gpu_device = 0  # GPU index

[transcription]
sliding_window_size_sec = 3.0
inference_interval_ms = 500
max_history_items = 100

[mcp]
log_level = "info"
```

### 5.4 Error Handling Strategy

```rust
// Error types hierarchy
pub enum VttError {
    Audio(AudioError),      // cpal errors, device not found
    Vad(VadError),          // VAD initialization/processing
    Whisper(WhisperError),  // Model loading, inference
    Mcp(McpError),          // Protocol errors
    Config(ConfigError),    // Configuration parsing
    Io(std::io::Error),     // File operations
}

// All errors propagate to MCP error responses
// Graceful degradation where possible
```

---

## 6. Testing Strategy

### 6.1 Unit Tests
- Audio capture mocking
- VAD accuracy on test clips
- Buffer management correctness
- Configuration parsing

### 6.2 Integration Tests
- End-to-end transcription pipeline
- MCP tool invocation
- Resource subscription
- Multi-session handling

### 6.3 Performance Tests
- Latency benchmarks (audio → text)
- GPU memory usage profiling
- CPU usage monitoring
- Throughput tests (words/minute)

### 6.4 Accuracy Tests
- Standard STT datasets (LibriSpeech, Common Voice)
- Word Error Rate (WER) calculation
- Comparison with cloud services (Whisper API baseline)

---

## 7. Risks and Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|-----------|------------|
| GPU compatibility issues | High | Medium | Provide CPU fallback, extensive docs |
| High latency (>2s) | High | Medium | Optimize buffer size, use smaller models |
| MCP protocol changes | Medium | Low | Track SDK updates, version pinning |
| Audio driver incompatibility | Medium | Medium | Support multiple backends (ALSA, Pulse, PipeWire) |
| Model download size (large-v3 = 3GB) | Low | High | Auto-download on first run, user warning |
| Whisper hallucinations | Medium | Medium | Confidence scoring, silence detection |

---

## 8. Success Metrics

### 8.1 Performance
- **Latency:** <1s from speech end to text output
- **Accuracy:** >95% WER on clear speech
- **Resource Usage:** <4GB RAM, <50% of one GPU

### 8.2 Usability
- **Setup Time:** <10 minutes from clone to running
- **Model Loading:** <30 seconds on first run
- **Error Rate:** <1% crash rate in 1-hour sessions

### 8.3 Integration
- **Goose Compatibility:** Works with Goose Desktop and CLI
- **Response Time:** Tool calls return within 100ms (excluding inference)
- **Streaming:** Real-time updates with <500ms gaps

---

## 9. Future Enhancements (Post-MVP)

### Phase 6+: Advanced Features
- [ ] Wake word detection ("Hey Goose")
- [ ] Speaker diarization (multi-speaker support)
- [ ] Custom vocabulary/domain adaptation
- [ ] Audio output (TTS for Goose responses)
- [ ] Multi-modal (screen context + voice)
- [ ] Whisper fine-tuning on user's voice
- [ ] Cloud sync for transcript history
- [ ] Mobile companion app

---

## 10. Development Workflow

### 10.1 Iteration Cycle
1. Implement feature in `vtt-core` crate
2. Test with `vtt-cli` standalone
3. Integrate into `vtt-mcp` server
4. Test with Goose Desktop
5. Document and commit

### 10.2 Daily Workflow
- Morning: Review TODO.md, update Beads tasks
- Development: TDD approach, write tests first
- Afternoon: Integration testing
- Evening: Documentation updates, `bd sync`

### 10.3 Quality Gates
Before each commit:
- [ ] Code compiles without warnings
- [ ] Tests pass (`cargo test`)
- [ ] Format checked (`cargo fmt`)
- [ ] Lints pass (`cargo clippy`)

---

## 11. Documentation Plan

### 11.1 User Documentation
- **README.md:** Quick start, installation
- **docs/SETUP.md:** Detailed setup (GPU drivers, models)
- **docs/USAGE.md:** How to use with Goose
- **docs/TROUBLESHOOTING.md:** Common issues

### 11.2 Developer Documentation
- **docs/ARCHITECTURE.md:** System design deep-dive
- **docs/CONTRIBUTING.md:** How to contribute
- **API docs:** Generated from rustdoc comments

### 11.3 Video/Demos
- Setup walkthrough (asciinema)
- Usage demo with Goose
- Performance comparison video

---

## 12. Next Steps (Immediate)

After plan approval, begin Phase 1:

1. **Setup Cargo workspace** (`vtt-core`, `vtt-mcp`, `vtt-cli`)
2. **Configure dependencies** (whisper-rs, cpal, silero-vad)
3. **Implement audio capture** (record to WAV file test)
4. **Integrate Whisper** (transcribe test audio file)
5. **Document progress** (update TODO.md, create Beads tasks)

**Estimated time to first working prototype:** 2-3 days
**Estimated time to MCP integration:** 1 week
**Estimated time to production-ready:** 3-4 weeks

---

**Plan Status:** ✅ Ready for Review  
**Next Action:** Approval → Create detailed Beads tasks → Begin Phase 1
