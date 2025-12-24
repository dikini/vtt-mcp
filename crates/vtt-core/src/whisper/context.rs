//! Whisper context and transcription

use crate::whisper::{Transcription, WhisperConfig, WhisperError, WhisperResult};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Main context for Whisper speech-to-text
///
/// This struct manages the Whisper model and provides transcription functionality.
/// It maintains internal state and ensures thread-safe access to the model.
pub struct WhisperContext {
    /// The underlying whisper-rs context
    context: Arc<Mutex<whisper_rs::WhisperContext>>,
    /// Configuration used to create this context
    config: WhisperConfig,
}

impl WhisperContext {
    /// Create a new Whisper context from a model file
    ///
    /// # Arguments
    /// * `config` - Configuration for the Whisper context
    ///
    /// # Errors
    /// Returns an error if the model file cannot be loaded
    pub fn new(config: WhisperConfig) -> WhisperResult<Self> {
        let model_path = Path::new(&config.model_path);

        if !model_path.exists() {
            return Err(WhisperError::ModelNotFound(model_path.to_path_buf()));
        }

        // Load the model using whisper-rs
        let params = whisper_rs::WhisperContextParameters {
            use_gpu: config.use_gpu,
            ..Default::default()
        };

        let ctx = whisper_rs::WhisperContext::new_with_params(&config.model_path, params).map_err(
            |e| WhisperError::ModelLoadError {
                path: model_path.to_path_buf(),
                reason: e.to_string(),
            },
        )?;

        Ok(Self {
            context: Arc::new(Mutex::new(ctx)),
            config,
        })
    }

    /// Create a new Whisper context with default configuration
    ///
    /// This uses the default model path (models/ggml-base.bin) and settings.
    pub fn with_default_model() -> WhisperResult<Self> {
        Self::new(WhisperConfig::default())
    }

    /// Transcribe audio data
    ///
    /// # Arguments
    /// * `audio_data` - Audio samples as f32 values (normalized to [-1.0, 1.0])
    /// * `sample_rate` - Sample rate of the audio data (will be resampled to 16kHz if needed)
    pub fn transcribe(&self, audio_data: &[f32], sample_rate: u32) -> WhisperResult<Transcription> {
        if audio_data.is_empty() {
            return Err(WhisperError::InvalidAudio(
                "Audio data is empty".to_string(),
            ));
        }

        if audio_data.len() < 100 {
            return Err(WhisperError::InvalidAudio(
                "Audio data too short".to_string(),
            ));
        }

        // Check for NaN or infinite values
        if audio_data.iter().any(|&v| !v.is_finite()) {
            return Err(WhisperError::InvalidAudio(
                "Audio data contains NaN or infinite values".to_string(),
            ));
        }

        // Resample to 16kHz if necessary
        let processed_audio = if sample_rate != self.config.required_sample_rate {
            self.resample_audio(audio_data, sample_rate, self.config.required_sample_rate)?
        } else {
            audio_data.to_vec()
        };

        // Get the context and create a state
        let ctx = self
            .context
            .lock()
            .map_err(|e| WhisperError::ContextError(format!("Failed to lock context: {}", e)))?;

        let mut state = ctx.create_state().map_err(|e| {
            WhisperError::TranscriptionFailed(format!("Failed to create state: {}", e))
        })?;

        // Create a full params struct
        let mut params =
            whisper_rs::FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 0 });

        // Set parameters from config
        params.set_n_threads(self.config.n_threads);
        params.set_translate(false);
        params.set_language(None);
        params.set_offset_ms(0);
        params.set_duration_ms(0);
        params.set_token_timestamps(true);
        params.set_single_segment(false);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        // Process the audio
        state.full(params, &processed_audio).map_err(|e| {
            WhisperError::TranscriptionFailed(format!("Full inference failed: {}", e))
        })?;

        // Extract the transcription text using the state iterator
        let mut text = String::new();
        let mut start_timestamp = 0_i64;
        let mut end_timestamp = 0_i64;

        for segment in state.as_iter() {
            let segment_text = segment.to_string();
            let segment_start = segment.start_timestamp();
            let segment_end = segment.end_timestamp();

            if text.is_empty() {
                start_timestamp = segment_start as i64;
            }
            end_timestamp = segment_end as i64;

            if !text.is_empty() && !segment_text.trim().is_empty() {
                text.push(' ');
            }
            text.push_str(segment_text.trim());
        }

        Ok(Transcription::new(
            text.trim().to_string(),
            start_timestamp,
            end_timestamp,
        ))
    }

    /// Resample audio data from one sample rate to another
    fn resample_audio(
        &self,
        audio_data: &[f32],
        from_sample_rate: u32,
        to_sample_rate: u32,
    ) -> WhisperResult<Vec<f32>> {
        if from_sample_rate == to_sample_rate {
            return Ok(audio_data.to_vec());
        }

        let ratio = from_sample_rate as f32 / to_sample_rate as f32;
        let output_length = ((audio_data.len() as f32) / ratio).ceil() as usize;

        let mut resampled = vec![0.0_f32; output_length];

        for i in 0..output_length {
            let src_pos = (i as f32) * ratio;
            let src_idx = src_pos.floor() as usize;
            let frac = src_pos - src_pos.floor();

            if src_idx + 1 < audio_data.len() {
                let sample1 = audio_data[src_idx];
                let sample2 = audio_data[src_idx + 1];
                resampled[i] = sample1 * (1.0 - frac) + sample2 * frac;
            } else if src_idx < audio_data.len() {
                resampled[i] = audio_data[src_idx];
            }
        }

        Ok(resampled)
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &WhisperConfig {
        &self.config
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcription_new() {
        let t = Transcription::new("Hello world".to_string(), 0, 1000);
        assert_eq!(t.text, "Hello world");
        assert_eq!(t.start_timestamp, 0);
        assert_eq!(t.end_timestamp, 1000);
        assert_eq!(t.duration_ms(), 1000);
    }
}
