//! Sliding window buffer for audio samples
//!
//! Provides a circular buffer that maintains a configurable duration
//! of audio samples, useful for incremental transcription.

use std::sync::Arc;
use tokio::sync::Mutex;
use crate::profile::Timer;

/// Errors that can occur in window operations
#[derive(Debug, thiserror::Error)]
pub enum WindowError {
    /// Buffer overflow occurred, some samples were dropped
    #[error("Buffer overflow: {dropped} samples dropped")]
    BufferOverflow { dropped: usize },
}

pub type WindowResult<T> = Result<T, WindowError>;

/// Configuration for the sliding window
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// Duration of the window in seconds
    pub duration_secs: f32,
    /// Sample rate in Hz
    pub sample_rate: u32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            duration_secs: 3.0,
            sample_rate: 16000,
        }
    }
}

impl WindowConfig {
    /// Create a new window configuration
    pub fn new(duration_secs: f32, sample_rate: u32) -> Self {
        Self {
            duration_secs,
            sample_rate,
        }
    }
    
    /// Calculate buffer capacity from configuration
    pub fn buffer_capacity(&self) -> usize {
        (self.duration_secs * self.sample_rate as f32) as usize
    }
}

/// Internal state of the sliding window
struct WindowInner {
    /// The circular buffer holding audio samples
    buffer: Vec<f32>,
    /// Current write position in the buffer
    write_pos: usize,
    /// Number of valid samples in the buffer
    len: usize,
    /// Total capacity of the buffer
    capacity: usize,
}

impl WindowInner {
    /// Create a new window inner state
    fn new(config: &WindowConfig) -> Self {
        let capacity = config.buffer_capacity();
        Self {
            buffer: vec![0.0; capacity],
            write_pos: 0,
            len: 0,
            capacity,
        }
    }
}

/// A sliding window buffer for audio samples
///
/// Maintains a circular buffer of audio samples with a configurable duration.
/// Thread-safe and suitable for use from async contexts.
#[derive(Clone)]
pub struct SlidingWindow {
    /// Internal window state
    inner: Arc<Mutex<WindowInner>>,
    /// Window configuration
    config: WindowConfig,
}

impl SlidingWindow {
    /// Create a new sliding window with default configuration
    pub fn new() -> Self {
        Self::with_config(WindowConfig::default())
    }
    
    /// Create a new sliding window with specific configuration
    pub fn with_config(config: WindowConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(WindowInner::new(&config))),
            config,
        }
    }
    
    /// Push audio samples into the window
    ///
    /// # Returns
    /// - Ok(0) if all samples were added successfully
    /// - Ok(dropped) if the buffer overflowed and some samples were dropped
    pub async fn push(&self, samples: &[f32]) -> WindowResult<usize> {
        let _timer = Timer::start("window_push");
        let mut inner = self.inner.lock().await;
        
        let samples_len = samples.len();
        let capacity = inner.capacity;
        
        // Check for overflow
        let new_len = inner.len + samples_len;
        let dropped = if new_len > capacity {
            new_len - capacity
        } else {
            0
        };
        
        // Copy samples into circular buffer
        for &sample in samples.iter() {
            let pos = inner.write_pos;
            inner.buffer[pos] = sample;
            inner.write_pos = (inner.write_pos + 1) % capacity;
        }
        
        // Update length, clamping to capacity
        inner.len = (inner.len + samples_len).min(capacity);
        
        if dropped > 0 {
            Err(WindowError::BufferOverflow { dropped })
        } else {
            Ok(0)
        }
    }
    
    /// Get the last N seconds of audio from the window
    ///
    /// # Arguments
    /// * `seconds` - Number of seconds to retrieve (must be <= config.duration_secs)
    pub async fn get_last_seconds(&self, seconds: f32) -> WindowResult<Vec<f32>> {
        let _timer = Timer::start("window_get");
        let inner = self.inner.lock().await;
        
        if seconds > self.config.duration_secs {
            return Err(WindowError::BufferOverflow { dropped: 0 });
        }
        
        let sample_count = (seconds * self.config.sample_rate as f32) as usize;
        self.get_last_n(sample_count, &inner)
    }
    
    /// Get all audio currently in the window
    pub async fn get_all(&self) -> Vec<f32> {
        let inner = self.inner.lock().await;
        let len = inner.len;
        
        if len == 0 {
            return Vec::new();
        }
        
        let mut result = Vec::with_capacity(len);
        let read_pos = if inner.len == inner.capacity {
            inner.write_pos
        } else {
            0
        };
        
        for i in 0..len {
            result.push(inner.buffer[(read_pos + i) % inner.capacity]);
        }
        
        result
    }
    
    /// Get the last N samples from the window
    fn get_last_n(&self, n: usize, inner: &WindowInner) -> WindowResult<Vec<f32>> {
        let available = inner.len.min(n);
        
        if available == 0 {
            return Ok(Vec::new());
        }
        
        let mut result = Vec::with_capacity(available);
        
        // Calculate read position
        let read_pos = if inner.len == inner.capacity {
            inner.write_pos
        } else {
            0
        };
        
        // Read the last 'available' samples
        let start_offset = inner.len - available;
        for i in 0..available {
            let pos = (read_pos + start_offset + i) % inner.capacity;
            result.push(inner.buffer[pos]);
        }
        
        Ok(result)
    }
    
    /// Get the current duration of audio in the window (in seconds)
    pub async fn duration_secs(&self) -> f32 {
        let inner = self.inner.lock().await;
        inner.len as f32 / self.config.sample_rate as f32
    }
    
    /// Clear all samples from the window
    pub async fn clear(&self) {
        let mut inner = self.inner.lock().await;
        inner.len = 0;
        inner.write_pos = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_basic_push_and_get() {
        let window = SlidingWindow::with_config(WindowConfig::new(1.0, 100));
        
        // Push 100 samples (1 second at 100Hz)
        let samples: Vec<f32> = (0..100).map(|i| i as f32).collect();
        let result = window.push(&samples).await;
        assert!(result.is_ok());
        
        // Get last 0.5 seconds
        let last = window.get_last_seconds(0.5).await.unwrap();
        assert_eq!(last.len(), 50);
        assert_eq!(last[0], 50.0);
        assert_eq!(last[49], 99.0);
    }
    
    #[tokio::test]
    async fn test_circular_overflow() {
        let window = SlidingWindow::with_config(WindowConfig::new(1.0, 100));
        
        // Push 150 samples (1.5 seconds - should overflow by 50)
        let samples: Vec<f32> = (0..150).map(|i| i as f32).collect();
        let result = window.push(&samples).await;
        assert!(matches!(result, Err(WindowError::BufferOverflow { dropped: 50 })));
        
        // Should only have the last 100 samples
        let last = window.get_last_seconds(1.0).await.unwrap();
        assert_eq!(last.len(), 100);
        assert_eq!(last[0], 50.0);
        assert_eq!(last[99], 149.0);
    }
    
    #[tokio::test]
    async fn test_get_last_n() {
        let window = SlidingWindow::with_config(WindowConfig::new(1.0, 100));
        
        let samples: Vec<f32> = (0..100).map(|i| i as f32).collect();
        window.push(&samples).await.unwrap();
        
        // Request more than available
        let result = window.get_last_seconds(2.0).await;
        assert!(matches!(result, Err(WindowError::BufferOverflow { .. })));
    }
    
    #[tokio::test]
    async fn test_duration() {
        let window = SlidingWindow::with_config(WindowConfig::new(2.0, 100));
        
        assert_eq!(window.duration_secs().await, 0.0);
        
        let samples: Vec<f32> = (0..100).map(|_| 1.0).collect();
        window.push(&samples).await.unwrap();
        
        assert_eq!(window.duration_secs().await, 1.0);
        
        let more: Vec<f32> = (0..50).map(|_| 1.0).collect();
        window.push(&more).await.unwrap();
        
        assert_eq!(window.duration_secs().await, 1.5);
    }
}
