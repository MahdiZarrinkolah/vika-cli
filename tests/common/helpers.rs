use std::path::Path;
use tempfile::TempDir;
use vika_cli::config::model::Config;

// Test helpers - allow dead_code as they're utilities for future tests

/// Create a temporary directory for testing
#[allow(dead_code)]
pub fn create_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// Assert file contents match expected
#[allow(dead_code)]
pub fn assert_file_contents(path: &Path, expected: &str) {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Failed to read file: {}", path.display()));
    assert_eq!(
        content.trim(),
        expected.trim(),
        "File contents don't match for {}",
        path.display()
    );
}

/// Create a mock config for testing
#[allow(dead_code)]
pub fn create_mock_config() -> Config {
    Config::default()
}

/// Create a mock config with custom paths
#[allow(dead_code)]
pub fn create_mock_config_with_paths(schemas_output: &str, apis_output: &str) -> Config {
    Config {
        root_dir: "src".to_string(),
        schemas: vika_cli::config::model::SchemasConfig {
            output: schemas_output.to_string(),
            naming: "PascalCase".to_string(),
        },
        apis: vika_cli::config::model::ApisConfig {
            output: apis_output.to_string(),
            style: "fetch".to_string(),
            base_url: None,
            header_strategy: "consumerInjected".to_string(),
        },
        modules: vika_cli::config::model::ModulesConfig {
            ignore: vec![],
            selected: vec![],
        },
        generation: vika_cli::config::model::GenerationConfig::default(),
        spec_path: None,
        specs: None,
        schema: vika_cli::config::model::default_schema(),
    }
}

/// Setup test environment (create temp dir, etc.)
pub fn setup_test_env() -> TempDir {
    create_temp_dir()
}

/// Cleanup test environment
#[allow(dead_code)]
pub fn cleanup_test_env(_dir: TempDir) {
    // TempDir automatically cleans up on drop
}

/// Assert error type matches expected
#[allow(dead_code)]
pub fn assert_error_type<T: std::fmt::Display>(
    result: Result<T, vika_cli::error::VikaError>,
    expected_error: &str,
) {
    match result {
        Ok(_) => panic!("Expected error but got Ok"),
        Err(e) => {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains(expected_error),
                "Expected error to contain '{}', but got: {}",
                expected_error,
                error_msg
            );
        }
    }
}

/// Load a test OpenAPI spec from JSON string
#[allow(dead_code)]
pub fn load_test_spec_from_json(json: &str) -> openapiv3::OpenAPI {
    serde_json::from_str(json).expect("Failed to parse test spec")
}

/// Create a minimal OpenAPI spec JSON string
#[allow(dead_code)]
pub fn minimal_spec_json() -> &'static str {
    r#"
    {
        "openapi": "3.0.0",
        "info": {
            "title": "Test API",
            "version": "1.0.0"
        },
        "paths": {}
    }
    "#
}
