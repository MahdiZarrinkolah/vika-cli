use crate::config::model::Config;
use crate::error::{ConfigError, Result};
use std::path::{Path, PathBuf};

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

fn validate_safe_path(path: &Path) -> Result<()> {
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

    #[test]
    fn test_validate_safe_path_etc() {
        let path = PathBuf::from("/etc/test");
        let result = validate_safe_path(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_safe_path_usr() {
        let path = PathBuf::from("/usr/test");
        let result = validate_safe_path(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_safe_path_bin() {
        let path = PathBuf::from("/bin/test");
        let result = validate_safe_path(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_safe_path_root() {
        let path = PathBuf::from("/");
        let result = validate_safe_path(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_safe_path_valid() {
        let path = PathBuf::from("/home/user/project");
        let result = validate_safe_path(&path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_config_absolute_paths() {
        let mut config = Config::default();
        config.schemas.output = "/home/user/schemas".to_string();
        config.apis.output = "/home/user/apis".to_string();

        let result = validate_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_config_unsafe_schemas_path() {
        let mut config = Config::default();
        config.schemas.output = "/etc/schemas".to_string();

        let result = validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_config_unsafe_apis_path() {
        let mut config = Config::default();
        config.apis.output = "/usr/apis".to_string();

        let result = validate_config(&config);
        assert!(result.is_err());
    }
}
