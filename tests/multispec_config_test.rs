use vika_cli::config::model::{Config, SpecEntry};
use vika_cli::config::validator::validate_config;

#[test]
fn test_multi_spec_config_deserialization() {
    let json = r#"
    {
        "$schema": "https://example.com/schema.json",
        "specs": [
            { "name": "auth", "path": "specs/auth.yaml" },
            { "name": "orders", "path": "specs/orders.json" },
            { "name": "products", "path": "specs/products.yaml" }
        ]
    }
    "#;

    let config: Config = serde_json::from_str(json).unwrap();
    assert!(config.specs.is_some());
    assert!(config.spec_path.is_none());
    let specs = config.specs.unwrap();
    assert_eq!(specs.len(), 3);
    assert_eq!(specs[0].name, "auth");
    assert_eq!(specs[0].path, "specs/auth.yaml");
    assert_eq!(specs[1].name, "orders");
    assert_eq!(specs[2].name, "products");
}

#[test]
fn test_single_spec_config_still_works() {
    let json = r#"
    {
        "$schema": "https://example.com/schema.json",
        "spec_path": "openapi.json"
    }
    "#;

    let config: Config = serde_json::from_str(json).unwrap();
    assert!(config.spec_path.is_some());
    assert!(config.specs.is_none());
    assert_eq!(config.spec_path.unwrap(), "openapi.json");
}

#[test]
fn test_validate_config_both_spec_and_specs_fails() {
    let mut config = Config::default();
    config.spec_path = Some("openapi.json".to_string());
    config.specs = Some(vec![SpecEntry {
        name: "auth".to_string(),
        path: "specs/auth.yaml".to_string(),
    }]);

    let result = validate_config(&config);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Both 'spec_path' and 'specs'"));
}

#[test]
fn test_validate_config_no_spec_defined_fails() {
    let config = Config::default();
    let result = validate_config(&config);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Neither 'spec_path' nor 'specs'"));
}

#[test]
fn test_validate_config_empty_specs_array_fails() {
    let mut config = Config::default();
    config.specs = Some(vec![]);

    let result = validate_config(&config);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("At least one spec must be defined"));
}

#[test]
fn test_validate_config_duplicate_spec_names_fails() {
    let mut config = Config::default();
    config.specs = Some(vec![
        SpecEntry {
            name: "auth".to_string(),
            path: "specs/auth.yaml".to_string(),
        },
        SpecEntry {
            name: "auth".to_string(),
            path: "specs/auth2.yaml".to_string(),
        },
    ]);

    let result = validate_config(&config);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Duplicate spec name"));
}

#[test]
fn test_validate_config_invalid_spec_name_with_space_fails() {
    let mut config = Config::default();
    config.specs = Some(vec![SpecEntry {
        name: "invalid name".to_string(),
        path: "specs/auth.yaml".to_string(),
    }]);

    let result = validate_config(&config);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Invalid spec name"));
}

#[test]
fn test_validate_config_invalid_spec_name_with_special_chars_fails() {
    let mut config = Config::default();
    config.specs = Some(vec![SpecEntry {
        name: "invalid@name".to_string(),
        path: "specs/auth.yaml".to_string(),
    }]);

    let result = validate_config(&config);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Invalid spec name"));
}

#[test]
fn test_validate_config_empty_spec_name_fails() {
    let mut config = Config::default();
    config.specs = Some(vec![SpecEntry {
        name: "".to_string(),
        path: "specs/auth.yaml".to_string(),
    }]);

    let result = validate_config(&config);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Invalid spec name"));
}

#[test]
fn test_validate_config_empty_spec_path_fails() {
    let mut config = Config::default();
    config.specs = Some(vec![SpecEntry {
        name: "auth".to_string(),
        path: "".to_string(),
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
        SpecEntry {
            name: "auth".to_string(),
            path: "specs/auth.yaml".to_string(),
        },
        SpecEntry {
            name: "orders".to_string(),
            path: "specs/orders.json".to_string(),
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

#[test]
fn test_validate_config_valid_spec_names() {
    let mut config = Config::default();
    config.specs = Some(vec![
        SpecEntry {
            name: "auth-service".to_string(),
            path: "specs/auth.yaml".to_string(),
        },
        SpecEntry {
            name: "orders_service".to_string(),
            path: "specs/orders.json".to_string(),
        },
        SpecEntry {
            name: "products123".to_string(),
            path: "specs/products.yaml".to_string(),
        },
    ]);

    let result = validate_config(&config);
    assert!(result.is_ok());
}

