#!/bin/bash
# Beads task creation script - Phase 3 and 4

echo "=== Creating Phase 3: Real-time Streaming sub-tasks ==="

bd create --title "Implement sliding window buffer"   --description "Create circular buffer for audio segments. Configure 3-5 second window size. Handle buffer overflow gracefully. Add tests for buffer management. ~2h"   --parent vtt-mcp-csh.4 --priority 2 --type task

bd create --title "Add incremental transcription"   --description "Implement partial result streaming every 500ms. Add duplicate suppression logic. Handle word boundaries correctly. Test with continuous speech. ~2h"   --parent vtt-mcp-csh.4 --priority 2 --type task

bd create --title "Implement transcript://live resource"   --description "Add MCP resource for real-time streaming. Implement subscription mechanism. Push incremental updates to clients. Handle client disconnection. ~2h"   --parent vtt-mcp-csh.4 --priority 2 --type task

bd create --title "Optimize buffer and inference timing"   --description "Profile latency bottlenecks. Tune buffer sizes (1-5s range). Adjust inference intervals (250-1000ms). Measure impact on latency and accuracy. ~2h"   --parent vtt-mcp-csh.4 --priority 2 --type task

bd create --title "GPU memory optimization"   --description "Profile VRAM usage. Implement model unloading when idle. Add GPU memory monitoring. Test with multiple concurrent sessions. ~90min"   --parent vtt-mcp-csh.4 --priority 2 --type task

bd create --title "Create performance profiling report"   --description "Measure end-to-end latency (<1s target). Test accuracy on diverse speech samples. Document resource usage. Create Phase 3 completion report. ~2h"   --parent vtt-mcp-csh.4 --priority 2 --type task

echo "=== Creating Phase 4: Robustness & Features tasks ==="

bd create --title "Implement configuration system"   --description "Create TOML config schema (audio, vad, whisper, transcription, mcp sections). Add config file loading from ~/.config/vtt-mcp/. Implement runtime config updates. Document config options. ~2h"   --parent vtt-mcp-csh --priority 2 --type task

bd create --title "Add multi-language support"   --description "Implement language detection. Add language parameter to tools. Test with non-English speech samples. Document supported languages. ~2h"   --parent vtt-mcp-csh --priority 2 --type task

bd create --title "Implement transcript history resource"   --description "Create transcript://history MCP resource. Add pagination support. Implement storage backend (in-memory + optional file). Add history size limits. ~2h"   --parent vtt-mcp-csh --priority 2 --type task

bd create --title "Write unit tests for core components"   --description "Add unit tests for audio capture, VAD, Whisper wrapper, buffer management. Use mocked audio input. Target >70% coverage for vtt-core. ~3h"   --parent vtt-mcp-csh --priority 2 --type task

bd create --title "Create end-to-end tests with Goose"   --description "Setup test environment with Goose. Test voice command flow end-to-end. Verify tool responses. Test error scenarios. Document test procedures. ~2h"   --parent vtt-mcp-csh --priority 2 --type task

bd create --title "Write API documentation"   --description "Add rustdoc comments to all public APIs. Document MCP tools and resources. Create architecture overview in docs/ARCHITECTURE.md. Generate docs with cargo doc. ~2h"   --parent vtt-mcp-csh --priority 2 --type task

bd create --title "Create user guide and troubleshooting docs"   --description "Write docs/USAGE.md with examples. Create docs/TROUBLESHOOTING.md for common issues. Add FAQ section. Include screenshots/demos. ~3h"   --parent vtt-mcp-csh --priority 2 --type task

echo "Phase 3-4 tasks created successfully"
