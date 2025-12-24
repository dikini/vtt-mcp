//! Whisper configuration

/// Configuration for Whisper transcription
#[derive(Debug, Clone)]
pub struct WhisperConfig {
    /// Path to the Whisper model file (e.g., ggml-base.bin)
    pub model_path: String,

    /// Number of threads to use for processing
    /// Default: number of physical CPU cores
    pub n_threads: i32,

    /// Whether to use GPU acceleration (CUDA)
    pub use_gpu: bool,

    /// Sample rate required by the model (Whisper requires 16kHz)
    /// Note: Audio must be resampled to this rate before transcription
    pub required_sample_rate: u32,

    /// Language code (e.g., "en", "es") or None for auto-detection
    pub language: Option<String>,

    /// Translate to English (if applicable)
    pub translate: bool,

    /// Maximum length of context (in tokens) to use for transcription
    pub n_max_context: i32,

    /// Maximum number of text tokens to generate
    pub n_max_text_tokens: i32,

    /// Offset in milliseconds to start transcription from
    pub offset_ms: i32,

    /// Duration in milliseconds to transcribe
    pub duration_ms: i32,

    /// Memory management: idle timeout in seconds before unloading model
    /// Set to None to keep model loaded permanently
    pub idle_timeout_secs: Option<u64>,

    /// Memory management: maximum concurrent sessions allowed
    pub max_sessions: usize,
}

impl Default for WhisperConfig {
    fn default() -> Self {
        Self {
            model_path: "models/ggml-base.bin".to_string(),
            n_threads: num_cpus::get_physical() as i32,
            use_gpu: true,
            required_sample_rate: 16000,
            language: None,
            translate: false,
            n_max_context: 0,
            n_max_text_tokens: 0,
            offset_ms: 0,
            duration_ms: 0,
            idle_timeout_secs: None,  // Keep model loaded by default
            max_sessions: 4,  // Allow up to 4 concurrent sessions
        }
    }
}

impl WhisperConfig {
    /// Create a new config with a custom model path
    pub fn with_model_path(mut self, path: impl Into<String>) -> Self {
        self.model_path = path.into();
        self
    }

    /// Create a new config with a specific language
    pub fn with_language(mut self, lang: impl Into<String>) -> Self {
        self.language = Some(lang.into());
        self
    }

    /// Create a new config with GPU enabled/disabled
    pub fn with_gpu(mut self, use_gpu: bool) -> Self {
        self.use_gpu = use_gpu;
        self
    }

    /// Create a new config with custom thread count
    pub fn with_threads(mut self, n_threads: i32) -> Self {
        self.n_threads = n_threads.max(1);
        self
    }

    /// Create a new config for translation
    pub fn with_translation(mut self, translate: bool) -> Self {
        self.translate = translate;
        self
    }

    /// Set idle timeout for model unloading (in seconds)
    /// Set to None to keep model loaded permanently
    pub fn with_idle_timeout(mut self, timeout_secs: Option<u64>) -> Self {
        self.idle_timeout_secs = timeout_secs;
        self
    }

    /// Set maximum concurrent sessions
    pub fn with_max_sessions(mut self, max_sessions: usize) -> Self {
        self.max_sessions = max_sessions.max(1);
        self
    }
}
