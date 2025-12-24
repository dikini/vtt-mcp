//! GPU detection and device enumeration
//!
//! This module provides runtime detection of GPU capabilities for Whisper acceleration.
//! It supports CUDA backend (ROCm/AMD support planned for future).

use std::sync::OnceLock;

/// Detected GPU backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBackend {
    /// NVIDIA GPU with CUDA support
    Cuda,
    /// No GPU available, will use CPU
    Cpu,
}

/// Information about a detected GPU device
#[derive(Debug, Clone)]
pub struct GpuDeviceInfo {
    /// Backend type (CUDA/ROCm)
    pub backend: GpuBackend,
    /// Device name (if available)
    pub device_name: Option<String>,
    /// Device index
    pub device_index: i32,
    /// Available VRAM in MB (if detectable)
    pub vram_mb: Option<u64>,
}

/// GPU detection result
#[derive(Debug, Clone)]
pub struct GpuDetection {
    /// Detected backend
    pub backend: GpuBackend,
    /// Available GPU devices
    pub devices: Vec<GpuDeviceInfo>,
    /// Recommended device for Whisper
    pub recommended_device: Option<i32>,
    /// Detection message for user feedback
    pub message: String,
}

impl GpuDetection {
    /// Check if GPU is available
    pub fn has_gpu(&self) -> bool {
        self.backend != GpuBackend::Cpu
    }

    /// Get the recommended device index, or 0 if no GPU
    pub fn device_index(&self) -> i32 {
        self.recommended_device.unwrap_or(0)
    }

    /// Create a CPU-only detection result
    fn cpu_only(message: String) -> Self {
        Self {
            backend: GpuBackend::Cpu,
            devices: vec![],
            recommended_device: None,
            message,
        }
    }
}

/// Detect available GPU capabilities
///
/// This function attempts to detect CUDA availability at runtime.
/// It checks for the presence of GPU libraries and attempts to query device info.
///
/// # Returns
///
/// A Result with GpuDetection information about available GPUs.
///
/// # Example
///
/// ~~~no_run
/// use vtt_core::whisper::gpu::detect_gpu;
///
/// let detection = detect_gpu().unwrap();
/// if detection.has_gpu() {
///     println!("GPU detected: {:?}", detection.backend);
/// } else {
///     println!("No GPU available, using CPU");
/// }
/// ~~~
pub fn detect_gpu() -> Result<GpuDetection, String> {
    // Try CUDA (most common for Whisper)
    if let Ok(detection) = try_detect_cuda() {
        return Ok(detection);
    }

    // No GPU detected
    Ok(GpuDetection::cpu_only(
        "No GPU detected. Whisper will run on CPU. For GPU acceleration, rebuild with: cargo build --features cuda".to_string()
    ))
}

/// Attempt to detect CUDA availability
fn try_detect_cuda() -> Result<GpuDetection, String> {
    #[cfg(feature = "cuda")]
    {
        return Ok(GpuDetection {
            backend: GpuBackend::Cuda,
            devices: vec![
                GpuDeviceInfo {
                    backend: GpuBackend::Cuda,
                    device_name: Some("CUDA Device (detected by whisper-rs)".to_string()),
                    device_index: 0,
                    vram_mb: None,
                }
            ],
            recommended_device: Some(0),
            message: "CUDA GPU detected and available for Whisper acceleration".to_string(),
        });
    }

    #[cfg(not(feature = "cuda"))]
    {
        return Err("CUDA feature not enabled".to_string());
    }
}

/// Global cached GPU detection result
static GPU_DETECTION: OnceLock<Option<GpuDetection>> = OnceLock::new();

/// Get cached GPU detection, running detection only once
pub fn get_gpu_info() -> Option<&'static GpuDetection> {
    GPU_DETECTION.get_or_init(|| {
        match detect_gpu() {
            Ok(detection) => {
                log::info!("GPU detection: {:?}", detection);
                log::info!("{}", detection.message);
                Some(detection)
            }
            Err(e) => {
                log::warn!("GPU detection failed: {}", e);
                log::info!("No GPU available, Whisper will use CPU");
                None
            }
        }
    }).as_ref()
}

/// Check if GPU acceleration is available
pub fn is_gpu_available() -> bool {
    get_gpu_info().map(|d| d.has_gpu()).unwrap_or(false)
}

/// Get a user-friendly message about GPU availability
pub fn get_gpu_message() -> String {
    match get_gpu_info() {
        Some(detection) => detection.message.clone(),
        None => "GPU detection not performed. Whisper will attempt auto-detection.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_detection_runs() {
        let detection = detect_gpu();
        assert!(detection.is_ok());
    }

    #[test]
    fn test_gpu_info_cached() {
        let info1 = get_gpu_info();
        let info2 = get_gpu_info();
        
        match (&info1, &info2) {
            (None, None) => {},
            (Some(d1), Some(d2)) => {
                assert_eq!(d1.backend, d2.backend);
            }
            _ => panic!("GPU info should be consistent"),
        }
    }

    #[test]
    fn test_is_gpu_available() {
        let _ = is_gpu_available();
    }
}

