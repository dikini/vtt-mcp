//! Core voice-to-text functionality
//!
//! This crate provides the fundamental building blocks for voice-to-text
//! processing, including audio capture, voice activity detection, and
//! speech-to-text transcription.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod audio;
pub mod vad;

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
