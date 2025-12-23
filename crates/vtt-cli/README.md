# VTT-CLI - Voice-to-Text Command Line Tool

End-to-end speech-to-text tool using Whisper for transcription.

## Features

- 🎤 **Record Audio**: Capture from microphone (PipeWire on Linux, cpal on macOS/Windows)
- 🧠 **Transcribe**: Uses OpenAI's Whisper model via whisper-rs
- 💾 **Save Results**: Output transcription to file or stdout
- 🔧 **Configurable**: Adjust duration, model, threads, and more

## Installation

```bash
cargo build --release --package vtt-cli
```

The binary will be at `target/release/vtt-cli`.

## Usage

### Basic Usage

Record for 5 seconds and transcribe:

```bash
cargo run --package vtt-cli
```

### Command-Line Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--duration` | `-d` | Recording duration in seconds | 5 |
| `--model` | `-m` | Path to Whisper model file | models/ggml-base.bin |
| `--output` | `-o` | Save transcription to file | stdout |
| `--threads` | `-t` | Number of threads for transcription | auto |
| `--list-devices` | | List audio devices and exit | - |
| `--save-audio` | | Save captured audio to WAV file | - |
| `--help` | `-h` | Show help | - |
| `--version` | `-V` | Show version | - |

### Examples

**Record for 10 seconds:**

```bash
vtt-cli --duration 10
```

**Save transcription to file:**

```bash
vtt-cli --duration 5 --output transcript.txt
```

**Save both audio and transcription:**

```bash
vtt-cli -d 5 --save-audio recording.wav -o transcript.txt
```

**Use custom model:**

```bash
vtt-cli --model models/ggml-small.bin
```

**List available audio devices:**

```bash
vtt-cli --list-devices
```

**Use 8 threads for faster transcription:**

```bash
vtt-cli --threads 8
```

## Model Setup

The tool requires a Whisper model file. Download from:

- Base (142MB): `wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin -O models/ggml-base.bin`
- Small (462MB): `wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin -O models/ggml-small.bin`

See [whisper.cpp models](https://github.com/ggerganov/whisper.cpp#models) for more options.

## End-to-End Pipeline

The CLI implements the complete pipeline:

1. **Capture**: Record audio from microphone via PipeWire/cpal
2. **Load**: Initialize Whisper model with configuration
3. **Transcribe**: Convert speech to text (with automatic resampling to 16kHz)
4. **Output**: Display or save transcription

## Requirements

- Rust 1.70+
- PipeWire (Linux) or CoreAudio/WASAPI (macOS/Windows)
- Whisper model file (download separately)

## License

GPL-3.0
