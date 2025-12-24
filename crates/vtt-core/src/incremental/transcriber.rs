//! Incremental transcriber implementation

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::whisper::WhisperContext;
use crate::window::{SlidingWindow, WindowConfig};

/// Configuration for incremental transcription
#[derive(Debug, Clone)]
pub struct TranscriberConfig {
    pub transcription_interval_ms: u64,
    pub min_text_length: usize,
    pub window_duration_secs: f32,
    pub sample_rate: u32,
}

impl Default for TranscriberConfig {
    fn default() -> Self {
        Self {
            transcription_interval_ms: 500,
            min_text_length: 3,
            window_duration_secs: 3.0,
            sample_rate: 16000,
        }
    }
}

/// Partial transcription result
#[derive(Debug, Clone)]
pub struct PartialResult {
    pub text: String,
    pub is_final: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub sample_count: usize,
}

impl PartialResult {
    fn new(text: String, sample_count: usize, is_final: bool) -> Self {
        Self {
            text,
            is_final,
            timestamp: chrono::Utc::now(),
            sample_count,
        }
    }
}

struct TranscriberState {
    window: SlidingWindow,
    last_text: String,
}

pub struct IncrementalTranscriber {
    whisper: WhisperContext,
    state: Arc<Mutex<TranscriberState>>,
    config: TranscriberConfig,
}

impl IncrementalTranscriber {
    pub fn new(config: TranscriberConfig, whisper: WhisperContext) -> Self {
        let window_config = WindowConfig {
            duration_secs: config.window_duration_secs,
            sample_rate: config.sample_rate,
        };
        
        let state = TranscriberState {
            window: SlidingWindow::with_config(window_config),
            last_text: String::new(),
        };
        
        Self {
            whisper,
            state: Arc::new(Mutex::new(state)),
            config,
        }
    }
    
    pub fn with_defaults(whisper: WhisperContext) -> Self {
        Self::new(TranscriberConfig::default(), whisper)
    }
    
    pub async fn push_audio(&self, samples: &[f32]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let state = self.state.lock().await;
        state.window.push(samples).await?;
        Ok(())
    }
    
    pub async fn transcribe_current(&self) -> Result<Option<PartialResult>, Box<dyn std::error::Error + Send + Sync>> {
        let window_data = {
            let state = self.state.lock().await;
            state.window.get_all().await
        };
        
        if window_data.len() < 1600 {
            return Ok(None);
        }
        
        match self.whisper.transcribe(&window_data, 16000) {
            Ok(transcription) => {
                let text = transcription.text.trim().to_string();
                
                if text.len() < self.config.min_text_length {
                    return Ok(None);
                }
                
                let is_duplicate = {
                    let mut state = self.state.lock().await;
                    let duplicate = Self::is_duplicate(&state.last_text, &text, 0.8);
                    if !duplicate {
                        state.last_text = text.clone();
                    }
                    duplicate
                };
                
                if is_duplicate {
                    Ok(None)
                } else {
                    Ok(Some(PartialResult::new(text, window_data.len(), false)))
                }
            }
            Err(_) => Ok(None)
        }
    }
    
    pub async fn buffer_duration_secs(&self) -> f32 {
        let state = self.state.lock().await;
        state.window.duration_secs().await
    }
    
    pub async fn clear(&self) {
        let mut state = self.state.lock().await;
        state.window.clear().await;
        state.last_text.clear();
    }
    
    fn is_duplicate(prev: &str, curr: &str, threshold: f32) -> bool {
        if prev.is_empty() || curr.is_empty() {
            return false;
        }
        
        if curr.len() < prev.len() && prev.contains(curr) {
            return true;
        }
        
        let overlap_len = prev.len().min(curr.len());
        let min_overlap = (prev.len() as f32 * threshold).ceil() as usize;
        
        if overlap_len >= min_overlap {
            let prev_suffix = &prev[prev.len().saturating_sub(overlap_len)..];
            let curr_prefix = &curr[..overlap_len.min(curr.len())];
            
            if prev_suffix == curr_prefix {
                return true;
            }
        }
        
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_duplicate_detection() {
        assert!(IncrementalTranscriber::is_duplicate("hello world", "hello", 0.8));
        assert!(IncrementalTranscriber::is_duplicate("hello world test", "world test", 0.7));
        assert!(!IncrementalTranscriber::is_duplicate("hello", "world", 0.8));
    }
}
