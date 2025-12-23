# Task Specification: vtt-mcp-csh.1.2
## Configure Audio Dependencies (cpal)

**Task ID:** vtt-mcp-csh.1.2  
**Parent Task:** vtt-mcp-csh.1 (Project Setup and Audio Capture Pipeline)  
**Status:** Open  
**Priority:** P2  
**Estimated Effort:** ~45 minutes  
**Created:** 2025-12-23  
**Updated:** 2025-12-23  

---

## 1. Overview

This task configures the audio processing dependencies for the vtt-core crate, specifically the `cpal` (Cross-Platform Audio Library) which will be used for real-time audio capture. The task also involves researching PipeWire compatibility and creating the initial audio module structure.

### Context
- **Prerequisite:** Task vtt-mcp-csh.1.1 (workspace structure) is complete ✅
- **Next Task:** vtt-mcp-csh.1.3 will implement actual audio capture using these dependencies
- **Purpose:** Enable cross-platform audio input with PipeWire support on Linux

---

## 2. Objectives

1. **Add cpal dependency** to vtt-core with appropriate version and features
2. **Research and configure PipeWire compatibility** for Linux audio backend
3. **Create audio module stub** with proper structure in vtt-core/src/audio/
4. **Add workspace-level audio dependencies** for shared usage
5. **Verify build succeeds** with new dependencies

---

## 3. Requirements

### 3.1 Functional Requirements

#### FR1: cpal Dependency Configuration
- **FR1.1:** Add `cpal` to vtt-core/Cargo.toml dependencies
- **FR1.2:** Use latest stable version (0.15.x or newer)
- **FR1.3:** Configure appropriate features for cross-platform support
- **FR1.4:** Ensure compatibility with Rust edition 2021

#### FR2: PipeWire Support
- **FR2.1:** Research cpal's ALSA backend compatibility with PipeWire
- **FR2.2:** Document any required system-level configurations
- **FR2.3:** Note potential fallback options (ALSA, PulseAudio)

#### FR3: Audio Module Structure
- **FR3.1:** Create `src/audio/` directory in vtt-core
- **FR3.2:** Create `mod.rs` with module documentation
- **FR3.3:** Create stub files for future implementations (capture.rs, device.rs)
- **FR3.4:** Export audio module from lib.rs

#### FR4: Additional Dependencies
- **FR4.1:** Add hound crate for WAV file I/O (needed for task 1.3)
- **FR4.2:** Add anyhow for error handling
- **FR4.3:** Add thiserror for custom error types

### 3.2 Non-Functional Requirements

#### NFR1: Documentation
- Module-level documentation for audio module
- Inline comments explaining PipeWire considerations
- README notes about audio backend requirements

#### NFR2: Build Performance
- Dependencies should not significantly increase build time
- Use default features unless specific features needed

#### NFR3: Cross-Platform Support
- Configuration must work on Linux (primary), macOS, Windows
- Use platform-agnostic APIs where possible

---

## 4. Implementation Specification

### 4.1 Dependency Configuration

#### 4.1.1 Root Workspace Cargo.toml
Add common audio dependencies to `[workspace.dependencies]`:

```toml
[workspace.dependencies]
cpal = "0.15"
hound = "3.5"
anyhow = "1.0"
thiserror = "1.0"
```

#### 4.1.2 vtt-core/Cargo.toml
Add dependencies using workspace versions:

```toml
[dependencies]
cpal.workspace = true
hound.workspace = true
anyhow.workspace = true
thiserror.workspace = true

[dev-dependencies]
# Test-specific dependencies will be added as needed
```

### 4.2 Directory Structure

Create the following structure under `crates/vtt-core/src/`:

```
src/
├── lib.rs                 # Update to export audio module
├── audio/
│   ├── mod.rs             # Audio module root
│   ├── capture.rs         # Audio capture (stub for task 1.3)
│   ├── device.rs          # Device enumeration (stub for task 1.3)
│   └── error.rs           # Audio-specific error types
```

### 4.3 Module Implementation

#### 4.3.1 audio/mod.rs
```rust
//! Audio processing module
//!
//! Provides cross-platform audio capture using cpal (Cross-Platform Audio Library).
//!
//! ## Platform Support
//!
//! - **Linux**: Uses ALSA backend, compatible with PipeWire via ALSA emulation
//! - **macOS**: Uses CoreAudio
//! - **Windows**: Uses WASAPI
//!
//! ## PipeWire Compatibility
//!
//! PipeWire provides ALSA compatibility through `pipewire-alsa`. Ensure the following:
//! - `pipewire` and `pipewire-alsa` packages are installed
//! - PipeWire is running as the audio server
//! - ALSA devices are exposed through PipeWire
//!
//! ## Usage
//!
//! Audio capture functionality will be implemented in task vtt-mcp-csh.1.3.

pub mod capture;
pub mod device;
pub mod error;

pub use error::{AudioError, AudioResult};

/// Audio module version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_version() {
        assert_eq!(VERSION, "0.1.0");
    }
}
```

#### 4.3.2 audio/error.rs
```rust
//! Error types for audio processing

use thiserror::Error;

/// Result type for audio operations
pub type AudioResult<T> = Result<T, AudioError>;

/// Errors that can occur during audio processing
#[derive(Error, Debug)]
pub enum AudioError {
    /// Audio device error (enumeration, initialization, etc.)
    #[error("Audio device error: {0}")]
    DeviceError(String),

    /// Audio stream error (configuration, start/stop, etc.)
    #[error("Audio stream error: {0}")]
    StreamError(String),

    /// Audio capture error
    #[error("Audio capture error: {0}")]
    CaptureError(String),

    /// I/O error (file operations)
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Generic error
    #[error("Audio error: {0}")]
    Other(String),
}

impl From<cpal::DevicesError> for AudioError {
    fn from(err: cpal::DevicesError) -> Self {
        AudioError::DeviceError(err.to_string())
    }
}

impl From<cpal::BuildStreamError> for AudioError {
    fn from(err: cpal::BuildStreamError) -> Self {
        AudioError::StreamError(err.to_string())
    }
}

impl From<cpal::PlayStreamError> for AudioError {
    fn from(err: cpal::PlayStreamError) -> Self {
        AudioError::StreamError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AudioError::DeviceError("test error".to_string());
        assert_eq!(err.to_string(), "Audio device error: test error");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let audio_err: AudioError = io_err.into();
        assert!(matches!(audio_err, AudioError::IoError(_)));
    }
}
```

#### 4.3.3 audio/capture.rs (stub)
```rust
//! Audio capture functionality
//!
//! This module will be implemented in task vtt-mcp-csh.1.3.
//! It will provide real-time audio capture from system microphones.

use super::error::AudioResult;

/// Audio capture interface (to be implemented in task 1.3)
pub struct AudioCapture {
    _private: (),
}

impl AudioCapture {
    /// Create a new audio capture instance
    ///
    /// # Errors
    ///
    /// Returns an error if audio device initialization fails.
    pub fn new() -> AudioResult<Self> {
        // Implementation in task vtt-mcp-csh.1.3
        Ok(Self { _private: () })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_capture_new() {
        let result = AudioCapture::new();
        assert!(result.is_ok());
    }
}
```

#### 4.3.4 audio/device.rs (stub)
```rust
//! Audio device enumeration and management
//!
//! This module will be implemented in task vtt-mcp-csh.1.3.
//! It will provide device discovery and selection.

use super::error::AudioResult;

/// Audio device information
#[derive(Debug, Clone)]
pub struct AudioDevice {
    /// Device name
    pub name: String,
    /// Whether this is the default device
    pub is_default: bool,
}

/// List available audio input devices
///
/// # Errors
///
/// Returns an error if device enumeration fails.
pub fn list_devices() -> AudioResult<Vec<AudioDevice>> {
    // Implementation in task vtt-mcp-csh.1.3
    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_devices() {
        let result = list_devices();
        assert!(result.is_ok());
    }
}
```

#### 4.3.5 Update lib.rs
Add audio module export:

```rust
//! Core voice-to-text functionality
//!
//! This crate provides the fundamental building blocks for voice-to-text
//! processing, including audio capture, voice activity detection, and
//! speech-to-text transcription.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod audio;

/// Core library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(VERSION, "0.1.0");
    }
}
```

### 4.4 PipeWire Research Notes

Add to project documentation (README or docs/):

```markdown
## PipeWire Compatibility

### Overview
PipeWire is a modern multimedia framework for Linux that provides:
- Low-latency audio processing
- Professional audio routing capabilities
- ALSA/PulseAudio compatibility layers

### cpal Integration
cpal uses the ALSA backend on Linux by default. PipeWire provides ALSA compatibility:

1. **Installation:**
   ```bash
   sudo pacman -S pipewire pipewire-alsa  # Arch Linux
   sudo apt install pipewire pipewire-alsa  # Debian/Ubuntu
   ```

2. **Verification:**
   ```bash
   pactl info | grep "Server Name"  # Should show PipeWire
   arecord -l  # Should list capture devices
   ```

3. **Configuration:**
   - PipeWire automatically exposes ALSA devices
   - No special cpal configuration required
   - Default device selection works out-of-the-box

### Fallback Options
If PipeWire is not available:
- ALSA (direct hardware access)
- PulseAudio (via pulseaudio-alsa)
- JACK (via jack-alsa)

### Testing
Test audio capture with:
```bash
arecord -d 5 -f S16_LE -r 16000 -c 1 test.wav
```
```

---

## 5. Acceptance Criteria

- [ ] **AC1:** Root Cargo.toml includes workspace dependencies (cpal, hound, anyhow, thiserror)
- [ ] **AC2:** vtt-core/Cargo.toml references workspace dependencies correctly
- [ ] **AC3:** `cargo build --workspace` succeeds without errors
- [ ] **AC4:** `crates/vtt-core/src/audio/` directory exists with all module files
- [ ] **AC5:** audio/mod.rs includes comprehensive documentation about PipeWire
- [ ] **AC6:** audio/error.rs defines AudioError and AudioResult types
- [ ] **AC7:** audio/capture.rs stub compiles and has placeholder implementation
- [ ] **AC8:** audio/device.rs stub compiles and has placeholder implementation
- [ ] **AC9:** lib.rs exports audio module
- [ ] **AC10:** `cargo test --workspace` passes all tests (including new audio module tests)
- [ ] **AC11:** `cargo clippy --all-targets --workspace -- -D warnings` produces no warnings
- [ ] **AC12:** `cargo doc --workspace --no-deps` generates documentation successfully
- [ ] **AC13:** Documentation includes PipeWire compatibility notes
- [ ] **AC14:** All changes committed to git with descriptive message

---

## 6. Quality Gates

Run the following commands to verify implementation:

```bash
# 1. Build
cargo build --workspace

# 2. Test
cargo test --workspace

# 3. Format check
cargo fmt --all -- --check

# 4. Clippy (strict)
cargo clippy --all-targets --workspace -- -D warnings

# 5. Documentation
cargo doc --workspace --no-deps --open

# 6. Dependency tree
cargo tree -p vtt-core
```

Expected results:
- ✅ All builds succeed without errors
- ✅ All tests pass (at least 6 tests: 3 existing + 3 new audio module tests)
- ✅ No clippy warnings
- ✅ Documentation builds successfully
- ✅ Dependency tree shows cpal, hound, anyhow, thiserror

---

## 7. Technical Constraints

### 7.1 Dependency Versions
- **cpal:** Use 0.15.x (latest stable at time of writing)
- **hound:** Use 3.5.x (stable WAV library)
- **anyhow:** Use 1.0.x (error handling)
- **thiserror:** Use 1.0.x (derive macros for errors)

### 7.2 Rust Features
- Must compile on Rust 1.70+ (edition 2021)
- No unsafe code in stubs
- All public APIs must have documentation

### 7.3 Platform Support
- Primary: Linux with PipeWire
- Secondary: macOS, Windows
- Use platform-agnostic cpal APIs

---

## 8. Integration Points

### 8.1 Dependencies
- **Task vtt-mcp-csh.1.1** (COMPLETE): Workspace structure provides foundation
- **PLAN.md Phase 1**: Audio capture pipeline setup

### 8.2 Dependents
- **Task vtt-mcp-csh.1.3**: Will use audio module for actual capture implementation
- **Task vtt-mcp-csh.1.4**: Will integrate VAD alongside audio capture

### 8.3 External
- System audio drivers (ALSA, PipeWire)
- Rust cpal ecosystem
- Future Whisper integration (will consume audio data)

---

## 9. Implementation Workflow

### Step 1: Update Root Cargo.toml
1. Open `Cargo.toml` in project root
2. Add workspace dependencies section with cpal, hound, anyhow, thiserror
3. Verify syntax

### Step 2: Update vtt-core Cargo.toml
1. Open `crates/vtt-core/Cargo.toml`
2. Add dependencies using workspace references
3. Run `cargo check -p vtt-core` to verify

### Step 3: Create Audio Module Structure
1. Create `crates/vtt-core/src/audio/` directory
2. Create mod.rs with module documentation
3. Create error.rs with error types
4. Create capture.rs stub
5. Create device.rs stub

### Step 4: Update lib.rs
1. Add `pub mod audio;` declaration
2. Update module-level documentation if needed

### Step 5: Quality Verification
1. Run `cargo build --workspace`
2. Run `cargo test --workspace`
3. Run `cargo clippy --all-targets --workspace -- -D warnings`
4. Run `cargo fmt --all -- --check`
5. Run `cargo doc --workspace --no-deps`

### Step 6: Documentation
1. Verify audio module documentation renders correctly
2. Check PipeWire notes are comprehensive
3. Add any missing inline comments

### Step 7: Commit Changes
1. Stage all changes: `git add .`
2. Commit: `git commit -m "task(csh.1.2): Configure audio dependencies (cpal)"`
3. Run `bd sync`

### Step 8: Task Closure
1. Close task: `bd close vtt-mcp-csh.1.2`
2. Add completion comment: `bd comment vtt-mcp-csh.1.2 -m "..."`
3. Final sync: `bd sync`

---

## 10. Success Metrics

- **Build Time:** Should remain under 60 seconds for clean build
- **Test Coverage:** All stub functions have basic tests
- **Documentation:** 100% of public APIs documented
- **Warnings:** Zero clippy warnings with `-D warnings` flag
- **Time to Complete:** Under 45 minutes (estimated)

---

## 11. Troubleshooting

### Issue: cpal version conflict
**Solution:** Use workspace dependency to ensure consistent version

### Issue: Build fails on missing features
**Solution:** Check cpal features, default should work for basic usage

### Issue: Documentation warnings
**Solution:** Add `#![warn(missing_docs)]` and ensure all pub items documented

### Issue: Circular dependency
**Solution:** Ensure audio module only depends on external crates, not other vtt-* crates

---

## 12. References

- **PLAN.md:** Phase 1 - Audio Capture Pipeline
- **AGENTS.md:** Project workflow and task management
- **Task vtt-mcp-csh.1.1:** Workspace structure (prerequisite)
- **Task vtt-mcp-csh.1.3:** Audio capture implementation (dependent)
- **cpal documentation:** https://docs.rs/cpal/
- **hound documentation:** https://docs.rs/hound/
- **PipeWire documentation:** https://pipewire.org/

---

## 13. Notes for Agent Implementation

### Context Management
- This is a small task (~45 min), single-session completion expected
- Can implement all files in one `execute_code` call
- Use text_editor to batch file creation

### Key Points
1. Stubs should compile and have basic tests
2. Error types should cover future use cases (capture, device, stream)
3. Documentation should be comprehensive (PipeWire notes important)
4. Follow existing patterns from task 1.1

### Validation Checklist
- [ ] All files created and contain correct content
- [ ] Build succeeds without warnings
- [ ] Tests pass (minimum 6 total)
- [ ] Clippy clean
- [ ] Git committed
- [ ] bd sync executed
- [ ] Task closed with comment

---

**Specification Version:** 1.0  
**Last Updated:** 2025-12-23 14:10  
**Reviewed By:** goose (agent)  
**Status:** Ready for Implementation
