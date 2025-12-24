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
