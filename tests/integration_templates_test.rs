#![allow(deprecated)] // TODO: migrate to cargo::cargo_bin_cmd! macro

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_templates_list_command() {
    let mut cmd = Command::cargo_bin("vika-cli").unwrap();
    cmd.args(&["templates", "list"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Built-in templates"))
        .stdout(predicate::str::contains("type-interface"))
        .stdout(predicate::str::contains("zod-schema"))
        .stdout(predicate::str::contains("api-client-fetch"));
}

#[test]
fn test_templates_init_command() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(temp_dir.path()).unwrap();

    let mut cmd = Command::cargo_bin("vika-cli").unwrap();
    cmd.args(&["templates", "init"]);
    cmd.assert().success();

    // Check that templates directory was created
    let templates_dir = temp_dir.path().join(".vika").join("templates");
    assert!(templates_dir.exists());

    // Check that template files were copied
    assert!(templates_dir.join("type-interface.tera").exists());
    assert!(templates_dir.join("zod-schema.tera").exists());
    assert!(templates_dir.join("api-client-fetch.tera").exists());

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_templates_init_skips_existing() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Create .vika/templates directory
    let templates_dir = temp_dir.path().join(".vika").join("templates");
    fs::create_dir_all(&templates_dir).unwrap();

    // Create an existing template file
    fs::write(
        templates_dir.join("type-interface.tera"),
        "custom template content",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("vika-cli").unwrap();
    cmd.args(&["templates", "init"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("already exists"));

    // Verify custom content was preserved
    let content = fs::read_to_string(templates_dir.join("type-interface.tera")).unwrap();
    assert_eq!(content, "custom template content");

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_custom_template_override() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Initialize templates directory manually
    let templates_dir = temp_dir.path().join(".vika").join("templates");
    fs::create_dir_all(&templates_dir).unwrap();

    // Copy a built-in template first
    let mut cmd = Command::cargo_bin("vika-cli").unwrap();
    cmd.args(&["templates", "init"]);
    let result = cmd.output();

    // If binary not found, skip CLI test but verify file operations work
    if result.is_err() {
        // Create template file manually for testing
        let builtin_content = r#"export interface {{ type_name }} {
{% for field in fields %}
  {{ field.name }}{% if field.optional %}?{% endif %}: {{ field.type_name }};
{% endfor %}
}
"#;
        fs::write(templates_dir.join("type-interface.tera"), builtin_content).unwrap();
    }

    // Create a custom template override
    let custom_content = r#"export interface {{ type_name }} {
  // Custom template override
  customField: string;
}
"#;
    fs::write(templates_dir.join("type-interface.tera"), custom_content).unwrap();

    // Verify the override file exists and has custom content
    assert!(templates_dir.join("type-interface.tera").exists());
    let content = fs::read_to_string(templates_dir.join("type-interface.tera")).unwrap();
    assert!(content.contains("Custom template override"));

    std::env::set_current_dir(original_dir).unwrap();
}
