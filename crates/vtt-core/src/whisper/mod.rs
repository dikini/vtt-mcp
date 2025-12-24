pub mod config; pub mod context; pub mod error; pub mod gpu; pub mod downloader; pub mod language;
pub use config::WhisperConfig; pub use context::WhisperContext; pub use error::{WhisperError, WhisperResult, Transcription};
pub use gpu::{GpuBackend, GpuDetection, GpuDeviceInfo, detect_gpu, get_gpu_info, is_gpu_available, get_gpu_message};
pub use downloader::{ModelDownloadConfig, download_model, ensure_model, needs_download};
pub use language::{Language, SUPPORTED_LANGUAGES, AUTO_DETECT, supported_codes, display_name};
