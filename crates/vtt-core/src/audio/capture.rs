//! Audio capture - platform abstraction

#[cfg(target_os = "linux")]
use super::pipewire_capture::PipeWireCapture as Impl;

#[cfg(not(target_os = "linux"))]
use super::cpal_capture::CpalCapture as Impl;

use super::{AudioResult, AudioFormat};

pub struct AudioCapture { inner: Impl }

impl AudioCapture {
    pub fn new() -> AudioResult<Self> { Ok(Self { inner: Impl::new()? }) }
    pub fn with_format(fmt: AudioFormat) -> AudioResult<Self> {
        Ok(Self { inner: Impl::with_format(fmt)? })
    }
    pub fn start(&mut self) -> AudioResult<()> { self.inner.start() }
    pub fn stop(&mut self) -> AudioResult<()> { self.inner.stop() }
    pub fn take_buffer(&mut self) -> Vec<f32> { self.inner.take_buffer() }
    pub fn buffer_len(&self) -> usize { self.inner.buffer_len() }
    pub fn is_active(&self) -> bool { self.inner.is_active() }
    pub fn format(&self) -> &AudioFormat { self.inner.format() }
}
