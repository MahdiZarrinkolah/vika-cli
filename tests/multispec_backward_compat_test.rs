use tempfile::TempDir;
use vika_cli::config::loader::{load_config, save_config};
use vika_cli::config::model::Config;
use vika_cli::config::validator::validate_config;
use vika_cli::specs::manager::is_multi_spec_mode;

#[test]
fn test_single_spec_config_validation_still_works() {
    let mut config = Config::default();
    config.spec_path = Some("openapi.json".to_string());

    let result = validate_config(&config);
    assert!(result.is_ok(), "Single spec config should still validate");
    assert!(!is_multi_spec_mode(&config), "Should not be in multi-spec mode");
}

#[test]
fn test_single_spec_config_serialization_backward_compat() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let config = Config {
        spec_path: Some("openapi.json".to_string()),
        ..Config::default()
    };

    save_config(&config).unwrap();
    let loaded = load_config().unwrap();

    assert_eq!(loaded.spec_path, Some("openapi.json".to_string()));
    assert!(loaded.specs.is_none());
    assert!(!is_multi_spec_mode(&loaded));

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_single_spec_output_structure_unchanged() {
    use vika_cli::generator::ts_typings::TypeScriptType;
    use vika_cli::generator::writer::write_schemas_with_options;
    use vika_cli::generator::zod_schema::ZodSchema;

    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("schemas");

    let types = vec![TypeScriptType {
        content: "export interface User { id: string; }".to_string(),
    }];

    let zod_schemas = vec![ZodSchema {
        content: "export const UserSchema = z.object({ id: z.string() });".to_string(),
    }];

    // Write without spec_name (single-spec mode)
    write_schemas_with_options(
        &output_dir,
        "users",
        &types,
        &zod_schemas,
        None, // No spec_name for backward compatibility
        false,
        false,
    )
    .unwrap();

    // Verify structure is: schemas/users/ (not schemas/{spec_name}/users/)
    let expected_dir = output_dir.join("users");
    assert!(expected_dir.exists(), "Expected schemas/users/ directory");

    // Verify no spec-name directory was created
    let spec_dirs: Vec<_> = output_dir
        .read_dir()
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name())
        .collect();

    // Should only have "users" directory, no spec-name directories
    assert_eq!(spec_dirs.len(), 1);
    assert_eq!(spec_dirs[0], "users");
}

#[test]
fn test_default_config_still_works() {
    let config = Config::default();
    
    // Default config should have no spec_path or specs
    assert!(config.spec_path.is_none());
    assert!(config.specs.is_none());
    
    // Validation should fail (no spec defined), but that's expected
    let result = validate_config(&config);
    assert!(result.is_err()); // Expected: no spec defined
}

#[test]
fn test_config_with_only_spec_path() {
    let mut config = Config::default();
    config.spec_path = Some("specs/api.yaml".to_string());
    config.specs = None;

    assert!(!is_multi_spec_mode(&config));
    let result = validate_config(&config);
    assert!(result.is_ok());
}

#[test]
fn test_config_with_only_specs() {
    use vika_cli::config::model::SpecEntry;

    let mut config = Config::default();
    config.spec_path = None;
    config.specs = Some(vec![SpecEntry {
        name: "api".to_string(),
        path: "specs/api.yaml".to_string(),
    }]);

    assert!(is_multi_spec_mode(&config));
    let result = validate_config(&config);
    assert!(result.is_ok());
}

