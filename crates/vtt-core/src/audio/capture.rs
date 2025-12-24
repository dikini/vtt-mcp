//! Audio capture - platform abstraction

#[cfg(target_os = "linux")]
use super::pipewire_capture::PipeWireCapture as Impl;

#[cfg(not(target_os = "linux"))]
use super::cpal_capture::CpalCapture as Impl;

use super::{AudioFormat, AudioResult};

/// Audio capture device abstraction
/// 
/// This struct provides a cross-platform interface for capturing audio
/// from the default input device (microphone).
#[derive(Debug, Clone)]
pub struct AudioCapture {
    inner: Impl,
}

impl AudioCapture {
    /// Create a new audio capture instance with default format
    pub fn new() -> AudioResult<Self> {
        Ok(Self {
            inner: Impl::new()?,
        })
    }
    
    /// Create a new audio capture instance with custom format
    pub fn with_format(fmt: AudioFormat) -> AudioResult<Self> {
        Ok(Self {
            inner: Impl::with_format(fmt)?,
        })
    }
    
    /// Start capturing audio
    pub fn start(&mut self) -> AudioResult<()> {
        self.inner.start()
    }
    
    /// Stop capturing audio
    pub fn stop(&mut self) -> AudioResult<()> {
        self.inner.stop()
    }
    
    /// Take the captured audio buffer
    /// 
    /// This consumes the buffer and returns it as a Vec<f32>.
    /// After calling this method, the internal buffer is empty.
    pub fn take_buffer(&mut self) -> Vec<f32> {
        self.inner.take_buffer()
    }
    
    /// Get the length of the captured audio buffer (in samples)
    pub fn buffer_len(&self) -> usize {
        self.inner.buffer_len()
    }
    
    /// Check if the capture is currently active
    pub fn is_active(&self) -> bool {
        self.inner.is_active()
    }
    
    /// Get the audio format being used
    pub fn format(&self) -> &AudioFormat {
        self.inner.format()
    }
}
