# VTT-MCP API Reference

## MCP Tools

### transcribe_clip

Transcribe an audio file.

**Parameters:**
- `audio_file` (string, required): Path to audio file (WAV, MP3, FLAC)
- `language` (string, optional): Language code (e.g., "en", "es") or "auto" for detection
- `model_path` (string, optional): Path to Whisper model file

**Returns:**
- `text` (string): Transcribed text
- `duration_ms` (number): Audio duration in milliseconds
- `timestamp` (string): ISO 8601 timestamp
- `model_path` (string): Model used for transcription

**Example:**
```javascript
const result = await mcp.callTool("transcribe_clip", {
  audio_file: "/path/to/audio.wav",
  language: "en"
});
```

---

### start_listening

Start real-time transcription from microphone.

**Parameters:**
- `session_name` (string, optional): Friendly name for the session
- `language` (string, optional): Language code or "auto" (default)
- `vad_threshold` (number, optional): VAD energy threshold (0.0-1.0, default 0.01)
- `model_path` (string, optional): Path to Whisper model file

**Returns:**
- `session_id` (string): Unique session identifier (UUID)
- `status` (string): "capturing" or "processing"
- `model_path` (string): Model being used

**Example:**
```javascript
const result = await mcp.callTool("start_listening", {
  session_name: "My Session",
  language: "en"
});
console.log("Session ID:", result.session_id);
```

---

### stop_listening

Stop an active transcription session.

**Parameters:**
- `session_id` (string, required): Session ID from start_listening

**Returns:**
- `session_id` (string): Stopped session ID
- `final_transcript` (string): Complete transcription
- `duration_ms` (number): Total audio duration
- `text_length` (number): Number of characters

**Example:**
```javascript
const result = await mcp.callTool("stop_listening", {
  session_id: "uuid-here"
});
console.log("Final transcript:", result.final_transcript);
```

---

### get_last_transcription

Get the most recent transcription result.

**Parameters:**
- `session_id` (string, optional): Get last result from specific session

**Returns:**
- `text` (string): Transcribed text
- `is_final` (boolean): Whether this is a final result
- `timestamp` (string): ISO 8601 timestamp
- `session_id` (string): Session UUID
- `model_path` (string): Model used

**Example:**
```javascript
const result = await mcp.callTool("get_last_transcription", {});
console.log("Last result:", result.text);
```

---

### list_languages

List supported languages for transcription.

**Parameters:** None

**Returns:**
- `languages` (array of objects): Supported languages
  - `code` (string): Language code
  - `name` (string): Language name

**Example:**
```javascript
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
- `available_devices` (array of objects):
  - `name` (string): Device name
  - `is_default` (boolean): Whether this is the default device

**Example:**
```javascript
const result = await mcp.callTool("list_audio_devices", {});
console.log("Default:", result.default_device);
result.available_devices.forEach(device => {
  console.log(`- ${device.name}`);
});
```

---

### configure_audio

Update audio and VAD configuration.

**Parameters:**
- `default_device` (string, optional): Set default audio device
- `vad_config` (object, optional): VAD configuration
  - `threshold` (number): Energy threshold (0.0-1.0)
  - `speech_frames` (number): Speech debounce frames
  - `silence_frames` (number): Silence hangover frames

**Returns:**
- `default_device` (string or null): Current default device
- `vad_config` (object): Current VAD configuration
- `available_devices` (array): List of available devices

**Example:**
```javascript
const result = await mcp.callTool("configure_audio", {
  vad_config: {
    threshold: 0.02,
    speech_frames: 5
  }
});
```

---

## MCP Resources

### transcript://history

Retrieve paginated transcript history.

**Query Parameters:**
- `page` (number, optional): Page number (default 0)
- `size` (number, optional): Items per page (default 20, max 100)

**Returns:** JSON array of history entries:
```json
[
  {
    "session_id": "uuid",
    "timestamp": "2025-12-24T12:00:00Z",
    "transcription": {
      "text": "Transcribed text here",
      "is_final": true,
      "timestamp": "2025-12-24T12:00:00Z",
      "duration_ms": 5000
    }
  }
]
```

**Example:**
```javascript
// Get first page with default size (20)
const page1 = await mcp.readResource("transcript://history");

// Get second page with custom size
const page2 = await mcp.readResource("transcript://history?page=1&size=50");
```

---

### transcript://live/{session_id}

Subscribe to real-time transcription updates for a session.

**Parameters:**
- `session_id` (string): Session UUID from start_listening

**Returns:** Server-Sent Events (SSE) stream with transcription updates.

**Example:**
```javascript
const resource = `transcript://live/${sessionId}`;
// Subscribe to resource notifications
// Updates will be pushed as they arrive
```

---

## Data Types

### TranscriptionResult

```typescript
{
  text: string;           // Transcribed text
  is_final: boolean;      // Whether result is final
  timestamp: string;      // ISO 8601 timestamp
  duration_ms: number;    // Audio duration in ms
}
```

### SessionStatus

Possible values:
- `"Idle"`: Session not started
- `"Capturing"`: Recording audio
- `"Processing"`: Transcribing audio
- `"Error: {message}"`: Error occurred

---

## Error Handling

Errors are returned with descriptive messages:

- `InvalidParameters`: Bad input parameters
- `DeviceNotFound`: Audio device unavailable
- `TranscriptionFailed`: Whisper error
- `InternalError`: Unexpected server error

---

## Language Codes

| Code | Language |
|------|----------|
| auto | Auto-detect |
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

See [docs/LANGUAGES.md](LANGUAGES.md) for details.
