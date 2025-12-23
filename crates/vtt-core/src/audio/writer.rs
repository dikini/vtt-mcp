//! WAV file writing functionality

use super::error::{AudioError, AudioResult};
use super::format::AudioFormat;
use hound::{WavSpec, WavWriter};
use std::path::Path;

/// Write f32 audio samples to WAV file
///
/// Converts f32 samples in range [-1.0, 1.0] to 16-bit PCM.
///
/// # Arguments
///
/// * `path` - Output file path
/// * `samples` - Audio samples as f32
/// * `format` - Audio format specification
///
/// # Errors
///
/// Returns an error if:
/// - File cannot be created
/// - Write operation fails
pub fn write_wav<P: AsRef<Path>>(
    path: P,
    samples: &[f32],
    format: &AudioFormat,
) -> AudioResult<()> {
    let spec = WavSpec {
        channels: format.channels,
        sample_rate: format.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(path, spec)
        .map_err(|e| AudioError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    for &sample in samples {
        let sample_i16 = (sample * i16::MAX as f32) as i16;
        writer
            .write_sample(sample_i16)
            .map_err(|e| AudioError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    }

    writer
        .finalize()
        .map_err(|e| AudioError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_write_wav() {
        let samples: Vec<f32> = (0..16000).map(|i| (i as f32 * 0.001).sin()).collect();

        let format = AudioFormat::DEFAULT;
        let path = "/tmp/test_audio.wav";

        let result = write_wav(path, &samples, &format);
        assert!(result.is_ok());

        // Verify file exists
        assert!(Path::new(path).exists());

        // Clean up
        let _ = fs::remove_file(path);
    }
}
