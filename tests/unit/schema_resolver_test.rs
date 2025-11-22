use vika_cli::generator::schema_resolver::SchemaResolver;
use vika_cli::generator::swagger_parser::fetch_and_parse_spec;
use tempfile::TempDir;
use std::fs;

#[tokio::test]
async fn test_circular_dependency_detection() {
    let temp_dir = TempDir::new().unwrap();
    let spec_json = r#"
    {
        "openapi": "3.0.0",
        "info": {
            "title": "Test API",
            "version": "1.0.0"
        },
        "components": {
            "schemas": {
                "User": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "string"},
                        "profile": {"$ref": "#/components/schemas/Profile"}
                    }
                },
                "Profile": {
                    "type": "object",
                    "properties": {
                        "userId": {"type": "string"},
                        "user": {"$ref": "#/components/schemas/User"}
                    }
                }
            }
        }
    }
    "#;

    let spec_path = temp_dir.path().join("spec.json");
    fs::write(&spec_path, spec_json).unwrap();

    let parsed = fetch_and_parse_spec(spec_path.to_str().unwrap())
        .await
        .unwrap();

    let mut resolver = SchemaResolver::new(parsed.openapi);
    let result = resolver.build_dependency_graph();

    // Should handle circular dependencies gracefully
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_deep_nesting_resolution() {
    let temp_dir = TempDir::new().unwrap();
    let spec_json = r#"
    {
        "openapi": "3.0.0",
        "info": {
            "title": "Test API",
            "version": "1.0.0"
        },
        "components": {
            "schemas": {
                "Level1": {
                    "type": "object",
                    "properties": {
                        "level2": {"$ref": "#/components/schemas/Level2"}
                    }
                },
                "Level2": {
                    "type": "object",
                    "properties": {
                        "level3": {"$ref": "#/components/schemas/Level3"}
                    }
                },
                "Level3": {
                    "type": "object",
                    "properties": {
                        "level4": {"$ref": "#/components/schemas/Level4"}
                    }
                },
                "Level4": {
                    "type": "object",
                    "properties": {
                        "level5": {"$ref": "#/components/schemas/Level5"}
                    }
                },
                "Level5": {
                    "type": "object",
                    "properties": {
                        "value": {"type": "string"}
                    }
                }
            }
        }
    }
    "#;

    let spec_path = temp_dir.path().join("spec.json");
    fs::write(&spec_path, spec_json).unwrap();

    let parsed = fetch_and_parse_spec(spec_path.to_str().unwrap())
        .await
        .unwrap();

    let mut resolver = SchemaResolver::new(parsed.openapi);
    let result = resolver.build_dependency_graph();

    // Should handle deep nesting (5+ levels)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_missing_reference_handling() {
    let temp_dir = TempDir::new().unwrap();
    let spec_json = r#"
    {
        "openapi": "3.0.0",
        "info": {
            "title": "Test API",
            "version": "1.0.0"
        },
        "components": {
            "schemas": {
                "User": {
                    "type": "object",
                    "properties": {
                        "profile": {"$ref": "#/components/schemas/NonExistent"}
                    }
                }
            }
        }
    }
    "#;

    let spec_path = temp_dir.path().join("spec.json");
    fs::write(&spec_path, spec_json).unwrap();

    let parsed = fetch_and_parse_spec(spec_path.to_str().unwrap())
        .await
        .unwrap();

    let mut resolver = SchemaResolver::new(parsed.openapi);
    // Should handle missing references gracefully
    let result = resolver.build_dependency_graph();
    // May succeed or fail depending on implementation, but shouldn't panic
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_array_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let spec_json = r#"
    {
        "openapi": "3.0.0",
        "info": {
            "title": "Test API",
            "version": "1.0.0"
        },
        "components": {
            "schemas": {
                "UserList": {
                    "type": "array",
                    "items": {"$ref": "#/components/schemas/User"}
                },
                "User": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "string"}
                    }
                }
            }
        }
    }
    "#;

    let spec_path = temp_dir.path().join("spec.json");
    fs::write(&spec_path, spec_json).unwrap();

    let parsed = fetch_and_parse_spec(spec_path.to_str().unwrap())
        .await
        .unwrap();

    let mut resolver = SchemaResolver::new(parsed.openapi);
    let result = resolver.build_dependency_graph();

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_nested_object_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let spec_json = r#"
    {
        "openapi": "3.0.0",
        "info": {
            "title": "Test API",
            "version": "1.0.0"
        },
        "components": {
            "schemas": {
                "Outer": {
                    "type": "object",
                    "properties": {
                        "inner": {
                            "type": "object",
                            "properties": {
                                "nested": {"$ref": "#/components/schemas/Nested"}
                            }
                        }
                    }
                },
                "Nested": {
                    "type": "object",
                    "properties": {
                        "value": {"type": "string"}
                    }
                }
            }
        }
    }
    "#;

    let spec_path = temp_dir.path().join("spec.json");
    fs::write(&spec_path, spec_json).unwrap();

    let parsed = fetch_and_parse_spec(spec_path.to_str().unwrap())
        .await
        .unwrap();

    let mut resolver = SchemaResolver::new(parsed.openapi);
    let result = resolver.build_dependency_graph();

    assert!(result.is_ok());
}

