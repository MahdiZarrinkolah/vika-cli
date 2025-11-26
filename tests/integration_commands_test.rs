use std::env;
use std::fs;
use tempfile::TempDir;
use vika_cli::commands::{inspect, update};
use vika_cli::config::loader::save_config;
use vika_cli::config::model::{ApisConfig, Config, ModulesConfig, SchemasConfig, SpecEntry};

// Init command requires interactive input, so we test config loading/saving separately

#[tokio::test]
async fn test_generate_command_with_local_spec() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();

    // Create a minimal spec
    let spec_content = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths:
  /test:
    get:
      tags:
        - test
      responses:
        '200':
          description: Success
"#;

    let spec_path = temp_dir.path().join("spec.yaml");
    fs::write(&spec_path, spec_content).unwrap();

    // Create config
    let config = Config::default();
    save_config(&config).unwrap();

    // Generate command requires interactive module selection
    // We can't test it fully without mocking, but we verify the spec exists
    assert!(spec_path.exists());

    env::set_current_dir(original_dir).unwrap();
}

#[tokio::test]
async fn test_inspect_command() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();

    let spec_content = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths:
  /test:
    get:
      tags:
        - test
      responses:
        '200':
          description: Success
"#;

    let spec_path = temp_dir.path().join("spec.yaml");
    fs::write(&spec_path, spec_content).unwrap();

    // Create config with spec (inspect command requires config)
    let spec_str = spec_path.to_str().unwrap().to_string();
    let config = Config {
        specs: vec![SpecEntry {
            name: "test".to_string(),
            path: spec_str.clone(),
            schemas: SchemasConfig::default(),
            apis: ApisConfig::default(),
            hooks: None,
            modules: ModulesConfig::default(),
        }],
        ..Default::default()
    };
    save_config(&config).unwrap();

    // Test inspect without module filter
    let result = inspect::run(
        Some(spec_str.clone()),
        false,
        None,
        None,
        false,
        false,
        false,
    )
    .await;
    assert!(result.is_ok());

    // Test inspect with schemas flag
    let result = inspect::run(Some(spec_str), false, None, None, true, false, false).await;
    assert!(result.is_ok());

    env::set_current_dir(original_dir).unwrap();
}

#[tokio::test]
async fn test_update_command() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();

    // Create a spec first
    let spec_content = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
tags:
  - name: test
paths:
  /test:
    get:
      tags:
        - test
      responses:
        '200':
          description: Success
"#;
    let spec_path = temp_dir.path().join("spec.yaml");
    fs::write(&spec_path, spec_content).unwrap();

    // Create config with spec (use relative path since we changed directory)
    let config = Config {
        specs: vec![SpecEntry {
            name: "test".to_string(),
            path: "spec.yaml".to_string(), // Relative path
            schemas: SchemasConfig::default(),
            apis: ApisConfig::default(),
            hooks: None,
            modules: ModulesConfig {
                ignore: vec![],
                selected: vec!["test".to_string()],
            },
        }],
        ..Default::default()
    };
    save_config(&config).unwrap();

    // Update command succeeds even if no operations found (graceful handling)
    let result = update::run().await;
    // Should succeed - update handles missing modules gracefully
    assert!(result.is_ok());

    env::set_current_dir(original_dir).unwrap();
}
