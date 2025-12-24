//! Sliding window audio buffer for incremental transcription

pub mod buffer;

pub use buffer::{SlidingWindow, WindowConfig, WindowError, WindowResult};
