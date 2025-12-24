use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config { pub audio: AudioConfig, pub vad: VadConfig, pub whisper: WhisperConfig, pub transcription: TranscriptionConfig, pub mcp: McpConfig }

impl Default for Config {
    fn default() -> Self { Self { audio: AudioConfig::default(), vad: VadConfig::default(), whisper: WhisperConfig::default(), transcription: TranscriptionConfig::default(), mcp: McpConfig::default() } }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AudioConfig { pub sample_rate: u32, pub channels: u16 }
impl Default for AudioConfig { fn default() -> Self { Self { sample_rate: 16000, channels: 1 } } }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VadConfig { pub threshold: f32 }
impl Default for VadConfig { fn default() -> Self { Self { threshold: 0.5 } } }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WhisperConfig { pub model_size: String, pub threads: usize, pub enable_gpu: bool, pub memory: MemoryConfig }
impl Default for WhisperConfig { fn default() -> Self { Self { model_size: String::from("base"), threads: 4, enable_gpu: true, memory: MemoryConfig::default() } } }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemoryConfig { pub idle_timeout_secs: u64, pub max_sessions: usize }
impl Default for MemoryConfig { fn default() -> Self { Self { idle_timeout_secs: 300, max_sessions: 10 } } }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TranscriptionConfig { pub interval_ms: u64, pub detect_language: bool }
impl Default for TranscriptionConfig { fn default() -> Self { Self { interval_ms: 500, detect_language: true } } }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpConfig { pub name: String, pub transport: String }
impl Default for McpConfig { fn default() -> Self { Self { name: String::from("vtt-mcp"), transport: String::from("stdio") } } }
