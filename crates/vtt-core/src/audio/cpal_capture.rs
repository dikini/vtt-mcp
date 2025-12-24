//! cpal-based audio capture
use super::error::{AudioError, AudioResult};
use super::format::AudioFormat;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use std::sync::{Arc, Mutex};

type AudioBuffer = Arc<Mutex<Vec<f32>>>;

/// cpal audio capture implementation
///
/// Provides cross-platform audio capture using the cpal library.
/// Clone creates a new capture instance that shares the same buffer
/// but without the active stream (if any).
#[derive(Debug)]
pub struct CpalCapture {
    device: Device,
    format: AudioFormat,
    stream: Option<Stream>,
    buffer: AudioBuffer,
}

// Manual Clone implementation - shares buffer but not stream
impl Clone for CpalCapture {
    fn clone(&self) -> Self {
        Self {
            device: self.device.clone(),
            format: self.format.clone(),
            stream: None, // Don't clone the stream
            buffer: Arc::clone(&self.buffer),
        }
    }
}

impl CpalCapture {
    pub fn new() -> AudioResult<Self> {
        let device = super::device::default_device()?;
        Self::with_format(AudioFormat::DEFAULT)
    }

    pub fn with_format(format: AudioFormat) -> AudioResult<Self> {
        let device = super::device::default_device()?;
        Ok(Self { device, format, stream: None, buffer: Arc::new(Mutex::new(Vec::new())) })
    }

    pub fn start(&mut self) -> AudioResult<()> {
        if self.stream.is_some() {
            return Err(AudioError::StreamError("active".to_string()));
        }
        let config = self.get_supported_config()?;
        self.buffer.lock().unwrap().clear();
        let buffer = Arc::clone(&self.buffer);
        let stream = self.device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                buffer.lock().unwrap().extend_from_slice(data);
            },
            |err| eprintln!("error: {}", err),
            None,
        )?;
        stream.play()?;
        self.stream = Some(stream);
        Ok(())
    }

    pub fn stop(&mut self) -> AudioResult<()> {
        self.stream = None;
        Ok(())
    }

    pub fn take_buffer(&mut self) -> Vec<f32> {
        let mut buffer = self.buffer.lock().unwrap();
        std::mem::take(&mut *buffer)
    }

    pub fn buffer_len(&self) -> usize {
        self.buffer.lock().unwrap().len()
    }

    pub fn is_active(&self) -> bool {
        self.stream.is_some()
    }

    pub fn format(&self) -> &AudioFormat {
        &self.format
    }

    fn get_supported_config(&self) -> AudioResult<StreamConfig> {
        let supported_configs = self.device.supported_input_configs()
            .map_err(|e| AudioError::DeviceError(e.to_string()))?;
        let desired_sample_rate = self.format.to_cpal_sample_rate();
        for config_range in supported_configs {
            if config_range.sample_format() == self.format.to_cpal_sample_format()
                && config_range.channels() == self.format.channels
                && config_range.min_sample_rate() <= desired_sample_rate
                && config_range.max_sample_rate() >= desired_sample_rate
            {
                return Ok(StreamConfig {
                    channels: config_range.channels(),
                    sample_rate: desired_sample_rate,
                    buffer_size: cpal::BufferSize::Default,
                });
            }
        }
        Ok(StreamConfig {
            channels: self.format.channels,
            sample_rate: self.format.to_cpal_sample_rate(),
            buffer_size: cpal::BufferSize::Default,
        })
    }
}

impl Drop for CpalCapture {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpal_capture_with_format() {
        let format = AudioFormat::DEFAULT;
        let result = CpalCapture::with_format(format);
        // May fail in CI without audio devices
        let _ = result;
    }

    #[test]
    fn test_cpal_capture_clone() {
        let format = AudioFormat::DEFAULT;
        let capture1 = match CpalCapture::with_format(format) {
            Ok(c) => c,
            Err(_) => return, // Skip if no device
        };
        
        let capture2 = capture1.clone();
        
        // Clones should share buffer but not stream
        assert_eq!(capture1.format().sample_rate, capture2.format().sample_rate);
        assert!(!capture2.is_active());
    }

    #[test]
    fn test_cpal_buffer_operations() {
        let format = AudioFormat::DEFAULT;
        let mut capture = match CpalCapture::with_format(format) {
            Ok(c) => c,
            Err(_) => return, // Skip if no device
        };
        
        // Initially empty
        assert_eq!(capture.buffer_len(), 0);
        
        // Take buffer should return empty
        let buffer = capture.take_buffer();
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_cpal_format_access() {
        let format = AudioFormat::DEFAULT;
        let capture = match CpalCapture::with_format(format) {
            Ok(c) => c,
            Err(_) => return, // Skip if no device
        };
        
        let fmt = capture.format();
        assert_eq!(fmt.sample_rate, 16000);
        assert_eq!(fmt.channels, 1);
    }
}
