//! Voice-to-Text Core Library
//!
//! Core functionality for:
//! - Audio capture
//! - VAD (Voice Activity Detection)
//! - Whisper transcription
//!
//! # Example Usage
//! ```
//! let example = 42;
//! assert_eq!(example, 42);
//! ```


pub mod audio;
pub mod vad;
pub mod config;
pub mod whisper;
pub mod window;
pub mod incremental;
pub mod profile;

pub use audio::{AudioFormat, AudioError, AudioResult};
pub use vad::{VadDetector, VadConfig, VadResult};
pub use whisper::{WhisperContext, WhisperConfig, WhisperError};
pub use window::{SlidingWindow, WindowConfig, WindowError};
pub use incremental::{IncrementalTranscriber, TranscriberConfig, PartialResult};
pub use profile::{Timer, ProfileData, Timing, TimingStats};

/// VTT-Core library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

