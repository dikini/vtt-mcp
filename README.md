### /home/dikini/Projects/vtt-mcp/./README.md
```markdown
1: # Voice-to-Text MCP Server
2: 
3: A high-accuracy, low-latency voice-to-text (STT) system that integrates with Goose via the Model Context Protocol (MCP).
4: 
5: ## Status
6: 
7: **Phase 1: Foundation** - COMPLETE (2025-12-23)
8: 
9: All Phase 1 milestones achieved:
10: - Audio capture working (PipeWire + cpal)
11: - Whisper transcription functional
12: - CLI tool for testing
13: - Comprehensive documentation
14: - Performance benchmarks
15: 
16: Performance (CPU-only, base model):
17: - Cold start: 1-3s
18: - Warm start: 500ms-2s per 5s clip
19: - Memory: ~200-500MB
20: - Accuracy: >95% on clear speech
21: 
22: Next Phase: MCP Server Integration
23: 
24: See PLAN.md for complete roadmap and docs/PHASE1_REPORT.md for detailed completion report.
25: 
26: ---
27: 
28: ## Overview
29: 
30: This project enables voice communication with the Goose AI agent through:
31: - Local speech-to-text using OpenAI's Whisper model (via whisper.cpp)
32: - MCP protocol for integration with Goose
33: - Offline processing - no cloud dependencies
34: - GPU acceleration support (CUDA/ROCm optional)
35: - Linux-first with PipeWire audio (cross-platform via cpal)
36: 
37: ---
38: 
39: ## Quick Start
40: 
41: ### Prerequisites
42: 
43: - Rust 1.70+
44: - PipeWire (Linux) or CoreAudio/WASAPI (macOS/Windows)
45: - Whisper model (downloaded separately)
46: 
47: ### Installation
48: 
49: ```bash
50: # Clone the repository
51: git clone <repository-url>
52: cd vtt-mcp
53: 
54: # Download Whisper base model (142MB)
55: wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin -O models/ggml-base.bin
56: 
57: # Build the CLI tool
58: cargo build --release --package vtt-cli
59: ```
60: 
61: ### Usage
62: 
63: Record and transcribe 5 seconds of audio:
64: 
65: ```bash
66: cargo run --release --package vtt-cli -- --duration 5
67: ```
68: 
69: For detailed setup instructions, see docs/SETUP.md.
70: 
71: ---
72: 
73: ## Project Structure
74: 
75: ```
76: vtt-mcp/
77: ├── crates/
78: │   ├── vtt-core/       # Core transcription library
79: │   │   ├── audio/      # Audio capture (PipeWire/cpal)
80: │   │   ├── vad/        # Voice activity detection
81: │   │   └── whisper/    # Whisper inference
82: │   ├── vtt-cli/        # CLI tool for testing
83: │   └── vtt-mcp/        # MCP server (Phase 2)
84: ├── docs/               # Documentation
85: ├── models/             # Whisper model files
86: └── scripts/            # Utilities
87: ```
88: 
89: ---
90: 
91: ## Features
92: 
93: ### Implemented (Phase 1)
94: 
95: - Audio Capture: PipeWire native integration with cpal fallback
96: - Whisper Transcription: End-to-end speech-to-text with resampling
97: - CLI Tool: Record and transcribe from command line
98: - Multi-client Audio: Multiple processes can access audio simultaneously
99: - Comprehensive Documentation: Setup guides and benchmarks
100: 
101: ### Planned (Phase 2+)
102: 
103: - MCP Server: Integrate with Goose via Model Context Protocol
104: - Real-time Streaming: Continuous transcription with low latency
105: - GPU Acceleration: CUDA/ROCm support for faster inference
106: - VAD Integration: Voice Activity Detection for efficiency
107: 
108: ---
109: 
110: ## Performance
111: 
112: Benchmarks from Phase 1 (CPU-only, Whisper base model):
113: 
114: | Metric | Value |
115: |--------|-------|
116: | Cold Start | 1-3s (model loading) |
117: | Warm Start | 500ms-2s per 5s audio |
118: | Memory Usage | ~200-500MB |
119: | Accuracy | >95% (WER <5%) on clear speech |
120: 
121: See docs/PHASE1_REPORT.md for detailed benchmarks.
122: 
123: ---
124: 
125: ## Documentation
126: 
127: - PLAN.md - Complete implementation roadmap
128: - docs/SETUP.md - Detailed setup guide
129: - docs/PHASE1_REPORT.md - Phase 1 completion report
130: - crates/vtt-cli/README.md - CLI tool documentation
131: 
132: ---
133: 
134: ## Roadmap
135: 
136: ### Phase 1: Foundation (Complete)
137: - Research and architecture
138: - Project structure setup
139: - Audio capture implementation
140: - Whisper integration
141: - CLI tool for testing
142: - Documentation and benchmarks
143: 
144: ### Phase 2: MCP Integration (Next)
145: - MCP server scaffold
146: - Core tools (transcribe_clip, start_listening, stop_listening)
147: - State management
148: - Error handling
149: 
150: ### Phase 3: Real-Time Streaming
151: - Sliding window buffer
152: - Incremental transcription
153: - MCP streaming resources
154: 
155: ### Phase 4: Robustness & Features
156: - Configuration system
157: - Advanced features
158: - Comprehensive testing
159: - Production polish
160: 
161: ### Phase 5: Distribution
162: - Installation automation
163: - Packaging (.deb, AUR)
164: - CI/CD pipeline
165: 
166: ---
167: 
168: ## Contributing
169: 
170: This project is part of the Goose ecosystem.
171: 
172: For development guidelines, see:
173: - AGENTS.md - Guidelines for AI agents working on this project
174: - PLAN.md - Development workflow and quality gates
175: 
176: ---
177: 
178: ## License
179: 
180: GPL-3.0
181: 
182: ---
183: 
184: ## Acknowledgments
185: 
186: Built with:
187: - whisper.cpp - Whisper inference engine
188: - whisper-rs - Rust bindings
189: - PipeWire - Linux audio system
190: - MCP SDK - Model Context Protocol
```


## Implementation Status

### Phase 3: MCP Server (IN PROGRESS - 85% Complete)

| Task | Status | Description |
|------|--------|-------------|
| vtt-mcp-csh.3.1 | ✅ Complete | MCP Server scaffold with rmcp integration |
| vtt-mcp-csh.3.2 | ✅ Complete | transcribe_clip tool |
| vtt-mcp-csh.3.3 | ✅ Complete | start_listening/stop_listening tools |
| vtt-mcp-csh.3.4 | ✅ Complete | get_last_transcription tool |
| vtt-mcp-csh.3.5 | ✅ Complete | configure_audio tool |
| vtt-mcp-csh.3.6 | ✅ Complete | Error handling and logging |
| vtt-mcp-csh.3.7 | ✅ Complete | MCP integration tests |

**Summary:** All 6 MCP tools implemented with full rmcp protocol integration. Server compiles and is ready for client testing.

See [`docs/mcp-integration.md`](docs/mcp-integration.md) for MCP tool documentation.
