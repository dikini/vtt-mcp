//! Voice Activity Detection using energy-based approach
//!
//! This is a simple but effective VAD based on signal energy.
//! For better accuracy, consider integrating ML-based models like Silero VAD.

use super::{VadConfig, VadError, VadResult};

/// Voice Activity Detector (energy-based)
pub struct VadDetector {
    config: VadConfig,
    speech_frame_count: usize,
    silence_frame_count: usize,
    current_state: VadResult,
    speech_segment_length: usize,
}

impl VadDetector {
    /// Create a new VAD detector
    pub fn new() -> Self {
        Self {
            config: VadConfig::default(),
            speech_frame_count: 0,
            silence_frame_count: 0,
            current_state: VadResult::Silence,
            speech_segment_length: 0,
        }
    }

    /// Create a new VAD detector with custom config
    pub fn with_config(config: VadConfig) -> Self {
        Self {
            config,
            speech_frame_count: 0,
            silence_frame_count: 0,
            current_state: VadResult::Silence,
            speech_segment_length: 0,
        }
    }

    /// Initialize the VAD (no-op for energy-based VAD)
    pub fn init(&mut self) -> Result<(), VadError> {
        // Energy-based VAD doesn't need initialization
        Ok(())
    }

    /// Check if the model is initialized (always true for energy-based)
    pub fn is_initialized(&self) -> bool {
        true
    }

    /// Process an audio frame and detect speech
    /// 
    /// # Arguments
    /// * `audio` - Audio samples (f32, normalized to [-1.0, 1.0])
    /// 
    /// # Returns
    /// * `Ok(VadResult)` - Speech detection result
    /// * `Err(VadError)` - If buffer is invalid
    pub fn process_frame(&mut self, audio: &[f32]) -> Result<VadResult, VadError> {
        if audio.is_empty() {
            return Ok(VadResult::Unknown);
        }

        // Calculate RMS energy of the frame
        let energy = calculate_rms_energy(audio);
        
        // Determine if this frame has speech energy
        let has_speech_energy = energy >= self.config.energy_threshold;

        // State machine for speech detection
        let new_state = if has_speech_energy {
            self.speech_frame_count += 1;
            self.silence_frame_count = 0;
            
            if self.speech_frame_count >= self.config.speech_frames_threshold {
                VadResult::Speech
            } else {
                self.current_state // Still in debounce period
            }
        } else {
            self.silence_frame_count += 1;
            
            if self.silence_frame_count >= self.config.silence_frames_threshold {
                self.speech_frame_count = 0;
                self.speech_segment_length = 0;
                VadResult::Silence
            } else {
                self.current_state // Still in hangover period
            }
        };

        // Update segment length if we're in speech
        if new_state == VadResult::Speech {
            self.speech_segment_length += 1;
        }

        self.current_state = new_state;
        Ok(self.current_state)
    }

    /// Get the current energy threshold
    pub fn threshold(&self) -> f32 {
        self.config.energy_threshold
    }

    /// Update the energy threshold
    pub fn set_threshold(&mut self, threshold: f32) {
        self.config.energy_threshold = threshold.clamp(0.0, 1.0);
    }

    /// Get the current state
    pub fn current_state(&self) -> VadResult {
        self.current_state
    }

    /// Reset the detector state
    pub fn reset(&mut self) {
        self.speech_frame_count = 0;
        self.silence_frame_count = 0;
        self.current_state = VadResult::Silence;
        self.speech_segment_length = 0;
    }
}

impl Default for VadDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate RMS energy of an audio buffer
fn calculate_rms_energy(audio: &[f32]) -> f32 {
    if audio.is_empty() {
        return 0.0;
    }

    let sum_squares: f32 = audio.iter().map(|&x| x * x).sum();
    let mean_square = sum_squares / audio.len() as f32;
    mean_square.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = VadConfig::default();
        assert_eq!(config.energy_threshold, 0.01);
        assert_eq!(config.speech_frames_threshold, 3);
    }

    #[test]
    fn test_config_sensitive() {
        let config = VadConfig::sensitive();
        assert_eq!(config.energy_threshold, 0.005);
        assert_eq!(config.speech_frames_threshold, 2);
    }

    #[test]
    fn test_config_strict() {
        let config = VadConfig::strict();
        assert_eq!(config.energy_threshold, 0.02);
        assert_eq!(config.speech_frames_threshold, 5);
    }

    #[test]
    fn test_vad_detector_creation() {
        let detector = VadDetector::new();
        assert!(detector.is_initialized());
        assert_eq!(detector.threshold(), 0.01);
        assert_eq!(detector.current_state(), VadResult::Silence);
    }

    #[test]
    fn test_process_silence() {
        let mut detector = VadDetector::new();
        // Process silence (zeros)
        for _ in 0..10 {
            let result = detector.process_frame(&[0.0; 512]).unwrap();
            assert_eq!(result, VadResult::Silence);
        }
    }

    #[test]
    fn test_process_speech() {
        let mut detector = VadDetector::new();
        // Process speech-like audio (need enough frames to trigger)
        for i in 0..10 {
            // Generate a sine wave pattern
            let audio: Vec<f32> = (0..512)
                .map(|j| (0.1 * ((i * 512 + j) as f32 * 0.1).sin() as f32))
                .collect();
            let result = detector.process_frame(&audio).unwrap();
            // Should eventually detect speech after threshold frames
            if i >= 3 {
                assert_eq!(result, VadResult::Speech);
            }
        }
    }

    #[test]
    fn test_empty_buffer() {
        let mut detector = VadDetector::new();
        let result = detector.process_frame(&[]);
        assert!(matches!(result, Ok(VadResult::Unknown)));
    }

    #[test]
    fn test_reset() {
        let mut detector = VadDetector::new();
        // Trigger speech detection
        for _ in 0..10 {
            let audio: Vec<f32> = (0..512).map(|_| 0.1).collect();
            detector.process_frame(&audio).unwrap();
        }
        assert_eq!(detector.current_state(), VadResult::Speech);
        
        // Reset
        detector.reset();
        assert_eq!(detector.current_state(), VadResult::Silence);
        assert_eq!(detector.speech_frame_count, 0);
    }

    #[test]
    fn test_rms_energy_calculation() {
        // Silence
        assert_eq!(calculate_rms_energy(&[0.0; 100]), 0.0);
        
        // Constant amplitude
        let signal: Vec<f32> = vec![0.5; 100];
        let energy = calculate_rms_energy(&signal);
        assert!((energy - 0.5).abs() < 0.01);
    }
}
