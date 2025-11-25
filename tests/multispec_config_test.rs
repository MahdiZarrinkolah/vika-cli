use vika_cli::config::model::{ApisConfig, Config, ModulesConfig, SchemasConfig, SpecEntry};
use vika_cli::config::validator::validate_config;

fn default_spec_entry(name: &str, path: &str) -> SpecEntry {
    SpecEntry {
        name: name.to_string(),
        path: path.to_string(),
        schemas: SchemasConfig::default(),
        apis: ApisConfig::default(),
        modules: ModulesConfig::default(),
    }
}

#[test]
fn test_multi_spec_config_deserialization() {
    let json = r#"
    {
        "$schema": "https://example.com/schema.json",
        "specs": [
            { 
                "name": "auth", 
                "path": "specs/auth.yaml",
                "schemas": {},
                "apis": {},
                "modules": {}
            },
            { 
                "name": "orders", 
                "path": "specs/orders.json",
                "schemas": {},
                "apis": {},
                "modules": {}
            },
            { 
                "name": "products", 
                "path": "specs/products.yaml",
                "schemas": {},
                "apis": {},
                "modules": {}
            }
        ]
    }
    "#;

    let config: Config = serde_json::from_str(json).unwrap();
    assert_eq!(config.specs.len(), 3);
    assert_eq!(config.specs[0].name, "auth");
    assert_eq!(config.specs[0].path, "specs/auth.yaml");
    assert_eq!(config.specs[1].name, "orders");
    assert_eq!(config.specs[2].name, "products");
}

#[test]
fn test_single_spec_config_still_works() {
    let json = r#"
    {
        "$schema": "https://example.com/schema.json",
        "specs": [
            {
                "name": "default",
                "path": "openapi.json",
                "schemas": {},
                "apis": {},
                "modules": {}
            }
        ]
    }
    "#;

    let config: Config = serde_json::from_str(json).unwrap();
    assert_eq!(config.specs.len(), 1);
    assert_eq!(config.specs[0].name, "default");
    assert_eq!(config.specs[0].path, "openapi.json");
}

#[test]
fn test_validate_config_no_spec_defined_fails() {
    let config = Config::default();
    let result = validate_config(&config);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("No specs are defined"));
}

#[test]
fn test_validate_config_empty_specs_array_fails() {
    let mut config = Config::default();
    config.specs = vec![];

    let result = validate_config(&config);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("No specs are defined"));
}

#[test]
fn test_validate_config_duplicate_spec_names_fails() {
    let mut config = Config::default();
    config.specs = vec![
        default_spec_entry("auth", "specs/auth.yaml"),
        default_spec_entry("auth", "specs/auth2.yaml"),
    ];

    let result = validate_config(&config);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Duplicate spec name"));
}

#[test]
fn test_validate_config_invalid_spec_name_with_space_fails() {
    let mut config = Config::default();
    config.specs = vec![default_spec_entry("invalid name", "specs/auth.yaml")];

    let result = validate_config(&config);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Invalid spec name"));
}

#[test]
fn test_validate_config_invalid_spec_name_with_special_chars_fails() {
    let mut config = Config::default();
    config.specs = vec![default_spec_entry("invalid@name", "specs/auth.yaml")];

    let result = validate_config(&config);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Invalid spec name"));
}

#[test]
fn test_validate_config_empty_spec_name_fails() {
    let mut config = Config::default();
    config.specs = vec![default_spec_entry("", "specs/auth.yaml")];

    let result = validate_config(&config);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Invalid spec name"));
}

#[test]
fn test_validate_config_empty_spec_path_fails() {
    let mut config = Config::default();
    config.specs = vec![default_spec_entry("auth", "")];

    let result = validate_config(&config);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("empty path"));
}

#[test]
fn test_validate_config_valid_multi_spec() {
    let mut config = Config::default();
    config.specs = vec![
        default_spec_entry("auth", "specs/auth.yaml"),
        default_spec_entry("orders", "specs/orders.json"),
    ];

    let result = validate_config(&config);
    assert!(result.is_ok());
}

#[test]
fn test_validate_config_valid_single_spec() {
    let mut config = Config::default();
    config.specs = vec![default_spec_entry("default", "openapi.json")];

    let result = validate_config(&config);
    assert!(result.is_ok());
}

#[test]
fn test_validate_config_valid_spec_names() {
    let mut config = Config::default();
    config.specs = vec![
        default_spec_entry("auth-service", "specs/auth.yaml"),
        default_spec_entry("orders_service", "specs/orders.json"),
        default_spec_entry("products123", "specs/products.yaml"),
    ];

    let result = validate_config(&config);
    assert!(result.is_ok());
}
