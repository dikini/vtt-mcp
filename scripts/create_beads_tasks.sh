#!/bin/bash
# Beads task creation script - Phase 1 detailed breakdown

echo "=== Creating Phase 1 detailed sub-tasks ==="

# Phase 1.1: Cargo workspace setup (breaking down csh.1)
bd create --title "Setup Cargo workspace structure"   --description "Create crates/ directory with vtt-core, vtt-mcp, vtt-cli subdirectories. Initialize Cargo.toml workspace. Configure workspace members. Setup basic lib.rs/main.rs files. ~30min"   --parent vtt-mcp-csh.1 --priority 2 --type task

bd create --title "Configure audio dependencies (cpal)"   --description "Add cpal to vtt-core/Cargo.toml. Research and configure audio backend features for PipeWire. Create audio module stub in vtt-core/src/audio/. ~45min"   --parent vtt-mcp-csh.1 --priority 2 --type task

bd create --title "Implement basic audio capture to WAV"   --description "Use cpal to enumerate audio devices. Implement microphone capture with 16kHz mono f32 format. Write captured audio to WAV file for testing. Verify PipeWire works. ~90min"   --parent vtt-mcp-csh.1 --priority 2 --type task

bd create --title "Integrate Silero VAD"   --description "Add silero-vad-rs dependency. Implement VAD module in vtt-core/src/vad/. Test speech/silence detection on sample audio. Configure sensitivity thresholds. ~2h"   --parent vtt-mcp-csh.1 --priority 2 --type task

echo "=== Creating Phase 1.2: Whisper integration sub-tasks ==="

bd create --title "Setup whisper-rs with GPU support"   --description "Add whisper-rs to Cargo.toml with cuda/hipblas features. Download Whisper base model to models/ directory. Verify GPU acceleration is working. ~1h"   --parent vtt-mcp-csh.2 --priority 2 --type task

bd create --title "Implement Whisper inference wrapper"   --description "Create whisper module in vtt-core. Load model into memory. Implement transcribe function for audio buffers. Handle errors gracefully. ~90min"   --parent vtt-mcp-csh.2 --priority 2 --type task

bd create --title "Create CLI tool for testing"   --description "Build vtt-cli that records audio and transcribes. Add command-line args for duration, model, output. Test end-to-end pipeline. ~1h"   --parent vtt-mcp-csh.2 --priority 2 --type task

bd create --title "Benchmark and document Phase 1"   --description "Measure latency from audio end to text. Test accuracy on sample speech. Document setup steps in docs/SETUP.md. Create Phase 1 completion report. ~90min"   --parent vtt-mcp-csh.2 --priority 2 --type task

echo "=== Creating Phase 2: MCP Integration sub-tasks ==="

bd create --title "Setup MCP server scaffold"   --description "Add rmcp SDK to vtt-mcp/Cargo.toml. Setup stdio transport. Implement basic MCP server struct with tool registration. Test with echo tool. ~2h"   --parent vtt-mcp-csh.3 --priority 2 --type task

bd create --title "Implement transcribe_clip tool"   --description "Create MCP tool for one-shot transcription. Handle duration and model parameters. Wire up to vtt-core transcription engine. Return JSON with text and confidence. ~90min"   --parent vtt-mcp-csh.3 --priority 2 --type task

bd create --title "Implement start_listening/stop_listening"   --description "Add session management with UUIDs. Implement start_listening tool with model/language params. Add stop_listening with session_id param. Return session state. ~2h"   --parent vtt-mcp-csh.3 --priority 2 --type task

bd create --title "Add get_last_transcription tool"   --description "Store recent transcriptions in state. Implement get_last_transcription tool. Return text, timestamp, confidence. Add transcript history management. ~1h"   --parent vtt-mcp-csh.3 --priority 2 --type task

bd create --title "Implement configure_audio tool"   --description "Add audio device enumeration. Create configure_audio tool with device and vad_sensitivity params. Update runtime configuration. Return current config. ~90min"   --parent vtt-mcp-csh.3 --priority 2 --type task

bd create --title "Add error handling and logging"   --description "Setup tracing/tracing-subscriber. Define VttError enum hierarchy. Add error propagation to MCP responses. Implement graceful failure modes. ~2h"   --parent vtt-mcp-csh.3 --priority 2 --type task

bd create --title "Write MCP integration tests"   --description "Create test script for MCP tool invocation. Test each tool with mock inputs. Verify error handling. Document integration with Goose. ~2h"   --parent vtt-mcp-csh.3 --priority 2 --type task

echo "Phase 1-2 sub-tasks created successfully"
