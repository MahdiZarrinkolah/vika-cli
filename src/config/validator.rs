use crate::config::model::Config;
use crate::error::{ConfigError, Result};
use std::path::PathBuf;

pub fn validate_config(config: &Config) -> Result<()> {
    // Validate root_dir
    let root_dir = PathBuf::from(&config.root_dir);
    if root_dir.is_absolute() && !root_dir.exists() {
        return Err(ConfigError::Invalid {
            message: format!("Root directory does not exist: {}", config.root_dir),
        }
        .into());
    }

    // Validate schemas output path
    let schemas_output = PathBuf::from(&config.schemas.output);
    if schemas_output.is_absolute() {
        validate_safe_path(&schemas_output)?;
    }

    // Validate apis output path
    let apis_output = PathBuf::from(&config.apis.output);
    if apis_output.is_absolute() {
        validate_safe_path(&apis_output)?;
    }

    // Validate style
    if config.apis.style != "fetch" {
        return Err(ConfigError::Invalid {
            message: format!(
                "Unsupported API style: {}. Only 'fetch' is supported.",
                config.apis.style
            ),
        }
        .into());
    }

    Ok(())
}

fn validate_safe_path(path: &PathBuf) -> Result<()> {
    // Prevent writing to system directories
    let path_str = path.to_string_lossy();

    if path_str.contains("/etc/")
        || path_str.contains("/usr/")
        || path_str.contains("/bin/")
        || path_str.contains("/sbin/")
        || path_str.contains("/var/")
        || path_str.contains("/opt/")
        || path_str == "/"
        || path_str == "/root"
    {
        return Err(ConfigError::InvalidOutputDirectory {
            path: path_str.to_string(),
        }
        .into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::Config;

    #[test]
    fn test_validate_config_valid() {
        let config = Config::default();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_invalid_style() {
        let mut config = Config::default();
        config.apis.style = "invalid".to_string();

        let result = validate_config(&config);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Unsupported API style"));
    }
}
