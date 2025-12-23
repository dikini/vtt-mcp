//! Audio format configuration

use cpal::{SampleFormat, SampleRate, SupportedStreamConfig};

/// Audio format specification for capture
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioFormat {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u16,
    /// Sample format
    pub sample_format: AudioSampleFormat,
}

/// Sample format types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioSampleFormat {
    /// 32-bit floating point
    F32,
    /// 16-bit signed integer
    I16,
}

impl AudioFormat {
    /// Default format: 48kHz stereo f32 (hardware-native for most devices)
    /// Note: 16kHz is common for speech-to-text but may require resampling
    pub const DEFAULT: Self = Self {
        sample_rate: 48000,
        channels: 2,
        sample_format: AudioSampleFormat::F32,
    };

    /// Speech-to-text optimized format: 16kHz mono f32
    /// This may require software resampling if hardware doesn't support it
    pub const STT_DEFAULT: Self = Self {
        sample_rate: 16000,
        channels: 1,
        sample_format: AudioSampleFormat::F32,
    };

    /// Create a new audio format
    pub fn new(sample_rate: u32, channels: u16, sample_format: AudioSampleFormat) -> Self {
        Self {
            sample_rate,
            channels,
            sample_format,
        }
    }

    /// Convert to cpal sample rate
    pub fn to_cpal_sample_rate(&self) -> SampleRate {
        SampleRate(self.sample_rate)
    }

    /// Get cpal sample format
    pub fn to_cpal_sample_format(&self) -> SampleFormat {
        match self.sample_format {
            AudioSampleFormat::F32 => SampleFormat::F32,
            AudioSampleFormat::I16 => SampleFormat::I16,
        }
    }

    /// Check if this format matches a supported config
    pub fn matches_config(&self, config: &SupportedStreamConfig) -> bool {
        config.sample_format() == self.to_cpal_sample_format()
            && config.channels() == self.channels
            && config.sample_rate() == self.to_cpal_sample_rate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_format() {
        let fmt = AudioFormat::DEFAULT;
        assert_eq!(fmt.sample_rate, 48000);
        assert_eq!(fmt.channels, 2);
        assert_eq!(fmt.sample_format, AudioSampleFormat::F32);
    }

    #[test]
    fn test_stt_format() {
        let fmt = AudioFormat::STT_DEFAULT;
        assert_eq!(fmt.sample_rate, 16000);
        assert_eq!(fmt.channels, 1);
        assert_eq!(fmt.sample_format, AudioSampleFormat::F32);
    }

    #[test]
    fn test_cpal_conversions() {
        let fmt = AudioFormat::DEFAULT;
        assert_eq!(fmt.to_cpal_sample_rate().0, 48000);
        assert_eq!(fmt.to_cpal_sample_format(), SampleFormat::F32);
    }
}
