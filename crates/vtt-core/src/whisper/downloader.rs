use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use sha2::{Sha256, Digest};
use tokio::io::AsyncWriteExt;

pub struct ModelDownloadConfig {
    pub model_name: String,
    pub base_url: String,
    pub model_dir: PathBuf,
    pub checksum: Option<String>,
}

impl Default for ModelDownloadConfig {
    fn default() -> Self {
        Self {
            model_name: "base".to_string(),
            base_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main".to_string(),
            model_dir: PathBuf::from("models"),
            checksum: None,
        }
    }
}

impl ModelDownloadConfig {
    pub fn for_model(model_name: String) -> Self {
        Self {
            model_name,
            ..Default::default()
        }
    }

    pub fn filename(&self) -> String {
        format!("ggml-{}.bin", self.model_name)
    }

    pub fn target_path(&self) -> PathBuf {
        self.model_dir.join(self.filename())
    }
}

pub fn needs_download(model_path: &Path) -> bool {
    !model_path.exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_config() {
        let config = ModelDownloadConfig::for_model("base".to_string());
        assert_eq!(config.filename(), "ggml-base.bin");
    }

    #[test]
    fn test_needs_download() {
        assert!(needs_download(Path::new("nonexistent.bin")));
    }
}

