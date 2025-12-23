# Task Specification: vtt-mcp-csh.1.3
## Implement Basic Audio Capture to WAV

**Task ID**: vtt-mcp-csh.1.3  
**Parent Task**: vtt-mcp-csh.1 (Project Setup and Audio Capture Pipeline)  
**Estimated Time**: ~90 minutes  
**Status**: ✅ Complete  
**Date**: 2025-12-23  
**Implementation**: PipeWire native API (Linux) with cpal fallback (other platforms)

---

## COMPLETION SUMMARY

**Completed**: 2025-12-23 16:37  
**Commit**: 694e973  
**Implementation**: PipeWire native API with multi-client support

### Key Achievements

✅ PipeWire Native Implementation on Linux (instead of cpal+ALSA)
- Multi-client audio capture support
- Threaded event loop model
- Real-time audio callback processing
- No "device busy" errors

✅ Platform Abstraction
- Linux: PipeWire native API
- macOS/Windows: cpal fallback

✅ All Acceptance Criteria Met
- Device enumeration working
- Audio capture functional (48kHz stereo)
- WAV file writing implemented
- All tests passing (11 tests)
- Multi-client capture verified

### Implementation Notes

**Key Changes from Original Spec:**
1. PipeWire Native API - Direct integration instead of ALSA compatibility layer
2. 48kHz stereo default - Hardware-native format (vs 16kHz mono in spec)
3. Multi-client support - Multiple processes can capture simultaneously
4. Threaded architecture - Event loop runs in background thread

**Dependencies Added:**
- pipewire = "0.9" (Linux only)
- System: libpipewire-0.3-dev, libspa-0.2-dev, clang, libclang-dev

**Files Created:**
- crates/vtt-core/src/audio/pipewire_capture.rs (250+ lines)
- crates/vtt-core/src/audio/cpal_capture.rs (platform fallback)
- crates/vtt-core/src/audio/format.rs (audio configuration)
- crates/vtt-core/src/audio/writer.rs (WAV output)
- crates/vtt-core/tests/audio_integration.rs (integration tests)

**Testing:**
- Unit tests: ✅ All passing
- Integration tests: ✅ Device enumeration working
- Build: ✅ Successful (doc warnings only)
- Multi-client: ✅ Verified (multiple instances can capture)

**Known Limitations:**
1. Stop mechanism waits for thread join (could add mainloop.quit() for cleaner shutdown)
2. No error channel from PipeWire thread to main thread
3. Format accepts whatever PipeWire negotiates (typically 48kHz)

---

## Original Specification (For Reference)

### 1. Overview

Implement functional audio capture using **PipeWire native API on Linux** (with cpal fallback on other platforms) to record microphone input and save it to WAV files. This task creates the core audio capture functionality with **multi-client support** for the VTT-MCP system.

### Context
- **Previous Task (1.2)**: Configured cpal, hound, anyhow, thiserror dependencies and created audio module structure with stubs
- **Current Task (1.3)**: Implement actual audio capture functionality with device enumeration and WAV recording
- **Next Task (1.4)**: Integrate Silero VAD for speech detection

### Implementation Notes

**Platform Abstraction:**
- **Linux**: PipeWire native API via pipewire crate (multi-client support)
- **macOS/Windows**: cpal library (CoreAudio/WASAPI)

**Key Changes from Original Spec:**
1. **PipeWire Native API** on Linux instead of cpal+ALSA
2. **48kHz stereo** default format (hardware-native for most devices)
3. **Multi-client capture** support (multiple processes can capture simultaneously)
4. **Threaded event loop** model for PipeWire

---

## 2. Objectives

### Primary Objectives
1. **Device Enumeration**: List and select available audio input devices
2. **Audio Capture**: Capture live microphone input using platform-specific API
3. **WAV Recording**: Write captured audio to WAV files
4. **Multi-Client Support**: Allow multiple capture instances on Linux (PipeWire)
5. **Error Handling**: Robust error handling for device and stream failures

### Success Criteria
- ✅ Can list all available audio input devices
- ✅ Can identify and select the default input device
- ✅ Can capture audio at 48kHz sample rate (Linux) or 16kHz (other platforms)
- ✅ Can write captured audio to WAV file format
- ✅ All tests pass
- ✅ Quality gates (build, test, format, clippy, doc) pass
- ✅ Works on PipeWire-based Linux systems with multi-client support

---

## 3. Implementation (COMPLETED)

### 3.1 File Structure

```
crates/vtt-core/src/audio/
├── mod.rs              # Platform abstraction exports
├── error.rs            # ✅ Complete (from task 1.2)
├── capture.rs          # ✅ Platform abstraction layer
├── cpal_capture.rs     # ✅ cpal implementation (macOS/Windows)
├── pipewire_capture.rs # ✅ PipeWire implementation (Linux)
├── device.rs           # ✅ Device enumeration
├── format.rs           # ✅ AudioFormat configuration
└── writer.rs           # ✅ WAV file writing
```

### 3.2 Platform Abstraction

Linux uses PipeWireCapture, other platforms use CpalCapture.

### 3.3 PipeWire Implementation

Spawns thread to run PipeWire event loop on start(). Real-time audio callback pushes samples to shared buffer.

### 3.4 cpal Fallback

Standard cpal implementation for macOS (CoreAudio) and Windows (WASAPI).

---

## 4. Acceptance Criteria (COMPLETED)

- [x] AC1: list_devices() returns available input devices
- [x] AC2: default_device() returns default input device or error
- [x] AC3: AudioCapture::new() creates capture instance
- [x] AC4: capture.start() begins audio recording
- [x] AC5: capture.stop() ends audio recording
- [x] AC6: capture.take_buffer() returns captured samples
- [x] AC7: write_wav() creates valid WAV file
- [x] AC8: AudioFormat configured correctly (48kHz stereo on Linux)
- [x] AC9: All unit tests pass (cargo test)
- [x] AC10: Integration test succeeds with audio hardware
- [x] AC11: CLI tool successfully captures and saves audio
- [x] AC12: Works on PipeWire-based Linux system
- [x] AC13: Build passes (documentation warnings only)
- [x] AC14: Code is documented

---

## 5. Sign-Off

**Implementation Completed**: 2025-12-23  
**Status**: ✅ COMPLETE  
**Platform**: Linux (PipeWire native), macOS/Windows (cpal fallback)  
**Multi-Client Support**: ✅ Yes (Linux via PipeWire)  
**All Acceptance Criteria**: ✅ Met  
**Quality Gates**: ✅ Passed  

### Files Modified/Created

**Modified:**
- crates/vtt-core/src/audio/capture.rs - Platform abstraction
- crates/vtt-core/src/audio/device.rs - Device enumeration
- crates/vtt-core/src/audio/mod.rs - Module exports
- crates/vtt-core/Cargo.toml - Added PipeWire dependency
- crates/vtt-cli/src/main.rs - Updated to use with_format

**Created:**
- crates/vtt-core/src/audio/pipewire_capture.rs - PipeWire implementation
- crates/vtt-core/src/audio/cpal_capture.rs - cpal implementation
- crates/vtt-core/src/audio/format.rs - Audio format configuration
- crates/vtt-core/src/audio/writer.rs - WAV file writing
- crates/vtt-core/tests/audio_integration.rs - Integration tests
- PIPEWIRE_STATUS.md - Implementation documentation
