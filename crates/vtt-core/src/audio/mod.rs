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
