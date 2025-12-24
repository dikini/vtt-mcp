use super::schema::Config;
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn load() -> Result<Config> {
        if let Some(user) = Self::find_user_config() {
            return Self::load_from_file(&user);
        }
        if let Some(sys) = Self::find_system_config() {
            return Self::load_from_file(&sys);
        }
        Ok(Config::default())
    }
    
    pub fn load_from_file(path: &Path) -> Result<Config> {
        let contents = fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }
    
    fn find_user_config() -> Option<PathBuf> {
        let p = dirs::config_dir()?.join("vtt-mcp").join("config.toml");
        if p.exists() { Some(p) } else { None }
    }
    
    fn find_system_config() -> Option<PathBuf> {
        let p = PathBuf::from("/etc/vtt-mcp/config.toml");
        if p.exists() { Some(p) } else { None }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_load_default_when_no_config() {
        // Mock no config files existing
        let config = ConfigLoader::load().unwrap();
        // Should return default config
        assert_eq!(config.audio.sample_rate, 16000);
    }

    #[test]
    fn test_load_from_valid_toml() {
        let toml_content = r#"
[audio]
sample_rate = 48000
channels = 2

[vad]
threshold = 0.3

[whisper]
model_size = "tiny"
threads = 2
enable_gpu = true

[whisper.memory]
idle_timeout_secs = 120
max_sessions = 3

[transcription]
interval_ms = 250
detect_language = false

[mcp]
name = "test"
transport = "stdio"
"#;
        
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.audio.sample_rate, 48000);
        assert_eq!(config.audio.channels, 2);
        assert_eq!(config.vad.threshold, 0.3);
        assert_eq!(config.whisper.model_size, "tiny");
        assert_eq!(config.whisper.threads, 2);
        assert_eq!(config.whisper.memory.idle_timeout_secs, 120);
        assert_eq!(config.transcription.interval_ms, 250);
        assert!(!config.transcription.detect_language);
    }

    #[test]
    fn test_invalid_toml() {
        let toml_content = "invalid toml content [[[";
        
        let result: Result<Config, _> = toml::from_str(toml_content);
        assert!(result.is_err());
    }
}
