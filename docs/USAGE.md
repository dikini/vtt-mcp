# VTT-MCP User Guide

Complete guide for using the VTT-MCP voice-to-text server with Goose and other MCP clients.

## Table of Contents

- [Quick Start](#quick-start)
- [Basic Usage](#basic-usage)
- [MCP Tools Reference](#mcp-tools-reference)
- [Real-Time Transcription](#real-time-transcription)
- [File Transcription](#file-transcription)
- [Multi-Language Support](#multi-language-support)
- [Configuration](#configuration)
- [Advanced Usage](#advanced-usage)

---

## Quick Start

### 1. Start the MCP Server

```bash
# Build the server
cargo build --release --package vtt-mcp

# Start the server (stdio mode)
cargo run --release --package vtt-mcp
```

### 2. Connect from Goose

In your Goose configuration:

```typescript
// goose.config.ts
export default {
  mcpServers: {
    "vtt-mcp": {
      command: "cargo",
      args: ["run", "--release", "--package", "vtt-mcp"]
    }
  }
};
```

### 3. Transcribe Audio

```typescript
// List available languages
const languages = await mcp.callTool("list_languages", {});

// Start real-time transcription
const session = await mcp.callTool("start_listening", {
  session_name: "My Session",
  language: "en"
});

// Speak into your microphone...

// Stop and get final transcription
const result = await mcp.callTool("stop_listening", {
  session_id: session.session_id
});

console.log(result.final_transcript);
```

---

## Basic Usage

### File Transcription

Transcribe a pre-recorded audio file:

```typescript
const result = await mcp.callTool("transcribe_clip", {
  audio_file: "/path/to/audio.wav",
  language: "en"
});

console.log(`Transcription: ${result.text}`);
console.log(`Duration: ${result.duration_ms}ms`);
```

**Supported formats**: WAV, MP3, FLAC, OGG

### Real-Time Transcription

Start listening and receive real-time updates:

```typescript
// 1. Start listening
const session = await mcp.callTool("start_listening", {
  session_name: "Meeting Notes",
  language: "auto"  // Auto-detect language
});

console.log(`Session ID: ${session.session_id}`);

// 2. Subscribe to live updates
// (Implementation depends on your MCP client)

// 3. Check for updates periodically
const latest = await mcp.callTool("get_last_transcription", {
  session_id: session.session_id
});

if (latest) {
  console.log(`Latest: ${latest.text}`);
}

// 4. Stop when done
const final = await mcp.callTool("stop_listening", {
  session_id: session.session_id
});

console.log(`Final: ${final.final_transcript}`);
```

---

## MCP Tools Reference

### transcribe_clip

Transcribe an audio file.

**Parameters:**
- `audio_file` (string, required): Path to audio file
- `language` (string, optional): Language code, e.g., "en", "es"
- `model_path` (string, optional): Custom model path

**Example:**
```typescript
// Transcribe Spanish audio
const result = await mcp.callTool("transcribe_clip", {
  audio_file: "/home/user/audio.wav",
  language: "es"
});
```

---

### start_listening

Begin real-time microphone transcription.

**Parameters:**
- `session_name` (string, optional): Friendly name for session
- `language` (string, optional): Language code (default: "auto")
- `vad_threshold` (number, optional): VAD sensitivity (0.0-1.0, default 0.01)
- `model_path` (string, optional): Custom model path

**Returns:**
- `session_id` (string): UUID for this session
- `status` (string): "capturing" or "processing"
- `model_path` (string): Model being used

**Example:**
```typescript
const session = await mcp.callTool("start_listening", {
  session_name: "Dictation",
  language: "en",
  vad_threshold: 0.02  // Less sensitive
});

console.log(`Session started: ${session.session_id}`);
```

---

### stop_listening

Stop an active session and get final transcription.

**Parameters:**
- `session_id` (string, required): Session UUID from start_listening

**Returns:**
- `session_id` (string): Stopped session ID
- `final_transcript` (string): Complete transcription
- `duration_ms` (number): Total audio duration
- `text_length` (number): Character count

**Example:**
```typescript
const result = await mcp.callTool("stop_listening", {
  session_id: "uuid-here"
});

console.log(`Duration: ${result.duration_ms}ms`);
console.log(`Text: ${result.final_transcript}`);
```

---

### get_last_transcription

Get the most recent transcription result.

**Parameters:**
- `session_id` (string, optional): Get from specific session

**Returns:**
- `text` (string): Transcribed text
- `is_final` (boolean): Whether final or partial
- `timestamp` (string): ISO 8601 timestamp
- `session_id` (string): Session UUID

**Example:**
```typescript
// Get most recent result from any session
const result = await mcp.callTool("get_last_transcription", {});

// Or from a specific session
const specific = await mcp.callTool("get_last_transcription", {
  session_id: "uuid-here"
});
```

---

### list_languages

List all supported languages.

**Parameters:** None

**Returns:** Array of language objects with `code` and `name`

**Example:**
```typescript
const result = await mcp.callTool("list_languages", {});

result.languages.forEach(lang => {
  console.log(`${lang.code}: ${lang.name}`);
});
```

---

### list_audio_devices

List available audio input devices.

**Parameters:** None

**Returns:**
- `default_device` (string or null): Default device name
- `available_devices` (array): Device info

**Example:**
```typescript
const result = await mcp.callTool("list_audio_devices", {});

console.log("Default:", result.default_device);
result.available_devices.forEach(device => {
  if (device.is_default) {
    console.log(`✓ ${device.name}`);
  } else {
    console.log(`  ${device.name}`);
  }
});
```

---

### configure_audio

Update audio and VAD settings.

**Parameters:**
- `default_device` (string, optional): Set default device
- `vad_config` (object, optional): VAD settings
  - `threshold` (number): Speech threshold (0.0-1.0)
  - `speech_frames` (number): Speech debounce (default 3)
  - `silence_frames` (number): Silence hangover (default 10)

**Example:**
```typescript
// Increase VAD sensitivity
const result = await mcp.callTool("configure_audio", {
  vad_config: {
    threshold: 0.005,  // More sensitive
    speech_frames: 2,
    silence_frames: 15  // Longer hangover
  }
});
```

---

## Real-Time Transcription

### Session Lifecycle

```typescript
// 1. Create session
const session = await mcp.callTool("start_listening", {
  session_name: "My Meeting",
  language: "en"
});

// 2. Monitor progress
let lastText = "";
setInterval(async () => {
  const result = await mcp.callTool("get_last_transcription", {
    session_id: session.session_id
  });
  
  if (result && result.text !== lastText) {
    console.log(`Update: ${result.text}`);
    lastText = result.text;
  }
}, 500);

// 3. Stop when finished
const final = await mcp.callTool("stop_listening", {
  session_id: session.session_id
});
```

### VAD Configuration

Adjust Voice Activity Detection for your environment:

```typescript
// Quiet environment (more sensitive)
await mcp.callTool("configure_audio", {
  vad_config: {
    threshold: 0.005,
    speech_frames: 2,
    silence_frames: 8
  }
});

// Noisy environment (less sensitive)
await mcp.callTool("configure_audio", {
  vad_config: {
    threshold: 0.02,
    speech_frames: 5,
    silence_frames: 15
  }
});
```

---

## File Transcription

### Supported Audio Formats

- **WAV**: Best quality, recommended
- **MP3**: Widely compatible
- **FLAC**: Lossless compression
- **OGG**: Open format

### Batch Processing

Transcribe multiple files:

```typescript
const files = [
  "/path/to/audio1.wav",
  "/path/to/audio2.wav",
  "/path/to/audio3.wav"
];

for (const file of files) {
  const result = await mcp.callTool("transcribe_clip", {
    audio_file: file,
    language: "en"
  });
  
  console.log(`${file}: ${result.text}`);
}
```

---

## Multi-Language Support

### Auto-Detection

Let Whisper detect the language:

```typescript
const result = await mcp.callTool("transcribe_clip", {
  audio_file: "/path/to/audio.wav",
  language: "auto"  // or omit parameter
});
```

### Specific Language

Specify language for better accuracy:

```typescript
const languages = {
  english: "en",
  spanish: "es",
  french: "fr",
  german: "de",
  italian: "it",
  portuguese: "pt",
  chinese: "zh",
  japanese: "ja",
  korean: "ko",
  russian: "ru",
  arabic: "ar",
  hindi: "hi"
};

const result = await mcp.callTool("transcribe_clip", {
  audio_file: "/path/to/audio.wav",
  language: languages.spanish
});
```

---

## Configuration

### Configuration File

Create `~/.config/vtt-mcp/config.toml`:

```toml
[audio]
sample_rate = 16000
channels = 1

[vad]
threshold = 0.01
speech_frames = 3
silence_frames = 10

[whisper]
model_size = "base"
threads = 4
enable_gpu = true

[whisper.memory]
idle_timeout_secs = 300
max_sessions = 10
```

### Environment Variables

```bash
# Set model path
export VTT_MODEL_PATH="/path/to/model.bin"

# Set log level
export RUST_LOG=debug

# Disable GPU
export VTT_NO_GPU=1
```

---

## Advanced Usage

### Custom Model Path

Use a different Whisper model:

```typescript
const result = await mcp.callTool("transcribe_clip", {
  audio_file: "/path/to/audio.wav",
  model_path: "/custom/path/ggml-small.bin"
});
```

### Transcript History

Access paginated history:

```typescript
// Get first page (20 items)
const page1 = await mcp.readResource("transcript://history");

// Get specific page
const page2 = await mcp.readResource("transcript://history?page=1&size=50");

// Parse JSON
const history = JSON.parse(page1);
history.forEach(entry => {
  console.log(`${entry.timestamp}: ${entry.transcription.text}`);
});
```

### Performance Tips

1. **Use GPU acceleration** for 3-5x speedup
2. **Use smaller models** (tiny/base) for faster transcription
3. **Increase threads** on multi-core systems
4. **Batch process** files for efficiency

### Error Handling

```typescript
try {
  const result = await mcp.callTool("transcribe_clip", {
    audio_file: "/path/to/audio.wav"
  });
} catch (error) {
  if (error.message.includes("DeviceNotFound")) {
    console.error("Microphone not available");
  } else if (error.message.includes("InvalidParameters")) {
    console.error("Invalid file path");
  } else {
    console.error("Transcription failed:", error.message);
  }
}
```

---

## Best Practices

### For Dictation

```typescript
await mcp.callTool("configure_audio", {
  vad_config: {
    threshold: 0.01,    // Standard sensitivity
    speech_frames: 3,   // Quick response
    silence_frames: 15  // Wait before ending
  }
});
```

### For Meetings

```typescript
await mcp.callTool("start_listening", {
  session_name: "Team Meeting",
  language: "auto",  // Multi-language
  vad_threshold: 0.015  // Moderate sensitivity
});
```

### For Noisy Environments

```typescript
await mcp.callTool("configure_audio", {
  vad_config: {
    threshold: 0.02,    // Less sensitive
    speech_frames: 5,   // Require more speech
    silence_frames: 20  # Longer hangover
  }
});
```

---

## Next Steps

- [API Reference](API.md) - Complete API documentation
- [Troubleshooting](TROUBLESHOOTING.md) - Common issues and solutions
- [Architecture](ARCHITECTURE.md) - System design and internals
- [Languages](LANGUAGES.md) - Multi-language support details
