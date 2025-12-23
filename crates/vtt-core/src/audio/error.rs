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
