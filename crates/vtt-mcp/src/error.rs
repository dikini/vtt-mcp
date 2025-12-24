//! Error types for VTT MCP server

use thiserror::Error;

/// VTT MCP server error type
#[derive(Error, Debug)]
/// VTT-MCP error types
///
/// Represents all errors that can occur in the VTT-MCP server.
pub enum VttError {
    /// Audio capture error
    #[error("Audio error: {0}")]
    Audio(#[from] vtt_core::audio::AudioError),

    /// Model loading/inference error
    #[error("Model error: {0}")]
    Model(String),

    /// Transcription error
    #[error("Transcription error: {0}")]
    Transcription(#[from] vtt_core::whisper::WhisperError),

    /// No audio data available
    #[error("No audio data: {0}")]
    NoAudioData(String),

    /// Invalid parameters
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    /// Device not found
    #[error("Device not found: {0}")]
    /// Requested audio device not found(String),

    /// Session error
    #[error("Session error: {0}")]
    Session(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Audio file error
    #[error("Audio file error: {0}")]
    AudioFile(#[from] hound::Error),
}

impl VttError {
    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self::InvalidParams(msg.into())
    }

    pub fn device_not_found(name: &str) -> Self {
        Self::DeviceNotFound(name.to_string())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

/// Result type for VTT operations
pub type VttResult<T> = Result<T, VttError>;

/// Trait to convert errors to MCP ErrorData
pub trait ToMcpError {
    fn into_mcp_error(self) -> rmcp::model::ErrorData;
    fn with_context(self, ctx: &str) -> rmcp::model::ErrorData;
}

impl ToMcpError for VttError {
    fn into_mcp_error(self) -> rmcp::model::ErrorData {
        rmcp::model::ErrorData {
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            message: self.to_string().into(),
            data: None,
        }
    }

    fn with_context(self, ctx: &str) -> rmcp::model::ErrorData {
        rmcp::model::ErrorData {
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            message: format!("{}: {}", ctx, self).into(),
            data: None,
        }
    }
}

// Implement From<VttError> for rmcp::ErrorData
impl From<VttError> for rmcp::model::ErrorData {
    fn from(err: VttError) -> Self {
        let code = match &err {
            VttError::InvalidParams(_) => rmcp::model::ErrorCode::INVALID_PARAMS,
            VttError::DeviceNotFound(_) => rmcp::model::ErrorCode::INVALID_REQUEST,
            _ => rmcp::model::ErrorCode::INTERNAL_ERROR,
        };

        rmcp::model::ErrorData {
            code,
            message: err.to_string().into(),
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let err = VttError::invalid_params("test error");
        let mcp_err: rmcp::model::ErrorData = err.into();
        assert_eq!(mcp_err.code, rmcp::model::ErrorCode::INVALID_PARAMS);
    }
}
