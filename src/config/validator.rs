use crate::config::model::Config;
use crate::error::{ConfigError, Result};
use std::path::{Path, PathBuf};

pub fn validate_config(config: &Config) -> Result<()> {
    // Validate that at least one spec is defined
    if config.specs.is_empty() {
        return Err(ConfigError::NoSpecDefined.into());
    }

    // Validate specs configuration
    // Check for duplicate names
    let mut seen_names = std::collections::HashSet::new();
    for spec in &config.specs {
        if seen_names.contains(&spec.name) {
            return Err(ConfigError::DuplicateSpecName {
                name: spec.name.clone(),
            }
            .into());
        }
        seen_names.insert(&spec.name);

        // Validate spec name
        if spec.name.is_empty() {
            return Err(ConfigError::InvalidSpecName {
                name: spec.name.clone(),
            }
            .into());
        }

        // Validate spec name format (alphanumeric, hyphens, underscores only)
        if !spec
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(ConfigError::InvalidSpecName {
                name: spec.name.clone(),
            }
            .into());
        }

        // Validate spec path is not empty
        if spec.path.is_empty() {
            return Err(ConfigError::Invalid {
                message: format!("Spec '{}' has an empty path", spec.name),
            }
            .into());
        }

        // Validate per-spec schemas output path
        let schemas_output = PathBuf::from(&spec.schemas.output);
        if schemas_output.is_absolute() {
            validate_safe_path(&schemas_output)?;
        }

        // Validate per-spec apis output path
        let apis_output = PathBuf::from(&spec.apis.output);
        if apis_output.is_absolute() {
            validate_safe_path(&apis_output)?;
        }

        // Validate per-spec API style
        if spec.apis.style != "fetch" {
            return Err(ConfigError::Invalid {
                message: format!(
                    "Unsupported API style for spec '{}': {}. Only 'fetch' is supported.",
                    spec.name, spec.apis.style
                ),
            }
            .into());
        }
    }

    // Validate root_dir
    let root_dir = PathBuf::from(&config.root_dir);
    if root_dir.is_absolute() && !root_dir.exists() {
        return Err(ConfigError::Invalid {
            message: format!("Root directory does not exist: {}", config.root_dir),
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

    #[test]
    fn test_validate_config_both_spec_and_specs() {
        let mut config = Config::default();
        config.spec_path = Some("openapi.json".to_string());
        config.specs = Some(vec![
            crate::config::model::SpecEntry {
                name: "auth".to_string(),
                path: "specs/auth.yaml".to_string(),
                schemas: crate::config::model::SchemasConfig::default(),
                apis: crate::config::model::ApisConfig::default(),
                modules: crate::config::model::ModulesConfig::default(),
            },
        ]);

        let result = validate_config(&config);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Both 'spec_path' and 'specs'"));
    }

    #[test]
    fn test_validate_config_no_spec_defined() {
        let config = Config::default();
        // Default config has no spec_path or specs, so this should fail
        let result = validate_config(&config);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Neither 'spec_path' nor 'specs'"));
    }

    #[test]
    fn test_validate_config_empty_specs_array() {
        let mut config = Config::default();
        config.specs = Some(vec![]);

        let result = validate_config(&config);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("At least one spec must be defined"));
    }

    #[test]
    fn test_validate_config_duplicate_spec_names() {
        let mut config = Config::default();
        config.specs = Some(vec![
            crate::config::model::SpecEntry {
                name: "auth".to_string(),
                path: "specs/auth.yaml".to_string(),
                schemas: crate::config::model::SchemasConfig::default(),
                apis: crate::config::model::ApisConfig::default(),
                modules: crate::config::model::ModulesConfig::default(),
            },
            crate::config::model::SpecEntry {
                name: "auth".to_string(),
                path: "specs/auth2.yaml".to_string(),
                schemas: crate::config::model::SchemasConfig::default(),
                apis: crate::config::model::ApisConfig::default(),
                modules: crate::config::model::ModulesConfig::default(),
            },
        ]);

        let result = validate_config(&config);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Duplicate spec name"));
    }

    #[test]
    fn test_validate_config_invalid_spec_name() {
        let mut config = Config::default();
        config.specs = Some(vec![crate::config::model::SpecEntry {
            name: "invalid name".to_string(), // contains space
            path: "specs/auth.yaml".to_string(),
            schemas: None,
            apis: None,
            modules: None,
        }]);

        let result = validate_config(&config);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid spec name"));
    }

    #[test]
    fn test_validate_config_empty_spec_name() {
        let mut config = Config::default();
        config.specs = Some(vec![crate::config::model::SpecEntry {
            name: "".to_string(),
            path: "specs/auth.yaml".to_string(),
            schemas: None,
            apis: None,
            modules: None,
        }]);

        let result = validate_config(&config);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid spec name"));
    }

    #[test]
    fn test_validate_config_empty_spec_path() {
        let mut config = Config::default();
        config.specs = Some(vec![crate::config::model::SpecEntry {
            name: "auth".to_string(),
            path: "".to_string(),
            schemas: None,
            apis: None,
            modules: None,
        }]);

        let result = validate_config(&config);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("empty path"));
    }

    #[test]
    fn test_validate_config_valid_multi_spec() {
        let mut config = Config::default();
        config.specs = Some(vec![
            crate::config::model::SpecEntry {
                name: "auth".to_string(),
                path: "specs/auth.yaml".to_string(),
                schemas: crate::config::model::SchemasConfig::default(),
                apis: crate::config::model::ApisConfig::default(),
                modules: crate::config::model::ModulesConfig::default(),
            },
            crate::config::model::SpecEntry {
                name: "orders".to_string(),
                path: "specs/orders.json".to_string(),
                schemas: crate::config::model::SchemasConfig::default(),
                apis: crate::config::model::ApisConfig::default(),
                modules: crate::config::model::ModulesConfig::default(),
            },
        ]);

        let result = validate_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_config_valid_single_spec() {
        let mut config = Config::default();
        config.spec_path = Some("openapi.json".to_string());

        let result = validate_config(&config);
        assert!(result.is_ok());
    }
}
