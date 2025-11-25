use std::fs;
use tempfile::TempDir;
use vika_cli::config::loader::{load_config, save_config};
use vika_cli::config::model::{ApisConfig, Config, ModulesConfig, SchemasConfig, SpecEntry};
use vika_cli::specs::manager::{list_specs, resolve_spec_selection};

fn default_spec_entry(name: &str, path: &str) -> SpecEntry {
    SpecEntry {
        name: name.to_string(),
        path: path.to_string(),
        schemas: SchemasConfig::default(),
        apis: ApisConfig::default(),
        modules: ModulesConfig::default(),
    }
}

fn create_test_spec(name: &str) -> String {
    format!(
        r#"
{{
    "openapi": "3.0.0",
    "info": {{
        "title": "{} API",
        "version": "1.0.0"
    }},
    "tags": [
        {{
            "name": "{}",
            "description": "{} operations"
        }}
    ],
    "paths": {{
        "/{}": {{
            "get": {{
                "tags": ["{}"],
                "summary": "Get {}",
                "operationId": "get{}",
                "responses": {{
                    "200": {{
                        "description": "Success",
                        "content": {{
                            "application/json": {{
                                "schema": {{
                                    "type": "object",
                                    "properties": {{
                                        "id": {{ "type": "string" }},
                                        "name": {{ "type": "string" }}
                                    }}
                                }}
                            }}
                        }}
                    }}
                }}
            }}
        }}
    }}
}}
"#,
        name, name, name, name, name, name, name
    )
}

#[tokio::test]
async fn test_multi_spec_generation_structure() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Create spec files
    let specs_dir = temp_dir.path().join("specs");
    fs::create_dir_all(&specs_dir).unwrap();

    let auth_spec = create_test_spec("auth");
    fs::write(specs_dir.join("auth.yaml"), &auth_spec).unwrap();

    let orders_spec = create_test_spec("orders");
    fs::write(specs_dir.join("orders.json"), &orders_spec).unwrap();

    // Create multi-spec config
    let config = Config {
        specs: vec![
            default_spec_entry("auth", "specs/auth.yaml"),
            default_spec_entry("orders", "specs/orders.json"),
        ],
        ..Config::default()
    };

    save_config(&config).unwrap();

    // Verify config loads correctly
    let loaded_config = load_config().unwrap();
    let specs = list_specs(&loaded_config);
    assert_eq!(specs.len(), 2);

    // Verify spec selection works
    let selected = resolve_spec_selection(&loaded_config, Some("auth".to_string()), false).unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].name, "auth");

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_multi_spec_config_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let config = Config {
        specs: vec![
            default_spec_entry("auth", "specs/auth.yaml"),
            default_spec_entry("orders", "specs/orders.json"),
        ],
        ..Config::default()
    };

    save_config(&config).unwrap();
    let loaded = load_config().unwrap();

    assert_eq!(loaded.specs.len(), 2);
    assert_eq!(loaded.specs[0].name, "auth");
    assert_eq!(loaded.specs[1].name, "orders");

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_single_spec_config() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let config = Config {
        specs: vec![default_spec_entry("default", "openapi.json")],
        ..Config::default()
    };

    save_config(&config).unwrap();
    let loaded = load_config().unwrap();

    assert_eq!(loaded.specs.len(), 1);
    assert_eq!(loaded.specs[0].name, "default");
    assert_eq!(loaded.specs[0].path, "openapi.json");

    std::env::set_current_dir(original_dir).unwrap();
}
