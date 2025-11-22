use std::path::{Path, PathBuf};
use tempfile::TempDir;
use vika_cli::config::model::Config;

/// Create a temporary directory for testing
pub fn create_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// Assert file contents match expected
pub fn assert_file_contents(path: &Path, expected: &str) {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Failed to read file: {}", path.display()));
    assert_eq!(content.trim(), expected.trim(), "File contents don't match for {}", path.display());
}

/// Create a mock config for testing
pub fn create_mock_config() -> Config {
    Config::default()
}

/// Create a mock config with custom paths
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
        spec_path: None,
        schema: vika_cli::config::model::default_schema(),
    }
}

/// Setup test environment (create temp dir, etc.)
pub fn setup_test_env() -> TempDir {
    create_temp_dir()
}

/// Cleanup test environment
pub fn cleanup_test_env(_dir: TempDir) {
    // TempDir automatically cleans up on drop
}

/// Assert error type matches expected
pub fn assert_error_type<T: std::fmt::Display>(result: Result<T, vika_cli::error::VikaError>, expected_error: &str) {
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
pub fn load_test_spec_from_json(json: &str) -> openapiv3::OpenAPI {
    serde_json::from_str(json).expect("Failed to parse test spec")
}

/// Create a minimal OpenAPI spec JSON string
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

