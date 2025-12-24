# Task Specification: vtt-mcp-csh.3.5
## Implement configure_audio tool

**Parent:** vtt-mcp-csh.3 (Implement MCP Server with Transcription Tools)
**Estimate:** ~90min
**Status:** COMPLETE

### Description
Add audio device enumeration. Create configure_audio tool with device and vad_sensitivity params. Update runtime configuration. Return current config.

### Implementation Status

#### ✅ Completed:
- [x] Added runtime audio configuration storage (`AudioRuntimeConfig`)
- [x] Implemented `list_audio_devices` tool for audio device enumeration
- [x] Implemented `configure_audio` tool with device_name and vad_sensitivity parameters
- [x] Added VAD configuration support
- [x] Return current configuration with available devices
- [x] Device validation before setting as default

### Implementation Details

#### Runtime Audio Configuration
- **Storage**: Thread-safe `AudioRuntimeConfig` wrapped in `Arc<Mutex<>>`
- **Scope**: Server-wide, affects all new listening sessions
- **Fields**:
  - `default_device`: Optional device name (None = system default)
  - `vad_config`: VAD configuration with energy threshold and timing parameters

#### Tools Implemented

##### `list_audio_devices`
**Parameters:** None

**Returns:**
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

**Features:**
- Lists all available audio input devices
- Marks the system default and/or configured default device
- Uses vtt-core's `list_devices()` function

##### `configure_audio`
**Parameters:**
- `device_name`: Optional - Name of device to set as default
- `vad_sensitivity`: Optional - VAD energy threshold (0.0 to 1.0)

**Returns:**
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

**Features:**
- Validates device exists before setting as default
- Clamps VAD sensitivity to 0.0-1.0 range
- Returns updated configuration and all available devices
- Errors if device not found

#### VAD Configuration
The `vad_sensitivity` parameter controls the energy threshold:
- **Lower values** (e.g., 0.005) = More sensitive, detects quieter speech
- **Higher values** (e.g., 0.02) = Less sensitive, ignores background noise
- **Default**: 0.01

Additional VAD parameters (not currently configurable):
- `speech_frames_threshold`: 3 - frames above threshold to trigger speech
- `silence_frames_threshold`: 10 - frames below threshold for silence
- `min_speech_duration`: 30 - minimum frames for valid speech segment

### Files Modified
- `crates/vtt-mcp/src/server.rs` - Added:
  - `AudioRuntimeConfig` struct
  - `audio_config` field to `VttMcpServer`
  - `ListAudioDevicesParams`, `AudioDeviceInfo`, `AudioDevicesListResult` structs
  - `ConfigureAudioParams`, `AudioConfigurationResult`, `VadConfigInfo` structs
  - `list_audio_devices` tool method
  - `configure_audio` tool method
  - Import for `list_devices` and `VadConfig`

### Test Results
```
running 3 tests
test tests::test_version ... ok
test server::tests::test_session_status_display ... ok
test server::tests::test_server_creation ... ok

test result: ok. 3 passed; 0 failed; 0 ignored
```

### Usage Examples

#### Example 1: List available audio devices
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list_audio_devices","arguments":{}}}' | cargo run --package vtt-mcp
```

#### Example 2: Set default audio device
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"configure_audio","arguments":{"device_name":"USB Microphone"}}}' | cargo run --package vtt-mcp
```

#### Example 3: Adjust VAD sensitivity
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"configure_audio","arguments":{"vad_sensitivity":0.02}}}' | cargo run --package vtt-mcp
```

#### Example 4: Configure both device and VAD
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"configure_audio","arguments":{"device_name":"USB Microphone","vad_sensitivity":0.005}}}' | cargo run --package vtt-mcp
```

### Notes
- Configuration is stored in-memory only (not persisted across restarts)
- Affects only new listening sessions (not active sessions)
- Device validation prevents setting non-existent devices as default
- VAD sensitivity is a simplified interface to the full VAD configuration
- The full VAD configuration is returned for reference but not all fields are configurable

### Future Enhancements
- Add per-device VAD profiles
- Support for audio format configuration (sample rate, channels)
- Persist configuration to disk
- Add `get_audio_config` tool for read-only access

### Status
✅ COMPLETE - Audio configuration tools implemented successfully.
