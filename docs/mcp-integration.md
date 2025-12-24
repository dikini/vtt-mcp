# VTT MCP Server Integration Guide

## Overview

The VTT (Voice-to-Text) MCP server provides speech-to-text functionality via the Model Context Protocol (MCP). This guide covers integration with AI assistants like Goose.

## Architecture

```
┌─────────────────┐     MCP Protocol      ┌──────────────────┐
│   Goose/AI      │ ◄────────────────────► │   VTT MCP Server │
│   Assistant     │      (stdio/JSON-RPC)  │   (vtt-mcp)      │
└─────────────────┘                        └────────┬─────────┘
                                                   │
                                                   ▼
                                          ┌─────────────────┐
                                          │   vtt-core      │
                                          │   (Whisper)     │
                                          └─────────────────┘
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `WHISPER_MODEL` | Path to Whisper model file | `models/ggml-base.bin` |
| `WHISPER_THREADS` | Number of CPU threads | `<physical cores>` |
| `WHISPER_USE_GPU` | Enable GPU acceleration | `true` |
| `RUST_LOG` | Log level filter | `info,vtt_mcp=debug` |

### Server Startup

```bash
# Start server with default settings
cargo run --package vtt-mcp

# Start with custom model
WHISPER_MODEL=models/ggml-tiny.bin cargo run --package vtt-mcp

# Start with logging
RUST_LOG=debug cargo run --package vtt-mcp
```

## MCP Tools

### 1. transcribe_clip

One-shot transcription from a WAV audio file.

**Parameters:**
```json
{
  "audio_file": "/path/to/audio.wav",
  "model_path": "models/ggml-base.bin",  // optional
  "use_gpu": true,                       // optional
  "threads": 4                           // optional
}
```

**Response:**
```json
{
  "text": "Transcribed text here...",
  "confidence": 0.95,
  "start_ms": 0,
  "end_ms": 5230
}
```

### 2. start_listening

Start an audio capture session for continuous recording.

**Parameters:**
```json
{
  "model_path": "models/ggml-base.bin",  // optional
  "language": "en",                      // optional
  "use_gpu": true,                       // optional
  "threads": 4,                          // optional
  "device_name": "Built-in Audio"        // optional (future)
}
```

**Response:**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "listening",
  "start_time": "2025-12-24T01:00:00Z",
  "model_path": "models/ggml-base.bin",
  "language": "en",
  "use_gpu": true
}
```

### 3. stop_listening

Stop an active listening session and optionally transcribe.

**Parameters:**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "transcribe": true
}
```

**Response:**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "transcribed",
  "duration_ms": 5000,
  "samples_captured": 80000,
  "transcription": {
    "text": "Transcribed audio...",
    "confidence": 0.92,
    "start_ms": 0,
    "end_ms": 5000
  },
  "error": null
}
```

### 4. get_last_transcription

Retrieve the most recent transcription or a session-specific one.

**Parameters (most recent):**
```json
{}
```

**Parameters (specific session):**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Response:**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-12-24T01:00:05Z",
  "text": "Transcribed text...",
  "confidence": 0.92,
  "start_ms": 0,
  "end_ms": 5000,
  "model_path": "models/ggml-base.bin",
  "language": "en"
}
```

### 5. list_audio_devices

Enumerate available audio input devices.

**Parameters:**
```json
{}
```

**Response:**
```json
{
  "devices": [
    {
      "name": "Built-in Audio",
      "is_default": true
    },
    {
      "name": "USB Microphone",
      "is_default": false
    }
  ],
  "default_device": "Built-in Audio"
}
```

### 6. configure_audio

Configure audio device and VAD settings.

**Parameters:**
```json
{
  "device_name": "USB Microphone",      // optional
  "vad_sensitivity": 0.01              // optional, 0.0-1.0
}
```

**Response:**
```json
{
  "default_device": "USB Microphone",
  "vad_config": {
    "energy_threshold": 0.01,
    "speech_frames_threshold": 3,
    "silence_frames_threshold": 10,
    "min_speech_duration": 30
  },
  "available_devices": [
    {
      "name": "Built-in Audio",
      "is_default": false
    },
    {
      "name": "USB Microphone",
      "is_default": true
    }
  ]
}
```

## Error Handling

### Error Codes

| Code | Description |
|------|-------------|
| `INVALID_PARAMS` | Invalid or missing parameters |
| `DEVICE_NOT_FOUND` | Audio device not found |
| `NO_AUDIO_DATA` | No audio data available |
| `MODEL_ERROR` | Whisper model error |
| `SESSION_ERROR` | Session management error |
| `AUDIO_ERROR` | Audio capture error |
| `TRANSCRIPTION_ERROR` | Transcription failed |
| `IO_ERROR` | File I/O error |
| `INTERNAL_ERROR` | Internal server error |

### Error Response Format

```json
{
  "error": {
    "code": -32602,
    "message": "Invalid params: Audio file not found: /path/to/file.wav",
    "data": null
  },
  "id": 1
}
```

## Integration with Goose

### MCP Client Configuration

Goose can connect to the VTT MCP server via stdio:

```toml
# goose config
[mcp.vtt-server]
command = "cargo run --package vtt-mcp"
args = []
env = { WHISPER_MODEL = "models/ggml-base.bin" }
```

### Example Usage

```python
# Pseudo-code for Goose integration
from goose import MCPClient

client = MCPClient("vtt-server")

# Start listening
session = client.call_tool("start_listening", {
    "language": "en"
})
session_id = session["session_id"]

# ... wait for speech ...

# Stop and transcribe
result = client.call_tool("stop_listening", {
    "session_id": session_id,
    "transcribe": True
})

text = result["transcription"]["text"]
print(f"Transcribed: {text}")
```

## Testing

### Unit Tests

```bash
# Run all tests
cargo test --package vtt-mcp

# Run integration tests
cargo test --package vtt-mcp --test integration_tests

# Run with output
cargo test --package vtt-mcp -- --nocapture
```

### Manual Testing

```bash
# List tools
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | cargo run --package vtt-mcp

# Call a tool
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"list_audio_devices","arguments":{}}}' | cargo run --package vtt-mcp
```

## Performance Considerations

- **Latency**: Target <1s from audio end to transcription
- **Memory**: Whisper model ~1GB for base model
- **CPU**: 4+ threads recommended for real-time
- **GPU**: Recommended for faster transcription

## Troubleshooting

### Common Issues

1. **Model not found**
   - Ensure `WHISPER_MODEL` points to valid model file
   - Download models from [Whisper.cpp](https://github.com/ggerganov/whisper.cpp)

2. **No audio devices**
   - Check system audio configuration
   - Verify microphone permissions

3. **Transcription errors**
   - Check audio quality and sample rate
   - Ensure 16kHz mono format for best results

4. **GPU errors**
   - Set `WHISPER_USE_GPU=false` to use CPU
   - Verify CUDA installation

## Development Status

### Completed (vtt-mcp-csh.3.7)
- ✅ All 6 MCP tools implemented
- ✅ Session management
- ✅ Transcription history
- ✅ Audio configuration
- ✅ Error handling
- ✅ Integration tests

### Pending
- ⏳ MCP protocol trait implementation (rmcp::Service)
- ⏳ Tool registration with rmcp macros
- ⏳ Resource support for live streaming
- ⏳ Incremental transcription

## Resources

- [MCP Specification](https://modelcontextprotocol.io)
- [Whisper.cpp](https://github.com/ggerganov/whisper.cpp)
- [Goose Documentation](https://goose.dev)
