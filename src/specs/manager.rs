use crate::config::model::{Config, SpecEntry};
use crate::error::{ConfigError, GenerationError, Result};
use dialoguer::Select;

/// Determines if the config is in multi-spec mode
pub fn is_multi_spec_mode(config: &Config) -> bool {
    config.specs.is_some()
}

/// Lists all specs from the config
pub fn list_specs(config: &Config) -> Vec<SpecEntry> {
    if let Some(ref specs) = config.specs {
        specs.clone()
    } else if let Some(ref spec_path) = config.spec_path {
        // Single spec mode: create a virtual SpecEntry
        vec![SpecEntry {
            name: "default".to_string(),
            path: spec_path.clone(),
        }]
    } else {
        vec![]
    }
}

/// Gets a spec by name from the config
pub fn get_spec_by_name(config: &Config, name: &str) -> Result<SpecEntry> {
    if let Some(ref specs) = config.specs {
        specs
            .iter()
            .find(|s| s.name == name)
            .cloned()
            .ok_or_else(|| {
                let available: Vec<String> = specs.iter().map(|s| s.name.clone()).collect();
                GenerationError::SpecNotFound {
                    name: name.to_string(),
                    available,
                }
                .into()
            })
    } else if let Some(ref spec_path) = config.spec_path {
        // Single spec mode: only "default" is valid
        if name == "default" {
            Ok(SpecEntry {
                name: "default".to_string(),
                path: spec_path.clone(),
            })
        } else {
            Err(GenerationError::SpecNotFound {
                name: name.to_string(),
                available: vec!["default".to_string()],
            }
            .into())
        }
    } else {
        Err(ConfigError::NoSpecDefined.into())
    }
}

/// Resolves which specs to generate based on CLI flags and config
pub fn resolve_spec_selection(
    config: &Config,
    cli_spec: Option<String>,
    all_specs: bool,
) -> Result<Vec<SpecEntry>> {
    if all_specs {
        // Generate all specs
        let specs = list_specs(config);
        if specs.is_empty() {
            return Err(ConfigError::NoSpecDefined.into());
        }
        Ok(specs)
    } else if let Some(spec_name) = cli_spec {
        // Generate specific spec by name
        let spec = get_spec_by_name(config, &spec_name)?;
        Ok(vec![spec])
    } else if is_multi_spec_mode(config) {
        // Multi-spec mode but no flag: prompt user
        let specs = list_specs(config);
        if specs.is_empty() {
            return Err(ConfigError::NoSpecDefined.into());
        }

        let spec_names: Vec<String> = specs.iter().map(|s| s.name.clone()).collect();
        let selection = Select::new()
            .with_prompt("Which spec do you want to generate?")
            .items(&spec_names)
            .interact()
            .map_err(|e| GenerationError::InvalidOperation {
                message: format!("Failed to get user selection: {}", e),
            })?;

        let selected_spec = specs.get(selection).ok_or_else(|| {
            GenerationError::InvalidOperation {
                message: "Invalid selection".to_string(),
            }
        })?;

        Ok(vec![selected_spec.clone()])
    } else {
        // Single spec mode: use the single spec
        if let Some(ref spec_path) = config.spec_path {
            Ok(vec![SpecEntry {
                name: "default".to_string(),
                path: spec_path.clone(),
            }])
        } else {
            Err(ConfigError::NoSpecDefined.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_multi_spec_mode() {
        let mut config = Config::default();
        assert!(!is_multi_spec_mode(&config));

        config.spec_path = Some("openapi.json".to_string());
        assert!(!is_multi_spec_mode(&config));

        config.spec_path = None;
        config.specs = Some(vec![SpecEntry {
            name: "auth".to_string(),
            path: "specs/auth.yaml".to_string(),
        }]);
        assert!(is_multi_spec_mode(&config));
    }

    #[test]
    fn test_list_specs_single_mode() {
        let mut config = Config::default();
        config.spec_path = Some("openapi.json".to_string());

        let specs = list_specs(&config);
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].name, "default");
        assert_eq!(specs[0].path, "openapi.json");
    }

    #[test]
    fn test_list_specs_multi_mode() {
        let mut config = Config::default();
        config.specs = Some(vec![
            SpecEntry {
                name: "auth".to_string(),
                path: "specs/auth.yaml".to_string(),
            },
            SpecEntry {
                name: "orders".to_string(),
                path: "specs/orders.json".to_string(),
            },
        ]);

        let specs = list_specs(&config);
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].name, "auth");
        assert_eq!(specs[1].name, "orders");
    }

    #[test]
    fn test_get_spec_by_name_single_mode() {
        let mut config = Config::default();
        config.spec_path = Some("openapi.json".to_string());

        let spec = get_spec_by_name(&config, "default").unwrap();
        assert_eq!(spec.name, "default");
        assert_eq!(spec.path, "openapi.json");

        let result = get_spec_by_name(&config, "auth");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_spec_by_name_multi_mode() {
        let mut config = Config::default();
        config.specs = Some(vec![
            SpecEntry {
                name: "auth".to_string(),
                path: "specs/auth.yaml".to_string(),
            },
            SpecEntry {
                name: "orders".to_string(),
                path: "specs/orders.json".to_string(),
            },
        ]);

        let spec = get_spec_by_name(&config, "auth").unwrap();
        assert_eq!(spec.name, "auth");
        assert_eq!(spec.path, "specs/auth.yaml");

        let result = get_spec_by_name(&config, "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_spec_selection_all_specs() {
        let mut config = Config::default();
        config.specs = Some(vec![
            SpecEntry {
                name: "auth".to_string(),
                path: "specs/auth.yaml".to_string(),
            },
            SpecEntry {
                name: "orders".to_string(),
                path: "specs/orders.json".to_string(),
            },
        ]);

        let specs = resolve_spec_selection(&config, None, true).unwrap();
        assert_eq!(specs.len(), 2);
    }

    #[test]
    fn test_resolve_spec_selection_specific_spec() {
        let mut config = Config::default();
        config.specs = Some(vec![
            SpecEntry {
                name: "auth".to_string(),
                path: "specs/auth.yaml".to_string(),
            },
            SpecEntry {
                name: "orders".to_string(),
                path: "specs/orders.json".to_string(),
            },
        ]);

        let specs = resolve_spec_selection(&config, Some("auth".to_string()), false).unwrap();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].name, "auth");
    }

    #[test]
    fn test_resolve_spec_selection_single_mode() {
        let mut config = Config::default();
        config.spec_path = Some("openapi.json".to_string());

        let specs = resolve_spec_selection(&config, None, false).unwrap();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].name, "default");
    }
}

