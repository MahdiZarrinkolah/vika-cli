use crate::config::model::Config;
use crate::error::{ConfigError, Result, VikaError};
use std::path::PathBuf;

const CONFIG_FILE: &str = ".vika.json";

pub fn load_config() -> Result<Config> {
    let config_path = PathBuf::from(CONFIG_FILE);

    if !config_path.exists() {
        return Ok(Config::default());
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| VikaError::from(ConfigError::ReadError(e)))?;

    let config: Config =
        serde_json::from_str(&content).map_err(|e| VikaError::from(ConfigError::ParseError(e)))?;

    Ok(config)
}

pub fn save_config(config: &Config) -> Result<()> {
    let config_path = PathBuf::from(CONFIG_FILE);

    // Ensure $schema is set (use default if not present)
    let mut config_to_save = config.clone();
    if config_to_save.schema.is_empty() {
        config_to_save.schema = crate::config::model::default_schema();
    }

    let content = serde_json::to_string_pretty(&config_to_save)
        .map_err(|e| VikaError::from(ConfigError::ParseError(e)))?;

    std::fs::write(&config_path, content)
        .map_err(|e| VikaError::from(ConfigError::ReadError(e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_save_and_load_config() {
        let temp_dir = tempfile::tempdir().unwrap();
        let original_dir = env::current_dir().ok();

        // Only change directory if we can get the current one
        if let Ok(_) = env::set_current_dir(&temp_dir) {
            let config = Config::default();
            save_config(&config).unwrap();

            let loaded = load_config().unwrap();
            assert_eq!(loaded.root_dir, config.root_dir);
            assert_eq!(loaded.schemas.output, config.schemas.output);

            if let Some(orig) = original_dir {
                let _ = env::set_current_dir(orig);
            }
        }
    }

    #[test]
    fn test_load_config_not_exists() {
        let temp_dir = tempfile::tempdir().unwrap();
        let original_dir = env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

        env::set_current_dir(&temp_dir).unwrap();

        let config = load_config().unwrap();
        // Should return default config when file doesn't exist
        assert_eq!(config.root_dir, "src");

        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_save_config_with_empty_schema() {
        let temp_dir = tempfile::tempdir().unwrap();
        let original_dir = env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

        env::set_current_dir(&temp_dir).unwrap();

        let mut config = Config::default();
        config.schema = String::new();
        save_config(&config).unwrap();

        let loaded = load_config().unwrap();
        // Schema should be set to default
        assert!(!loaded.schema.is_empty());

        env::set_current_dir(original_dir).unwrap();
    }
}
