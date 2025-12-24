# Task 5: Configuration System

## Overview
Implement a comprehensive configuration system for the VTT-MCP server using TOML format.

## Sub-tasks

### 5.1 Design TOML Schema
Create configuration schema with sections for audio, vad, whisper, transcription, mcp, memory.

### 5.2 Implement Config Loader
Add toml and serde dependencies. Create vtt-core/src/config with schema, loader, validator modules.

### 5.3 Runtime Config Updates
Implement hot-reload for config changes and add validation.
