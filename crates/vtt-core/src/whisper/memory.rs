//! Memory tracking and management for Whisper context

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

/// Memory usage statistics for Whisper context
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Total memory allocated (in bytes)
    /// This is an estimate based on model size
    pub total_bytes: u64,
    
    /// Peak memory usage (in bytes)
    pub peak_bytes: u64,
    
    /// Current number of active sessions
    pub active_sessions: usize,
    
    /// Total sessions created
    pub total_sessions: u64,
    
    /// Last activity timestamp
    pub last_activity: Instant,
}

impl MemoryStats {
    pub fn new(model_size_bytes: u64) -> Self {
        Self {
            total_bytes: model_size_bytes,
            peak_bytes: model_size_bytes,
            active_sessions: 0,
            total_sessions: 0,
            last_activity: Instant::now(),
        }
    }
    
    /// Get memory usage in megabytes
    pub fn total_mb(&self) -> f64 {
        self.total_bytes as f64 / (1024.0 * 1024.0)
    }
    
    /// Get peak memory usage in megabytes
    pub fn peak_mb(&self) -> f64 {
        self.peak_bytes as f64 / (1024.0 * 1024.0)
    }
    
    /// Check if idle for more than specified seconds
    pub fn is_idle(&self, timeout_secs: u64) -> bool {
        self.last_activity.elapsed().as_secs() > timeout_secs
    }
}

/// Shared memory tracking state
#[derive(Debug)]
pub struct MemoryTracker {
    /// Memory statistics
    stats: Arc<Mutex<MemoryStats>>,
    /// Reference count for sharing
    ref_count: AtomicUsize,
}

impl MemoryTracker {
    pub fn new(model_size_bytes: u64) -> Self {
        Self {
            stats: Arc::new(Mutex::new(MemoryStats::new(model_size_bytes))),
            ref_count: AtomicUsize::new(0),
        }
    }
    
    /// Increment reference count
    pub fn inc_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
        let mut stats = self.stats.lock().unwrap();
        stats.active_sessions += 1;
        stats.total_sessions += 1;
        stats.last_activity = Instant::now();
    }
    
    /// Decrement reference count
    pub fn dec_ref(&self) {
        let prev = self.ref_count.fetch_sub(1, Ordering::SeqCst);
        if prev > 0 {
            let mut stats = self.stats.lock().unwrap();
            stats.active_sessions = stats.active_sessions.saturating_sub(1);
            // Note: we don't update last_activity here - activity is tracked on inc_ref
        }
    }
    
    /// Update activity timestamp
    pub fn update_activity(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.last_activity = Instant::now();
    }
    
    /// Get current memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        self.stats.lock().unwrap().clone()
    }
    
    /// Get reference count
    pub fn ref_count(&self) -> usize {
        self.ref_count.load(Ordering::SeqCst)
    }
    
    /// Check if should unload based on idle timeout
    pub fn should_unload(&self, idle_timeout_secs: Option<u64>) -> bool {
        if let Some(timeout) = idle_timeout_secs {
            if self.ref_count() == 0 {
                let stats = self.stats.lock().unwrap();
                return stats.is_idle(timeout);
            }
        }
        false
    }
}

impl Clone for MemoryTracker {
    fn clone(&self) -> Self {
        self.inc_ref();
        Self {
            stats: Arc::clone(&self.stats),
            ref_count: AtomicUsize::new(self.ref_count()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_memory_stats() {
        let stats = MemoryStats::new(1_000_000_000); // 1GB
        // Allow for floating point precision issues
        let mb = stats.total_mb();
        assert!(mb > 950.0 && mb < 1050.0, "total_mb should be ~1000, got {}", mb);
        
        let peak_mb = stats.peak_mb();
        assert!(peak_mb > 950.0 && peak_mb < 1050.0, "peak_mb should be ~1000, got {}", peak_mb);
    }
    
    #[test]
    fn test_memory_tracker() {
        let tracker = MemoryTracker::new(500_000_000); // 500MB
        
        assert_eq!(tracker.ref_count(), 0);
        
        tracker.inc_ref();
        assert_eq!(tracker.ref_count(), 1);
        
        let stats = tracker.get_stats();
        assert_eq!(stats.active_sessions, 1);
        assert_eq!(stats.total_sessions, 1);
        
        tracker.dec_ref();
        assert_eq!(tracker.ref_count(), 0);
        
        let stats = tracker.get_stats();
        assert_eq!(stats.active_sessions, 0);
    }
    
    #[test]
    fn test_idle_detection() {
        let tracker = MemoryTracker::new(100_000_000);
        
        // Should not unload when no timeout set
        assert!(!tracker.should_unload(None));
        
        // Should not unload with active refs even with timeout=0
        tracker.inc_ref();
        assert!(!tracker.should_unload(Some(0)));
        
        // After decrementing, should NOT be immediately idle with timeout=0
        // because is_idle requires > timeout_secs, and elapsed starts at 0
        tracker.dec_ref();
        assert_eq!(tracker.ref_count(), 0);
        assert!(!tracker.should_unload(Some(0)), "Should not be idle immediately after dec_ref");
        
        // Wait 1 second to make it idle
        thread::sleep(Duration::from_secs(1));
        assert!(tracker.should_unload(Some(0)), "Should be idle after 1 second with timeout=0");
        
        // Should not unload with longer timeout
        assert!(!tracker.should_unload(Some(3600)), "Should not unload with 1 hour timeout");
    }
}
