use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub data_dir: PathBuf,
    pub static_dir: PathBuf,
    pub cache_cleanup_days: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("data"),
            static_dir: PathBuf::from("data/static"),
            cache_cleanup_days: 7,
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config = Self::default();
        
        // 确保目录存在
        std::fs::create_dir_all(&config.data_dir)?;
        std::fs::create_dir_all(&config.static_dir)?;
        std::fs::create_dir_all(config.data_dir.join("fonts"))?;
        
        Ok(config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    pub id: String,
    pub version: String,
    pub font_family: String,
    pub fallback: Vec<String>,
    pub license: String,
    pub files: Vec<FontFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontFile {
    pub name: String,
    pub path: String,
    pub font_family: String,
}

impl FontConfig {
    pub fn load_from_dir(font_dir: &PathBuf) -> Result<Self> {
        let config_path = font_dir.join("config.json");
        let content = std::fs::read_to_string(config_path)?;
        let config: FontConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
    
    pub fn save_to_dir(&self, font_dir: &PathBuf) -> Result<()> {
        let config_path = font_dir.join("config.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }
}