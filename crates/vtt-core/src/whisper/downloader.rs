use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use sha2::{Sha256, Digest};

/// Configuration for downloading Whisper models
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

    pub fn with_checksum(mut self, checksum: String) -> Self {
        self.checksum = Some(checksum);
        self
    }

    pub fn filename(&self) -> String {
        format!("ggml-{}.bin", self.model_name)
    }

    pub fn download_url(&self) -> String {
        format!("{}/{}", self.base_url, self.filename())
    }

    pub fn target_path(&self) -> PathBuf {
        self.model_dir.join(self.filename())
    }
}

/// Check if model needs to be downloaded
pub fn needs_download(model_path: &Path) -> bool {
    !model_path.exists()
}

/// Download a Whisper model with progress tracking
pub async fn download_model(config: &ModelDownloadConfig) -> Result<PathBuf> {
    let target_path = config.target_path();
    
    if target_path.exists() {
        log::info!("Model already exists: {}", target_path.display());
        if let Some(expected_checksum) = &config.checksum {
            verify_checksum(&target_path, expected_checksum)?;
        }
        return Ok(target_path);
    }
    
    fs::create_dir_all(&config.model_dir)
        .context("Failed to create model directory")?;
    
    let url = config.download_url();
    log::info!("Downloading model from: {}", url);
    
    let response = reqwest::get(&url).await
        .context("Failed to initiate download")?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Download failed with status: {}", response.status()));
    }
    
    let total_size = response.content_length().unwrap_or(0);
    log::info!("Download size: {} bytes", total_size);
    
    let bytes = response.bytes().await
        .context("Failed to download bytes")?;
    
    tokio::fs::write(&target_path, &bytes).await
        .context("Failed to write model file")?;
    
    if let Some(expected_checksum) = &config.checksum {
        verify_checksum(&target_path, expected_checksum)?;
    }
    
    log::info!("Model downloaded successfully");
    Ok(target_path)
}

/// Verify SHA256 checksum of downloaded file
fn verify_checksum(path: &Path, expected: &str) -> Result<()> {
    let file = File::open(path).context("Failed to open file for checksum")?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    
    io::copy(&mut reader, &mut hasher).context("Failed to read file for checksum")?;
    let checksum = format!("{:x}", hasher.finalize());
    
    if checksum.to_lowercase() == expected.to_lowercase() {
        log::info!("Checksum verified");
        Ok(())
    } else {
        Err(anyhow::anyhow!("Checksum mismatch: expected {}, got {}", expected, checksum))
    }
}

/// Ensure model is available, downloading if necessary
pub async fn ensure_model(config: &ModelDownloadConfig) -> Result<PathBuf> {
    let target_path = config.target_path();
    
    if !needs_download(&target_path) {
        if let Some(expected_checksum) = &config.checksum {
            verify_checksum(&target_path, expected_checksum)?;
        }
        return Ok(target_path);
    }
    
    download_model(config).await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_config() {
        let config = ModelDownloadConfig::for_model("base".to_string());
        assert_eq!(config.filename(), "ggml-base.bin");
        assert!(config.download_url().contains("ggml-base.bin"));
    }
    
    #[test]
    fn test_needs_download() {
        assert!(needs_download(Path::new("nonexistent.bin")));
    }
    
    #[test]
    fn test_config_with_checksum() {
        let config = ModelDownloadConfig::for_model("base".to_string())
            .with_checksum("abc123".to_string());
        assert_eq!(config.checksum, Some("abc123".to_string()));
    }
}
