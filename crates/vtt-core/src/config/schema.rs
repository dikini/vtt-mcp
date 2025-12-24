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



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.audio.sample_rate, 16000);
        assert_eq!(config.audio.channels, 1);
        assert_eq!(config.vad.threshold, 0.5);
        assert_eq!(config.whisper.model_size, "base");
        assert_eq!(config.mcp.name, "vtt-mcp");
    }

    #[test]
    fn test_audio_config() {
        let config = AudioConfig::default();
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.channels, 1);
    }

    #[test]
    fn test_vad_config() {
        let config = VadConfig::default();
        assert_eq!(config.threshold, 0.5);
    }

    #[test]
    fn test_whisper_config() {
        let config = WhisperConfig::default();
        assert_eq!(config.model_size, "base");
        assert_eq!(config.threads, 4);
        assert!(config.enable_gpu);
        assert_eq!(config.memory.idle_timeout_secs, 300);
        assert_eq!(config.memory.max_sessions, 10);
    }

    #[test]
    fn test_memory_config() {
        let config = MemoryConfig::default();
        assert_eq!(config.idle_timeout_secs, 300);
        assert_eq!(config.max_sessions, 10);
    }

    #[test]
    fn test_transcription_config() {
        let config = TranscriptionConfig::default();
        assert_eq!(config.interval_ms, 500);
        assert!(config.detect_language);
    }

    #[test]
    fn test_mcp_config() {
        let config = McpConfig::default();
        assert_eq!(config.name, "vtt-mcp");
        assert_eq!(config.transport, "stdio");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("sample_rate"));
        assert!(toml.contains("threshold"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
[audio]
sample_rate = 48000
channels = 2

[vad]
threshold = 0.7

[whisper]
model_size = "small"
threads = 8
enable_gpu = false

[whisper.memory]
idle_timeout_secs = 600
max_sessions = 5

[transcription]
interval_ms = 1000
detect_language = false

[mcp]
name = "test-server"
transport = "tcp"
"#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.audio.sample_rate, 48000);
        assert_eq!(config.audio.channels, 2);
        assert_eq!(config.vad.threshold, 0.7);
        assert_eq!(config.whisper.model_size, "small");
        assert_eq!(config.whisper.threads, 8);
        assert!(!config.whisper.enable_gpu);
        assert_eq!(config.whisper.memory.idle_timeout_secs, 600);
        assert_eq!(config.transcription.interval_ms, 1000);
        assert!(!config.transcription.detect_language);
        assert_eq!(config.mcp.name, "test-server");
        assert_eq!(config.mcp.transport, "tcp");
    }
}
