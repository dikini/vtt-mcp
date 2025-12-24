# Multi-Language Support

The VTT-MCP server supports transcription in multiple languages using OpenAI's Whisper model.

## Supported Languages

The following languages are supported for transcription:

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

## Usage

### Using the list_languages Tool

To see all available languages:

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

### Auto-detection (default)

If no language is specified, or if `"auto"` is used, the system will automatically detect the language:

```javascript
// Auto-detect language
const result = await mcp.callTool("transcribe_clip", {
  audio_file: "/path/to/audio.wav"
  // language not specified = auto-detect
});
```

## Language Validation

The server validates all language codes before processing. If an unsupported language code is provided, the server will return an error message listing the available languages.

## Implementation Notes

- Language codes follow ISO 639-1 two-letter standards
- The `auto` option uses Whisper's built-in language detection
- Language detection happens during the transcription process
- Specifying a language can improve accuracy for known languages
- Auto-detection may be slightly slower than specifying a known language
