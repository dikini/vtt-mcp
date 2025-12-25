# Voice-to-Text MCP Server (vtt-mcp)

## Overview
The Voice-to-Text MCP Server (vtt-mcp) enables high-accuracy, low-latency transcription of voice commands and integrates seamlessly with the Goose AI agent. By leveraging Whisper-based Speech-to-Text (STT) models, the system provides outstanding transcription while preserving user data privacy by running completely offline.

### Key Features
- Real-time transcription for active audio input.
- Push-to-Talk support for single-session recordings.
- Historical log management for saved transcripts.
- Linux-first design with PipeWire audio support.

---

## Installation
The vtt-mcp requires the following tools to be installed:
1. Rust programming language. Install it using rustup.
2. PipeWire audio framework to handle system audio.
3. Whisper machine learning models downloaded from whisper.cpp.

To install and run:
- Clone the repository from GitHub.
- Navigate to the root folder and build the project using Cargo.
- Launch vtt-mcp server from the terminal.

## Usage
Here are common vtt-mcp commands:
- Start real-time transcription: ./vtt-mcp --start-session
- Retrieve logs: ./vtt-mcp --get-log
- Transcribe single audio clip with specific duration: ./vtt-mcp --transcribe-clip --duration <seconds>
- List all commands: ./vtt-mcp --help

## Contribution
Currently, there is no contribution guide (CONTRIBUTING.md). If you'd like to contribute, please open an issue or contact the maintainers.

## License
This project is licensed under the GNU General Public License v3.0 (GPLv3). See the LICENSE file for the complete terms.
