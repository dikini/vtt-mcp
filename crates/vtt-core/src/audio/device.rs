//! Audio device enumeration and management
//!
//! This module will be implemented in task vtt-mcp-csh.1.3.
//! It will provide device discovery and selection.

use super::error::AudioResult;

/// Audio device information
#[derive(Debug, Clone)]
pub struct AudioDevice {
    /// Device name
    pub name: String,
    /// Whether this is the default device
    pub is_default: bool,
}

/// List available audio input devices
///
/// # Errors
///
/// Returns an error if device enumeration fails.
pub fn list_devices() -> AudioResult<Vec<AudioDevice>> {
    // Implementation in task vtt-mcp-csh.1.3
    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_devices() {
        let result = list_devices();
        assert!(result.is_ok());
    }
}
