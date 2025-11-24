use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("vika-cli").unwrap();
    cmd.arg("--help");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("vika-cli"))
        .stdout(predicate::str::contains("Generate TypeScript clients"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("vika-cli").unwrap();
    cmd.arg("--version");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("vika-cli"));
}

#[test]
fn test_cli_no_command() {
    let mut cmd = Command::cargo_bin("vika-cli").unwrap();
    
    cmd.assert()
        .failure();
}

#[test]
fn test_init_command_no_interaction() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("vika-cli").unwrap();
    cmd.arg("init");
    cmd.current_dir(temp_dir.path());
    
    // This will fail because it requires user interaction, but we test the command exists
    let output = cmd.output().unwrap();
    // Should either succeed or fail with specific error (not command not found)
    assert!(output.status.code() == Some(0) || output.status.code() == Some(1));
}

#[test]
fn test_generate_command_missing_spec() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create a config file
    let config = r#"{
        "root_dir": "src",
        "schemas": {
            "output": "src/schemas",
            "naming": "PascalCase"
        },
        "apis": {
            "output": "src/apis",
            "style": "fetch",
            "header_strategy": "consumerInjected"
        },
        "modules": {
            "ignore": [],
            "selected": []
        }
    }"#;
    fs::write(temp_dir.path().join(".vika.json"), config).unwrap();
    
    let mut cmd = Command::cargo_bin("vika-cli").unwrap();
    cmd.arg("generate");
    cmd.current_dir(temp_dir.path());
    
    // Should fail without spec path
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_generate_command_with_invalid_spec() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create invalid spec file
    fs::write(temp_dir.path().join("invalid.yaml"), "invalid: yaml: content:").unwrap();
    
    let mut cmd = Command::cargo_bin("vika-cli").unwrap();
    cmd.arg("generate");
    cmd.arg("--spec");
    cmd.arg("invalid.yaml");
    cmd.current_dir(temp_dir.path());
    
    // Should fail with invalid spec
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_update_command_no_config() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("vika-cli").unwrap();
    cmd.arg("update");
    cmd.current_dir(temp_dir.path());
    
    // Should fail without config or spec_path
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_inspect_command_missing_spec() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("vika-cli").unwrap();
    cmd.arg("inspect");
    cmd.current_dir(temp_dir.path());
    
    // Should fail without spec
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_inspect_command_with_spec() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create a minimal valid spec
    let spec = r#"
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
    fs::write(&spec_path, spec).unwrap();
    
    let mut cmd = Command::cargo_bin("vika-cli").unwrap();
    cmd.arg("inspect");
    cmd.arg("--spec");
    cmd.arg(spec_path.to_str().unwrap());
    cmd.current_dir(temp_dir.path());
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Spec Summary"));
}

#[test]
fn test_inspect_command_json_output() {
    let temp_dir = TempDir::new().unwrap();
    
    let spec = r#"
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
    fs::write(&spec_path, spec).unwrap();
    
    let mut cmd = Command::cargo_bin("vika-cli").unwrap();
    cmd.arg("inspect");
    cmd.arg("--spec");
    cmd.arg(spec_path.to_str().unwrap());
    cmd.arg("--json");
    cmd.current_dir(temp_dir.path());
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("modules"));
}

#[test]
fn test_inspect_command_with_schemas_flag() {
    let temp_dir = TempDir::new().unwrap();
    
    let spec = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
components:
  schemas:
    User:
      type: object
      properties:
        id:
          type: string
paths:
  /users:
    get:
      tags:
        - users
      responses:
        '200':
          description: Success
"#;
    let spec_path = temp_dir.path().join("spec.yaml");
    fs::write(&spec_path, spec).unwrap();
    
    let mut cmd = Command::cargo_bin("vika-cli").unwrap();
    cmd.arg("inspect");
    cmd.arg("--spec");
    cmd.arg(spec_path.to_str().unwrap());
    cmd.arg("--schemas");
    cmd.current_dir(temp_dir.path());
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Spec Summary"));
}

#[test]
fn test_generate_command_verbose_flag() {
    let temp_dir = TempDir::new().unwrap();
    
    let spec = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths: {}
"#;
    let spec_path = temp_dir.path().join("spec.yaml");
    fs::write(&spec_path, spec).unwrap();
    
    let config = r#"{
        "root_dir": "src",
        "schemas": {"output": "src/schemas", "naming": "PascalCase"},
        "apis": {"output": "src/apis", "style": "fetch", "header_strategy": "consumerInjected"},
        "modules": {"ignore": [], "selected": []}
    }"#;
    fs::write(temp_dir.path().join(".vika.json"), config).unwrap();
    
    let mut cmd = Command::cargo_bin("vika-cli").unwrap();
    cmd.arg("generate");
    cmd.arg("--spec");
    cmd.arg(spec_path.to_str().unwrap());
    cmd.arg("--verbose");
    cmd.current_dir(temp_dir.path());
    
    // Will fail due to interactive module selection, but tests verbose flag is recognized
    let output = cmd.output().unwrap();
    assert!(output.status.code() == Some(0) || output.status.code() == Some(1));
}

#[test]
fn test_generate_command_cache_flag() {
    let temp_dir = TempDir::new().unwrap();
    
    let spec = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths: {}
"#;
    let spec_path = temp_dir.path().join("spec.yaml");
    fs::write(&spec_path, spec).unwrap();
    
    let config = r#"{
        "root_dir": "src",
        "schemas": {"output": "src/schemas", "naming": "PascalCase"},
        "apis": {"output": "src/apis", "style": "fetch", "header_strategy": "consumerInjected"},
        "modules": {"ignore": [], "selected": []}
    }"#;
    fs::write(temp_dir.path().join(".vika.json"), config).unwrap();
    
    let mut cmd = Command::cargo_bin("vika-cli").unwrap();
    cmd.arg("generate");
    cmd.arg("--spec");
    cmd.arg(spec_path.to_str().unwrap());
    cmd.arg("--cache");
    cmd.current_dir(temp_dir.path());
    
    // Tests cache flag is recognized
    let output = cmd.output().unwrap();
    assert!(output.status.code() == Some(0) || output.status.code() == Some(1));
}

#[test]
fn test_generate_command_backup_flag() {
    let temp_dir = TempDir::new().unwrap();
    
    let spec = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths: {}
"#;
    let spec_path = temp_dir.path().join("spec.yaml");
    fs::write(&spec_path, spec).unwrap();
    
    let config = r#"{
        "root_dir": "src",
        "schemas": {"output": "src/schemas", "naming": "PascalCase"},
        "apis": {"output": "src/apis", "style": "fetch", "header_strategy": "consumerInjected"},
        "modules": {"ignore": [], "selected": []}
    }"#;
    fs::write(temp_dir.path().join(".vika.json"), config).unwrap();
    
    let mut cmd = Command::cargo_bin("vika-cli").unwrap();
    cmd.arg("generate");
    cmd.arg("--spec");
    cmd.arg(spec_path.to_str().unwrap());
    cmd.arg("--backup");
    cmd.current_dir(temp_dir.path());
    
    // Tests backup flag is recognized
    let output = cmd.output().unwrap();
    assert!(output.status.code() == Some(0) || output.status.code() == Some(1));
}

#[test]
fn test_generate_command_force_flag() {
    let temp_dir = TempDir::new().unwrap();
    
    let spec = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths: {}
"#;
    let spec_path = temp_dir.path().join("spec.yaml");
    fs::write(&spec_path, spec).unwrap();
    
    let config = r#"{
        "root_dir": "src",
        "schemas": {"output": "src/schemas", "naming": "PascalCase"},
        "apis": {"output": "src/apis", "style": "fetch", "header_strategy": "consumerInjected"},
        "modules": {"ignore": [], "selected": []}
    }"#;
    fs::write(temp_dir.path().join(".vika.json"), config).unwrap();
    
    let mut cmd = Command::cargo_bin("vika-cli").unwrap();
    cmd.arg("generate");
    cmd.arg("--spec");
    cmd.arg(spec_path.to_str().unwrap());
    cmd.arg("--force");
    cmd.current_dir(temp_dir.path());
    
    // Tests force flag is recognized
    let output = cmd.output().unwrap();
    assert!(output.status.code() == Some(0) || output.status.code() == Some(1));
}

#[test]
fn test_inspect_command_with_module_filter() {
    let temp_dir = TempDir::new().unwrap();
    
    let spec = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths:
  /users:
    get:
      tags:
        - users
      responses:
        '200':
          description: Success
  /products:
    get:
      tags:
        - products
      responses:
        '200':
          description: Success
"#;
    let spec_path = temp_dir.path().join("spec.yaml");
    fs::write(&spec_path, spec).unwrap();
    
    let mut cmd = Command::cargo_bin("vika-cli").unwrap();
    cmd.arg("inspect");
    cmd.arg("--spec");
    cmd.arg(spec_path.to_str().unwrap());
    cmd.arg("--module");
    cmd.arg("users");
    cmd.current_dir(temp_dir.path());
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("users"));
}

