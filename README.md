# Voice-to-Text MCP Server

A high-accuracy, low-latency voice-to-text (STT) system that integrates with Goose via the Model Context Protocol (MCP).

## Status

**Phase 1: Foundation** - COMPLETE (2025-12-23)

All Phase 1 milestones achieved:
- Audio capture working (PipeWire + cpal)
- Whisper transcription functional
- CLI tool for testing
- Comprehensive documentation
- Performance benchmarks

Performance (CPU-only, base model):
- Cold start: 1-3s
- Warm start: 500ms-2s per 5s clip
- Memory: ~200-500MB
- Accuracy: >95% on clear speech

Next Phase: MCP Server Integration

See PLAN.md for complete roadmap and docs/PHASE1_REPORT.md for detailed completion report.

---

## Overview

This project enables voice communication with the Goose AI agent through:
- Local speech-to-text using OpenAI's Whisper model (via whisper.cpp)
- MCP protocol for integration with Goose
- Offline processing - no cloud dependencies
- GPU acceleration support (CUDA/ROCm optional)
- Linux-first with PipeWire audio (cross-platform via cpal)
- **Multi-language support** with 12+ languages and auto-detection

---

## Quick Start

### Prerequisites

- Rust 1.70+
- PipeWire (Linux) or CoreAudio/WASAPI (macOS/Windows)
- Whisper model (downloaded separately)

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd vtt-mcp

# Download Whisper base model (142MB)
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin -O models/ggml-base.bin

# Build the CLI tool
cargo build --release --package vtt-cli
```

### Usage

Record and transcribe 5 seconds of audio:

```bash
cargo run --release --package vtt-cli -- --duration 5
```

For detailed setup instructions, see docs/SETUP.md.

---

## Multi-Language Support

The VTT-MCP server supports transcription in multiple languages using OpenAI's Whisper model.

### Supported Languages

| Code | Language |
|------|----------|
| auto | Auto-detect (default) |
| en | English |
| es | Spanish |
| fr | French |
| de | German |
| it | Italian |
| pt | Portuguese |
| zh | Chinese |
| ja | Japanese |
| ko | Korean |
| ru | Russian |
| ar | Arabic |
| hi | Hindi |

### Using the list_languages Tool

```javascript
// Call the list_languages tool
const result = await mcp.callTool("list_languages", {});
```

### Specifying Language in transcribe_clip

```javascript
// Transcribe a Spanish audio file
const result = await mcp.callTool("transcribe_clip", {
  audio_file: "/path/to/audio.wav",
  language: "es"  // Spanish
});
```

### Specifying Language in start_listening

```javascript
// Start listening with French language detection
const result = await mcp.callTool("start_listening", {
  language: "fr"  // French
});
```

For more details, see [docs/LANGUAGES.md](docs/LANGUAGES.md).

---

## Project Structure

```
vtt-mcp/
├── crates/
│   ├── vtt-core/       # Core transcription library
│   │   ├── audio/      # Audio capture (PipeWire/cpal)
│   │   ├── vad/        # Voice activity detection
│   │   └── whisper/    # Whisper inference
│   ├── vtt-cli/        # CLI tool for testing
│   └── vtt-mcp/        # MCP server (Phase 2)
├── docs/               # Documentation
│   ├── LANGUAGES.md    # Multi-language support
│   ├── SETUP.md        # Setup guide
│   └── PHASE1_REPORT.md # Phase 1 completion
├── models/             # Whisper model files
└── scripts/            # Utilities
```

---

## Features

### Implemented (Phase 1)

- Audio Capture: PipeWire native integration with cpal fallback
- Whisper Transcription: End-to-end speech-to-text with resampling
- CLI Tool: Record and transcribe from command line
- Multi-client Audio: Multiple processes can access audio simultaneously
- Multi-Language Support: 12+ languages with auto-detection
- Comprehensive Documentation: Setup guides and benchmarks

### Planned (Phase 2+)

- MCP Server: Integrate with Goose via Model Context Protocol
- Real-time Streaming: Continuous transcription with low latency
- GPU Acceleration: CUDA/ROCm support for faster inference
- VAD Integration: Voice Activity Detection for efficiency

---

## Performance

Benchmarks from Phase 1 (CPU-only, Whisper base model):

| Metric | Value |
|--------|-------|
| Cold Start | 1-3s (model loading) |
| Warm Start | 500ms-2s per 5s audio |
| Memory Usage | ~200-500MB |
| Accuracy | >95% (WER <5%) on clear speech |

See docs/PHASE1_REPORT.md for detailed benchmarks.

---

## Documentation

- PLAN.md - Complete implementation roadmap
- docs/SETUP.md - Detailed setup guide
- docs/LANGUAGES.md - Multi-language support guide
- docs/PHASE1_REPORT.md - Phase 1 completion report
- crates/vtt-cli/README.md - CLI tool documentation

---

## Roadmap

### Phase 1: Foundation (Complete)
- Research and architecture
- Project structure setup
- Audio capture implementation
- Whisper integration
- CLI tool for testing
- Documentation and benchmarks

### Phase 2: MCP Integration (Next)
- MCP server scaffold
- Core tools (transcribe_clip, start_listening, stop_listening)
- State management
- Error handling

### Phase 3: Real-Time Streaming
- Sliding window buffer
- Incremental transcription
- MCP streaming resources

### Phase 4: Robustness & Features
- Configuration system
- Advanced features
- Comprehensive testing
- Production polish

### Phase 5: Distribution
- Installation automation
- Packaging (.deb, AUR)
- CI/CD pipeline

---

## Contributing

This project is part of the Goose ecosystem.

For development guidelines, see:
- AGENTS.md - Guidelines for AI agents working on this project
- PLAN.md - Development workflow and quality gates

---

## License

GPL-3.0

---

## Acknowledgments

Built with:
- whisper.cpp - Whisper inference engine
- whisper-rs - Rust bindings
- PipeWire - Linux audio system
- MCP SDK - Model Context Protocol

---

## Implementation Status

### Phase 2: MCP Server (IN PROGRESS - 90% Complete)

| Task | Status | Description |
|------|--------|-------------|
| vtt-mcp-csh.3.1 | ✅ Complete | MCP Server scaffold with rmcp integration |
| vtt-mcp-csh.3.2 | ✅ Complete | transcribe_clip tool |
| vtt-mcp-csh.3.3 | ✅ Complete | start_listening/stop_listening tools |
| vtt-mcp-csh.3.4 | ✅ Complete | get_last_transcription tool |
| vtt-mcp-csh.3.5 | ✅ Complete | configure_audio tool |
| vtt-mcp-csh.3.6 | ✅ Complete | Error handling and logging |
| vtt-mcp-csh.3.7 | ✅ Complete | MCP integration tests |
| vtt-mcp-csh.6 | ✅ Complete | Multi-language support |

**Summary:** All 6 MCP tools implemented with full rmcp protocol integration. Multi-language support added with language parameter validation and list_languages tool.

See [docs/mcp-integration.md](docs/mcp-integration.md) for MCP tool documentation.
