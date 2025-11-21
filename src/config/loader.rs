use anyhow::{Context, Result};
use std::path::PathBuf;
use crate::config::model::Config;

const CONFIG_FILE: &str = ".vika.json";

pub fn load_config() -> Result<Config> {
    let config_path = PathBuf::from(CONFIG_FILE);
    
    if !config_path.exists() {
        return Ok(Config::default());
    }
    
    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file: {}", CONFIG_FILE))?;
    
    let config: Config = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", CONFIG_FILE))?;
    
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<()> {
    let config_path = PathBuf::from(CONFIG_FILE);
    let content = serde_json::to_string_pretty(config)
        .context("Failed to serialize config")?;
    
    std::fs::write(&config_path, content)
        .with_context(|| format!("Failed to write config file: {}", CONFIG_FILE))?;
    
    Ok(())
}

