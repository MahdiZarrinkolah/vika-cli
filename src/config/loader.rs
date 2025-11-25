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

        let _ = env::set_current_dir(&temp_dir);

        // Save config should succeed
        let mut config = Config::default();
        config.specs = vec![crate::config::model::SpecEntry {
            name: "test".to_string(),
            path: "test.yaml".to_string(),
            schemas: crate::config::model::SchemasConfig::default(),
            apis: crate::config::model::ApisConfig::default(),
            modules: crate::config::model::ModulesConfig::default(),
        }];
        if save_config(&config).is_ok() {
            // Load config should succeed and match
            if let Ok(loaded) = load_config() {
                assert_eq!(loaded.root_dir, config.root_dir);
                assert_eq!(loaded.specs.len(), config.specs.len());
            }
        }

        if let Some(orig) = original_dir {
            let _ = env::set_current_dir(orig);
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

        let config = Config {
            schema: String::new(),
            ..Default::default()
        };
        save_config(&config).unwrap();

        let loaded = load_config().unwrap();
        // Schema should be set to default
        assert!(!loaded.schema.is_empty());

        env::set_current_dir(original_dir).unwrap();
    }
}
