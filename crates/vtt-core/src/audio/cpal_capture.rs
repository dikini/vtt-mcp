//! cpal-based audio capture
use super::error::{AudioError, AudioResult};
use super::format::AudioFormat;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use std::sync::{Arc, Mutex};

type AudioBuffer = Arc<Mutex<Vec<f32>>>;

pub struct CpalCapture {
    device: Device,
    format: AudioFormat,
    stream: Option<Stream>,
    buffer: AudioBuffer,
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
