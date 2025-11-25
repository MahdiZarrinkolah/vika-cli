use vika_cli::config::model::{Config, SpecEntry};
use vika_cli::specs::manager::{
    get_spec_by_name, is_multi_spec_mode, list_specs, resolve_spec_selection,
};

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
    assert_eq!(specs[0].path, "specs/auth.yaml");
    assert_eq!(specs[1].name, "orders");
    assert_eq!(specs[1].path, "specs/orders.json");
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
    assert_eq!(specs[0].name, "auth");
    assert_eq!(specs[1].name, "orders");
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
    assert_eq!(specs[0].path, "specs/auth.yaml");
}

#[test]
fn test_resolve_spec_selection_single_mode() {
    let mut config = Config::default();
    config.spec_path = Some("openapi.json".to_string());

    let specs = resolve_spec_selection(&config, None, false).unwrap();
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].name, "default");
    assert_eq!(specs[0].path, "openapi.json");
}

#[test]
fn test_resolve_spec_selection_nonexistent_spec_fails() {
    let mut config = Config::default();
    config.specs = Some(vec![SpecEntry {
        name: "auth".to_string(),
        path: "specs/auth.yaml".to_string(),
    }]);

    let result = resolve_spec_selection(&config, Some("nonexistent".to_string()), false);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Spec not found"));
}
