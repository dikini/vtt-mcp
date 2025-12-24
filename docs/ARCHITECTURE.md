# VTT-MCP Architecture

## Overview

The VTT-MCP (Voice-to-Text Model Context Protocol) server is a high-performance, offline speech-to-text system built in Rust. It provides real-time transcription capabilities through the MCP protocol, enabling integration with AI assistants like Goose.

## System Architecture

The system is organized into layers:

1. **Client Layer**: Goose or other MCP clients
2. **MCP Server Layer**: Protocol handling, tool routing, resource management
3. **Core Transcription**: Audio capture, VAD, Whisper inference

## Component Overview

### MCP Server Layer (crates/vtt-mcp)

**VttMcpServer**
- Main MCP server implementation
- Handles MCP protocol via rmcp
- Manages sessions, tools, and resources
- Maintains transcript history with pagination

**Tools**
- transcribe_clip: Transcribe audio file
- start_listening: Begin real-time transcription
- stop_listening: Stop active transcription
- get_last_transcription: Retrieve recent result
- list_languages: Get supported languages
- list_audio_devices: Enumerate audio input devices
- configure_audio: Update audio/VAD settings

**Resources**
- transcript://history: Paginated transcript history
- transcript://live/{session_id}: Real-time transcript stream

### Core Transcription (crates/vtt-core)

**Audio Capture (audio/)**
- PipeWireCapture: Native PipeWire integration (Linux)
- CpalCapture: Cross-platform audio (cpal)
- AudioFormat: Sample rate, channels, format configuration

**Voice Activity Detection (vad/)**
- VadDetector: Energy-based speech detection
- VadConfig: Threshold, debounce/hangover settings

**Whisper Integration (whisper/)**
- WhisperContext: Model loading and inference
- WhisperConfig: Model path, threads, GPU, language

**Sliding Window (window/)**
- SlidingWindow: Circular buffer for audio samples

**Incremental Transcription (incremental/)**
- IncrementalTranscriber: Duplicate-suppressed results

## Data Flow

### Real-Time Transcription

1. Client calls start_listening → Creates session with UUID
2. Audio capture begins at 16kHz mono
3. VAD processes frames, detects speech vs silence
4. On speech end, sliding window provides last N seconds
5. Whisper transcribes audio
6. Incremental transcriber checks for duplicates
7. New text emitted via broadcast channel
8. Client receives updates via resource notification

### File Transcription

1. Client calls transcribe_clip with file path
2. Audio is decoded and resampled to 16kHz
3. Whisper processes entire audio
4. Result stored in session history
5. Available via get_last_transcription

## Concurrency Model

- Arc<Mutex<T>>: Shared state protected by mutex
- tokio::spawn: Async task spawning
- broadcast channel: Multi-producer, multi-consumer updates
- spawn_blocking: Offload blocking I/O operations

## Performance

| Metric | Value |
|--------|-------|
| Cold start | 1-3s |
| Warm transcription | 500ms-2s per 5s audio |
| VAD detection | <10ms |
| Memory (base model) | ~140MB |

## Configuration

Default config paths:
- Linux: ~/.config/vtt-mcp/config.toml
- System: /etc/vtt-mcp/config.toml

Key settings include audio sample rate, VAD thresholds, Whisper model selection, and memory management.
