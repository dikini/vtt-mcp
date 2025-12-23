//! Whisper speech-to-text module
//!
//! This module provides speech-to-text functionality using OpenAI's Whisper model
//! via the whisper-rs bindings. It supports GPU acceleration via CUDA for fast
//! real-time transcription.

pub mod config;
pub mod context;
pub mod error;

pub use config::WhisperConfig;
pub use context::WhisperContext;
pub use error::{WhisperError, WhisperResult};

/// Result of a transcription operation
#[derive(Debug, Clone)]
pub struct Transcription {
    /// The transcribed text
    pub text: String,
    /// Confidence score (0.0 to 1.0) if available
    pub confidence: Option<f32>,
    /// Timestamp of the transcription start (in milliseconds)
    pub start_ms: i64,
    /// Timestamp of the transcription end (in milliseconds)
    pub end_ms: i64,
}

impl Transcription {
    /// Create a new transcription
    pub fn new(text: String, start_ms: i64, end_ms: i64) -> Self {
        Self {
            text,
            confidence: None,
            start_ms,
            end_ms,
        }
    }

    /// Create a new transcription with confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = Some(confidence.clamp(0.0, 1.0));
        self
    }
}
