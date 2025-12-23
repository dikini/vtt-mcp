//! Voice Activity Detection (VAD) module
//! 
//! This module provides speech detection functionality using an energy-based approach.
//! For production use, consider integrating Silero VAD or similar ML-based models.

pub mod detector;

pub use detector::VadDetector;

/// Configuration for VAD processing
#[derive(Debug, Clone, Copy)]
pub struct VadConfig {
    /// Energy threshold for speech detection (0.0 to 1.0)
    /// Default: 0.01 - values above this are considered potential speech
    pub energy_threshold: f32,
    
    /// Number of frames above threshold to trigger speech detection
    /// Default: 3 - helps reduce false positives from transient noise
    pub speech_frames_threshold: usize,
    
    /// Number of frames below threshold to trigger silence
    /// Default: 10 - helps prevent cutting off speech during brief pauses
    pub silence_frames_threshold: usize,
    
    /// Minimum duration for speech segment (in frames)
    /// Default: 30 (~500ms at 48kHz)
    pub min_speech_duration: usize,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            energy_threshold: 0.01,
            speech_frames_threshold: 3,
            silence_frames_threshold: 10,
            min_speech_duration: 30,
        }
    }
}

impl VadConfig {
    /// Create a new VAD config with custom energy threshold
    pub fn with_energy_threshold(threshold: f32) -> Self {
        Self {
            energy_threshold: threshold.clamp(0.0, 1.0),
            ..Default::default()
        }
    }
    
    /// Create a sensitive config (lower threshold, more detections)
    pub fn sensitive() -> Self {
        Self {
            energy_threshold: 0.005,
            speech_frames_threshold: 2,
            silence_frames_threshold: 15,
            ..Default::default()
        }
    }
    
    /// Create a strict config (higher threshold, fewer detections)
    pub fn strict() -> Self {
        Self {
            energy_threshold: 0.02,
            speech_frames_threshold: 5,
            silence_frames_threshold: 8,
            ..Default::default()
        }
    }
}

/// Result of VAD processing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VadResult {
    /// Speech detected
    Speech,
    /// Silence detected
    Silence,
    /// Not enough data to determine
    Unknown,
}

/// Error type for VAD operations
#[derive(Debug, thiserror::Error)]
pub enum VadError {
    #[error("Invalid audio buffer: {0}")]
    InvalidBuffer(String),
}

/// Type alias for speech energy level (0.0 to 1.0)
pub type SpeechEnergy = f32;
