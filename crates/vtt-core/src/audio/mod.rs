//! Audio processing

pub mod capture;
pub mod device;
pub mod error;
pub mod format;
pub mod writer;

#[cfg(target_os = "linux")]
pub mod pipewire_capture;

pub use capture::AudioCapture;
pub use device::{default_device, device_by_name, list_devices, AudioDevice};
pub use error::{AudioError, AudioResult};
pub use format::{AudioFormat, AudioSampleFormat};
pub use writer::write_wav;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_module_version() { assert_eq!(VERSION, "0.1.0"); }
}
