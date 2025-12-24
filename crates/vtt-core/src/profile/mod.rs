//! Profiling utilities for performance measurement
//!
//! This module provides tools for measuring and analyzing latency
//! in the transcription pipeline.

use std::time::Instant;
use std::collections::HashMap;

/// A timing measurement for a single operation
#[derive(Debug, Clone)]
pub struct Timing {
    /// Name of the operation being measured
    pub name: String,
    /// Duration of the operation
    pub duration: std::time::Duration,
    /// Timestamp when measurement started
    pub timestamp: Instant,
}

impl Timing {
    /// Create a new timing measurement
    pub fn new(name: String, duration: std::time::Duration) -> Self {
        Self {
            name,
            duration,
            timestamp: Instant::now(),
        }
    }
    
    /// Get duration in milliseconds
    pub fn as_millis(&self) -> u128 {
        self.duration.as_millis()
    }
    
    /// Get duration in microseconds
    pub fn as_micros(&self) -> u128 {
        self.duration.as_micros()
    }
}

/// Profile data collector for multiple operations
#[derive(Debug, Default)]
pub struct ProfileData {
    /// All timing measurements collected
    timings: Vec<Timing>,
    /// Statistics per operation name
    stats: HashMap<String, TimingStats>,
}

/// Statistics for a single operation type
#[derive(Debug, Clone)]
pub struct TimingStats {
    pub operation: String,
    pub count: usize,
    pub total_ms: u128,
    pub min_ms: u128,
    pub max_ms: u128,
    pub avg_ms: f64,
}

impl ProfileData {
    /// Create a new profile data collector
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Record a timing measurement
    pub fn record(&mut self, timing: Timing) {
        self.timings.push(timing.clone());
        
        let stats = self.stats.entry(timing.name.clone()).or_insert_with(|| {
            TimingStats {
                operation: timing.name.clone(),
                count: 0,
                total_ms: 0,
                min_ms: u128::MAX,
                max_ms: 0,
                avg_ms: 0.0,
            }
        });
        
        let ms = timing.as_millis();
        stats.count += 1;
        stats.total_ms += ms;
        stats.min_ms = stats.min_ms.min(ms);
        stats.max_ms = stats.max_ms.max(ms);
        stats.avg_ms = stats.total_ms as f64 / stats.count as f64;
    }
    
    /// Get statistics for a specific operation
    pub fn get_stats(&self, operation: &str) -> Option<&TimingStats> {
        self.stats.get(operation)
    }
    
    /// Get all statistics
    pub fn all_stats(&self) -> Vec<&TimingStats> {
        self.stats.values().collect()
    }
    
    /// Generate a summary report
    pub fn summary(&self) -> String {
        let mut report = String::from("=== Performance Profile Summary ===\n\n");
        
        let mut stats: Vec<_> = self.all_stats();
        stats.sort_by(|a, b| b.avg_ms.partial_cmp(&a.avg_ms).unwrap());
        
        for stat in stats {
            report.push_str(&format!(
                "{operation}:\n  Count: {count}\n  Avg: {avg:.2}ms\n  Min: {min}ms\n  Max: {max}ms\n\n",
                operation = stat.operation,
                count = stat.count,
                avg = stat.avg_ms,
                min = stat.min_ms,
                max = stat.max_ms,
            ));
        }
        
        report
    }
}

/// A simple timer that measures elapsed time
#[derive(Debug)]
pub struct Timer {
    name: String,
    start: Instant,
}

impl Timer {
    /// Start a new timer
    pub fn start(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
        }
    }
    
    /// Stop the timer and record the measurement
    pub fn stop(self, profile: &mut ProfileData) -> Timing {
        let duration = self.start.elapsed();
        let timing = Timing::new(self.name, duration);
        profile.record(timing.clone());
        timing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_timer() {
        let mut profile = ProfileData::new();
        
        let timer = Timer::start("test_operation");
        thread::sleep(std::time::Duration::from_millis(10));
        let timing = timer.stop(&mut profile);
        
        assert!(timing.as_millis() >= 10);
        
        let stats = profile.get_stats("test_operation").unwrap();
        assert_eq!(stats.count, 1);
        assert!(stats.avg_ms >= 10.0);
    }
    
    #[test]
    fn test_profile_summary() {
        let mut profile = ProfileData::new();
        
        profile.record(Timing::new("fast_op".to_string(), std::time::Duration::from_millis(5)));
        profile.record(Timing::new("fast_op".to_string(), std::time::Duration::from_millis(15)));
        profile.record(Timing::new("slow_op".to_string(), std::time::Duration::from_millis(100)));
        
        let summary = profile.summary();
        assert!(summary.contains("fast_op"));
        assert!(summary.contains("slow_op"));
        assert!(summary.contains("Avg:"));
        
        let fast_stats = profile.get_stats("fast_op").unwrap();
        assert_eq!(fast_stats.count, 2);
        assert_eq!(fast_stats.min_ms, 5);
        assert_eq!(fast_stats.max_ms, 15);
        assert_eq!(fast_stats.avg_ms, 10.0);
    }
}
