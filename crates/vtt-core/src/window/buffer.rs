//! Sliding window buffer implementation

use std::sync::Arc;
use tokio::sync::Mutex;

/// Error type for sliding window operations
#[derive(Debug, thiserror::Error)]
pub enum WindowError {
    #[error("Buffer overflow: dropping {dropped} oldest samples")]
    BufferOverflow { dropped: usize },
}

pub type WindowResult<T> = Result<T, WindowError>;

/// Configuration for the sliding window buffer
#[derive(Debug, Clone, Copy)]
pub struct WindowConfig {
    pub duration_secs: f32,
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
    pub fn with_duration_secs(duration_secs: f32) -> Self {
        Self {
            duration_secs: duration_secs.max(0.5),
            ..Default::default()
        }
    }

    pub fn capacity_samples(&self) -> usize {
        (self.duration_secs * self.sample_rate as f32) as usize
    }
}

pub struct SlidingWindow {
    inner: Arc<Mutex<WindowInner>>,
    config: WindowConfig,
}

struct WindowInner {
    buffer: Vec<f32>,
    head: usize,
    len: usize,
    dropped_samples: usize,
}

impl SlidingWindow {
    pub fn new(config: WindowConfig) -> Self {
        let capacity = config.capacity_samples();
        assert!(capacity > 0);

        Self {
            inner: Arc::new(Mutex::new(WindowInner {
                buffer: vec![0.0; capacity],
                head: 0,
                len: 0,
                dropped_samples: 0,
            })),
            config,
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(WindowConfig::default())
    }

    pub async fn push(&self, samples: &[f32]) -> WindowResult<usize> {
        let mut inner = self.inner.lock().await;
        let capacity = inner.buffer.len();

        if samples.is_empty() {
            return Ok(0);
        }

        let dropped = if inner.len + samples.len() > capacity {
            let overflow = (inner.len + samples.len()) - capacity;
            inner.len = capacity;
            inner.dropped_samples += overflow;
            overflow
        } else {
            inner.len += samples.len();
            0
        };

        for &sample in samples {
            let idx = inner.head;
            inner.buffer[idx] = sample;
            inner.head = (inner.head + 1) % capacity;
        }

        Ok(dropped)
    }

    pub async fn get_all(&self) -> Vec<f32> {
        let inner = self.inner.lock().await;
        inner.get_all()
    }

    pub async fn get_last_seconds(&self, seconds: f32) -> WindowResult<Vec<f32>> {
        let requested_samples = (seconds * self.config.sample_rate as f32) as usize;
        self.get_last_n(requested_samples).await
    }

    pub async fn get_last_n(&self, n: usize) -> WindowResult<Vec<f32>> {
        let inner = self.inner.lock().await;

        if n > inner.len {
            return Ok(inner.get_all());
        }

        Ok(inner.get_last_n(n))
    }

    pub async fn len(&self) -> usize {
        let inner = self.inner.lock().await;
        inner.len
    }

    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }

    pub async fn dropped_samples(&self) -> usize {
        let inner = self.inner.lock().await;
        inner.dropped_samples
    }

    pub async fn clear(&self) {
        let mut inner = self.inner.lock().await;
        inner.head = 0;
        inner.len = 0;
    }
}

impl WindowInner {
    fn get_all(&self) -> Vec<f32> {
        if self.len == 0 {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(self.len);
        let capacity = self.buffer.len();

        if self.len < capacity {
            result.extend_from_slice(&self.buffer[0..self.len]);
        } else {
            result.extend_from_slice(&self.buffer[self.head..]);
            let remaining = self.len - (capacity - self.head);
            result.extend_from_slice(&self.buffer[..remaining]);
        }

        result
    }

    fn get_last_n(&self, n: usize) -> Vec<f32> {
        let mut result = Vec::with_capacity(n);
        let capacity = self.buffer.len();

        let start = if self.head >= n {
            self.head - n
        } else {
            capacity - (n - self.head)
        };

        if start + n <= capacity {
            result.extend_from_slice(&self.buffer[start..start + n]);
        } else {
            result.extend_from_slice(&self.buffer[start..]);
            let remaining = n - (capacity - start);
            result.extend_from_slice(&self.buffer[..remaining]);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_push_and_get() {
        let window = SlidingWindow::with_default_config();
        let samples = vec![1.0_f32; 100];
        window.push(&samples).await.unwrap();

        assert_eq!(window.len().await, 100);
    }

    #[tokio::test]
    async fn test_circular_overflow() {
        let config = WindowConfig {
            duration_secs: 0.1,
            sample_rate: 1000,
        };
        let window = SlidingWindow::new(config);

        let samples = vec![1.0_f32; 200];
        let dropped = window.push(&samples).await.unwrap();

        assert_eq!(dropped, 100);
        assert_eq!(window.len().await, 100);
    }

    #[tokio::test]
    async fn test_get_last_n() {
        let window = SlidingWindow::with_default_config();

        for i in 1..=10 {
            window.push(&[i as f32]).await.unwrap();
        }

        let last_5 = window.get_last_n(5).await.unwrap();
        assert_eq!(last_5, vec![6.0, 7.0, 8.0, 9.0, 10.0]);
    }
}
