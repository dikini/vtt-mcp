//! Voice-to-Text Core Library
//!
//! This library provides core functionality for voice-to-text processing,
//! including audio capture, VAD (Voice Activity Detection), and Whisper
//! transcription integration.

pub mod audio;
pub mod vad;
pub mod whisper;
pub mod window;
pub mod incremental;
pub mod profile;

pub use audio::{AudioFormat, AudioError, AudioResult};
pub use vad::{VadDetector, VadConfig, VadResult};
pub use whisper::{WhisperContext, WhisperConfig, Transcription};
pub use window::{SlidingWindow, WindowConfig, WindowError};
pub use incremental::{IncrementalTranscriber, TranscriberConfig, PartialResult};
pub use profile::{Timer, ProfileData, Timing, TimingStats};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
