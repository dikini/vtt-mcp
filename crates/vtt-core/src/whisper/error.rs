//! Error types for Whisper operations

use std::path::PathBuf;

/// Errors that can occur during Whisper operations
#[derive(Debug, thiserror::Error)]
pub enum WhisperError {
    /// Failed to load the model file
    #[error("Failed to load model from {path}: {reason}")]
    ModelLoadError {
        /// Path to the model file
        path: PathBuf,
        /// Reason for the failure
        reason: String,
    },

    /// Model file not found
    #[error("Model file not found: {0}")]
    ModelNotFound(PathBuf),

    /// Invalid audio data
    #[error("Invalid audio data: {0}")]
    InvalidAudio(String),

    /// Transcription failed
    #[error("Transcription failed: {0}")]
    TranscriptionFailed(String),

    /// Whisper context error
    #[error("Whisper context error: {0}")]
    ContextError(String),

    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

/// Type alias for Whisper operation results
pub type WhisperResult<T> = Result<T, WhisperError>;

/// Transcription result from Whisper
#[derive(Debug, Clone)]
pub struct Transcription {
    /// The transcribed text
    pub text: String,
    /// Start timestamp in milliseconds
    pub start_timestamp: i64,
    /// End timestamp in milliseconds
    pub end_timestamp: i64,
}

impl Transcription {
    pub fn new(text: String, start_timestamp: i64, end_timestamp: i64) -> Self {
        Self {
            text,
            start_timestamp,
            end_timestamp,
        }
    }

    pub fn duration_ms(&self) -> i64 {
        self.end_timestamp - self.start_timestamp
    }
}
