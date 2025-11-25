use vika_cli::config::model::{ApisConfig, Config, ModulesConfig, SchemasConfig, SpecEntry};
use vika_cli::specs::manager::{get_spec_by_name, list_specs, resolve_spec_selection};

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
fn test_list_specs_single_mode() {
    let mut config = Config::default();
    config.specs = vec![default_spec_entry("default", "openapi.json")];

    let specs = list_specs(&config);
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].name, "default");
    assert_eq!(specs[0].path, "openapi.json");
}

#[test]
fn test_list_specs_multi_mode() {
    let mut config = Config::default();
    config.specs = vec![
        default_spec_entry("auth", "specs/auth.yaml"),
        default_spec_entry("orders", "specs/orders.json"),
    ];

    let specs = list_specs(&config);
    assert_eq!(specs.len(), 2);
    assert_eq!(specs[0].name, "auth");
    assert_eq!(specs[0].path, "specs/auth.yaml");
    assert_eq!(specs[1].name, "orders");
    assert_eq!(specs[1].path, "specs/orders.json");
}

#[test]
fn test_get_spec_by_name_single_mode() {
    let mut config = Config::default();
    config.specs = vec![default_spec_entry("default", "openapi.json")];

    let spec = get_spec_by_name(&config, "default").unwrap();
    assert_eq!(spec.name, "default");
    assert_eq!(spec.path, "openapi.json");

    let result = get_spec_by_name(&config, "auth");
    assert!(result.is_err());
}

#[test]
fn test_get_spec_by_name_multi_mode() {
    let mut config = Config::default();
    config.specs = vec![
        default_spec_entry("auth", "specs/auth.yaml"),
        default_spec_entry("orders", "specs/orders.json"),
    ];

    let spec = get_spec_by_name(&config, "auth").unwrap();
    assert_eq!(spec.name, "auth");
    assert_eq!(spec.path, "specs/auth.yaml");

    let spec2 = get_spec_by_name(&config, "orders").unwrap();
    assert_eq!(spec2.name, "orders");
    assert_eq!(spec2.path, "specs/orders.json");

    let result = get_spec_by_name(&config, "nonexistent");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Spec not found"));
}

#[test]
fn test_resolve_spec_selection_all_specs() {
    let mut config = Config::default();
    config.specs = vec![
        default_spec_entry("auth", "specs/auth.yaml"),
        default_spec_entry("orders", "specs/orders.json"),
    ];

    let specs = resolve_spec_selection(&config, None, true).unwrap();
    assert_eq!(specs.len(), 2);
    assert_eq!(specs[0].name, "auth");
    assert_eq!(specs[1].name, "orders");
}

#[test]
fn test_resolve_spec_selection_specific_spec() {
    let mut config = Config::default();
    config.specs = vec![
        default_spec_entry("auth", "specs/auth.yaml"),
        default_spec_entry("orders", "specs/orders.json"),
    ];

    let specs = resolve_spec_selection(&config, Some("auth".to_string()), false).unwrap();
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].name, "auth");
    assert_eq!(specs[0].path, "specs/auth.yaml");
}

#[test]
fn test_resolve_spec_selection_single_mode() {
    let mut config = Config::default();
    config.specs = vec![default_spec_entry("default", "openapi.json")];

    let specs = resolve_spec_selection(&config, None, false).unwrap();
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].name, "default");
    assert_eq!(specs[0].path, "openapi.json");
}

#[test]
fn test_resolve_spec_selection_nonexistent_spec_fails() {
    let mut config = Config::default();
    config.specs = vec![default_spec_entry("auth", "specs/auth.yaml")];

    let result = resolve_spec_selection(&config, Some("nonexistent".to_string()), false);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Spec not found"));
}
