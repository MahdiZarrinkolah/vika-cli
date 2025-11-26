use tempfile::TempDir;
use vika_cli::config::loader::{load_config, save_config};
use vika_cli::config::model::{Config, SpecEntry};
use vika_cli::config::validator::validate_config;

#[test]
fn test_single_spec_config_validation_still_works() {
    let config = Config {
        specs: vec![SpecEntry {
            name: "default".to_string(),
            path: "openapi.json".to_string(),
            schemas: vika_cli::config::model::SchemasConfig::default(),
            apis: vika_cli::config::model::ApisConfig::default(),
            hooks: None,
            modules: vika_cli::config::model::ModulesConfig::default(),
        }],
        ..Default::default()
    };

    let result = validate_config(&config);
    assert!(result.is_ok(), "Single spec config should still validate");
}

#[test]
fn test_single_spec_config_serialization() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let config = Config {
        specs: vec![SpecEntry {
            name: "default".to_string(),
            path: "openapi.json".to_string(),
            schemas: vika_cli::config::model::SchemasConfig::default(),
            apis: vika_cli::config::model::ApisConfig::default(),
            hooks: None,
            modules: vika_cli::config::model::ModulesConfig::default(),
        }],
        ..Config::default()
    };

    save_config(&config).unwrap();
    let loaded = load_config().unwrap();

    assert_eq!(loaded.specs.len(), 1);
    assert_eq!(loaded.specs[0].name, "default");
    assert_eq!(loaded.specs[0].path, "openapi.json");

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

    // Default config should have no specs
    assert!(config.specs.is_empty());

    // Validation should fail (no spec defined), but that's expected
    let result = validate_config(&config);
    assert!(result.is_err()); // Expected: no spec defined
}

#[test]
fn test_config_with_single_spec() {
    let config = Config {
        specs: vec![SpecEntry {
            name: "api".to_string(),
            path: "specs/api.yaml".to_string(),
            schemas: vika_cli::config::model::SchemasConfig::default(),
            apis: vika_cli::config::model::ApisConfig::default(),
            hooks: None,
            modules: vika_cli::config::model::ModulesConfig::default(),
        }],
        ..Default::default()
    };

    let result = validate_config(&config);
    assert!(result.is_ok());
}

#[test]
fn test_config_with_multiple_specs() {
    let mut config = Config::default();
    config.specs = vec![
        SpecEntry {
            name: "api".to_string(),
            path: "specs/api.yaml".to_string(),
            schemas: vika_cli::config::model::SchemasConfig::default(),
            apis: vika_cli::config::model::ApisConfig::default(),
            hooks: None,
            modules: vika_cli::config::model::ModulesConfig::default(),
        },
        SpecEntry {
            name: "auth".to_string(),
            path: "specs/auth.yaml".to_string(),
            schemas: vika_cli::config::model::SchemasConfig::default(),
            apis: vika_cli::config::model::ApisConfig::default(),
            hooks: None,
            modules: vika_cli::config::model::ModulesConfig::default(),
        },
    ];

    let result = validate_config(&config);
    assert!(result.is_ok());
}
