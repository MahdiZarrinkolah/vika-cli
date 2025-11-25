use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use vika_cli::config::loader::{load_config, save_config};
use vika_cli::config::model::{Config, SpecEntry};
use vika_cli::config::validator::validate_config;
use vika_cli::specs::manager::{is_multi_spec_mode, list_specs, resolve_spec_selection};

#[tokio::test]
async fn test_full_multi_spec_generation_flow() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(temp_dir.path()).unwrap();

    // Setup: Copy fixture specs
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/multi-spec");
    let specs_dir = temp_dir.path().join("specs");
    fs::create_dir_all(&specs_dir).unwrap();

    // Copy spec files
    fs::copy(fixtures_dir.join("auth.yaml"), specs_dir.join("auth.yaml")).unwrap();
    fs::copy(
        fixtures_dir.join("orders.json"),
        specs_dir.join("orders.json"),
    )
    .unwrap();

    // Create multi-spec config
    let config = Config {
        specs: Some(vec![
            SpecEntry {
                name: "auth".to_string(),
                path: "specs/auth.yaml".to_string(),
            },
            SpecEntry {
                name: "orders".to_string(),
                path: "specs/orders.json".to_string(),
            },
        ]),
        ..Config::default()
    };

    save_config(&config).unwrap();
    validate_config(&config).unwrap();

    // Verify multi-spec mode
    assert!(is_multi_spec_mode(&config));
    let specs = list_specs(&config);
    assert_eq!(specs.len(), 2);

    // Test spec selection
    let selected = resolve_spec_selection(&config, Some("auth".to_string()), false).unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].name, "auth");

    // Generate for auth spec (non-interactive - we'd need to mock module selection)
    // For now, just verify the config and spec files are set up correctly
    let loaded_config = load_config().unwrap();
    assert!(is_multi_spec_mode(&loaded_config));

    // Verify spec files exist
    assert!(specs_dir.join("auth.yaml").exists());
    assert!(specs_dir.join("orders.json").exists());

    env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_multi_spec_config_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(temp_dir.path()).unwrap();

    let config = Config {
        specs: Some(vec![
            SpecEntry {
                name: "service1".to_string(),
                path: "specs/service1.yaml".to_string(),
            },
            SpecEntry {
                name: "service2".to_string(),
                path: "specs/service2.yaml".to_string(),
            },
            SpecEntry {
                name: "service3".to_string(),
                path: "specs/service3.yaml".to_string(),
            },
        ]),
        ..Config::default()
    };

    save_config(&config).unwrap();
    let loaded = load_config().unwrap();

    assert!(loaded.specs.is_some());
    let specs = loaded.specs.unwrap();
    assert_eq!(specs.len(), 3);
    assert_eq!(specs[0].name, "service1");
    assert_eq!(specs[1].name, "service2");
    assert_eq!(specs[2].name, "service3");

    env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_multi_spec_error_handling() {
    use vika_cli::specs::manager::get_spec_by_name;

    let mut config = Config::default();
    config.specs = Some(vec![SpecEntry {
        name: "auth".to_string(),
        path: "specs/auth.yaml".to_string(),
    }]);

    // Test getting non-existent spec
    let result = get_spec_by_name(&config, "nonexistent");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Spec not found"));

    // Test getting existing spec
    let spec = get_spec_by_name(&config, "auth").unwrap();
    assert_eq!(spec.name, "auth");
    assert_eq!(spec.path, "specs/auth.yaml");
}

#[test]
fn test_multi_spec_directory_structure_verification() {
    use vika_cli::generator::ts_typings::TypeScriptType;
    use vika_cli::generator::writer::write_schemas_with_options;
    use vika_cli::generator::zod_schema::ZodSchema;

    let temp_dir = TempDir::new().unwrap();
    let schemas_dir = temp_dir.path().join("schemas");

    let types = vec![TypeScriptType {
        content: "export interface Test { value: string; }".to_string(),
    }];

    let zod_schemas = vec![ZodSchema {
        content: "export const TestSchema = z.object({ value: z.string() });".to_string(),
    }];

    // Write for multiple specs
    for spec_name in &["auth", "orders", "products"] {
        write_schemas_with_options(
            &schemas_dir,
            "test",
            &types,
            &zod_schemas,
            Some(spec_name),
            false,
            false,
        )
        .unwrap();
    }

    // Verify all spec directories exist
    for spec_name in &["auth", "orders", "products"] {
        let spec_dir = schemas_dir.join(spec_name).join("test");
        assert!(
            spec_dir.exists(),
            "Expected directory for spec {} to exist",
            spec_name
        );

        let types_file = spec_dir.join("types.ts");
        assert!(types_file.exists());
    }

    // Verify they're separate directories
    let auth_dir = schemas_dir.join("auth");
    let orders_dir = schemas_dir.join("orders");
    let products_dir = schemas_dir.join("products");

    assert_ne!(auth_dir, orders_dir);
    assert_ne!(orders_dir, products_dir);
    assert_ne!(auth_dir, products_dir);
}
