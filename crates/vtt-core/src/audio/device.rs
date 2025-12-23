//! Audio device enumeration and management

use super::error::{AudioError, AudioResult};
use cpal::traits::{DeviceTrait, HostTrait};

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
/// Returns an error if device enumeration fails or if no host is available.
pub fn list_devices() -> AudioResult<Vec<AudioDevice>> {
    let host = cpal::default_host();

    let default_device = host.default_input_device();
    let default_name = default_device.as_ref().and_then(|d| d.name().ok());

    let devices = host.input_devices()?;

    let mut result = Vec::new();
    for device in devices {
        if let Ok(name) = device.name() {
            let is_default = default_name.as_ref() == Some(&name);
            result.push(AudioDevice { name, is_default });
        }
    }

    Ok(result)
}

/// Get the default audio input device
///
/// # Errors
///
/// Returns an error if no default device is available.
pub fn default_device() -> AudioResult<cpal::Device> {
    let host = cpal::default_host();
    host.default_input_device()
        .ok_or_else(|| AudioError::DeviceError("No default input device available".to_string()))
}

/// Get device by name
///
/// # Errors
///
/// Returns an error if the device is not found or enumeration fails.
pub fn device_by_name(name: &str) -> AudioResult<cpal::Device> {
    let host = cpal::default_host();
    let devices = host.input_devices()?;

    for device in devices {
        if let Ok(device_name) = device.name() {
            if device_name == name {
                return Ok(device);
            }
        }
    }

    Err(AudioError::DeviceError(format!(
        "Device not found: {}",
        name
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_devices() {
        let result = list_devices();
        // Should succeed even if no devices (returns empty vec)
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_device() {
        let result = default_device();
        // May fail on CI systems without audio hardware
        // Just verify it returns a result
        let _ = result;
    }
}
