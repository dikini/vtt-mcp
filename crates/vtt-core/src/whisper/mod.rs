//! Whisper speech-to-text module
//!
//! This module provides speech-to-text functionality using OpenAI's Whisper model
//! via the whisper-rs bindings. It supports GPU acceleration via CUDA for fast
//! real-time transcription.

pub mod config;
pub mod context;
pub mod error;
pub mod gpu;
pub mod memory;

pub use config::WhisperConfig;
pub use context::WhisperContext;
pub use error::{WhisperError, WhisperResult};
pub use gpu::{GpuBackend, GpuDetection, GpuDeviceInfo, detect_gpu, get_gpu_info, is_gpu_available, get_gpu_message};
pub use memory::{MemoryStats, MemoryTracker};

/// Result of a transcription operation
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
    /// Create a new transcription result
    pub fn new(text: String, start_timestamp: i64, end_timestamp: i64) -> Self {
        Self {
            text,
            start_timestamp,
            end_timestamp,
        }
    }
    
    /// Get the duration of the transcription in milliseconds
    pub fn duration_ms(&self) -> i64 {
        self.end_timestamp - self.start_timestamp
    }
}

