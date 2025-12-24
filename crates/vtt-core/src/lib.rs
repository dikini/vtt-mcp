//! Voice-to-Text Core Library
//!
//! This library provides core functionality for voice-to-text processing,
//! including audio capture, VAD (Voice Activity Detection), and Whisper
//! transcription integration.
//!
//! # Modules
//!
//! - `audio`: Audio capture from various backends (PipeWire, cpal)
//! - `vad`: Voice Activity Detection for speech/silence detection
//! - `whisper`: Whisper model integration for transcription
//! - `window`: Sliding window buffer for audio samples
//! - `incremental`: Incremental transcription with duplicate suppression
//! - `profile`: Performance profiling utilities
//! - `config`: Configuration management
//!
//! # Example
//!
//! ```rust
//! use vtt_core::{WhisperContext, AudioFormat};
//!
//! // Load Whisper model
//! let whisper = WhisperContext::new("models/ggml-base.bin")?;
//!
//! // Transcribe audio
//! let audio: Vec<f32> = /* ... */;
//! let result = whisper.transcribe(&audio, 16000)?;
//! println!("{}", result.text);
//! # Ok::<(), Box<dyn std::error::Error>>(())
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
