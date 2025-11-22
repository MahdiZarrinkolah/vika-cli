use insta::assert_snapshot;
use vika_cli::generator::ts_typings::generate_typings;
use vika_cli::generator::swagger_parser::fetch_and_parse_spec;
use tempfile::TempDir;
use std::fs;

#[tokio::test]
async fn test_complex_nested_objects() {
    let temp_dir = TempDir::new().unwrap();
    let spec_json = r#"
    {
        "openapi": "3.0.0",
        "info": {"title": "Test", "version": "1.0.0"},
        "paths": {},
        "components": {
            "schemas": {
                "ComplexObject": {
                    "type": "object",
                    "properties": {
                        "nested": {
                            "type": "object",
                            "properties": {
                                "deep": {
                                    "type": "object",
                                    "properties": {
                                        "value": {"type": "string"}
                                    }
                                }
                            }
                        }
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

    let types = generate_typings(&parsed.openapi, &parsed.schemas, &["ComplexObject".to_string()])
        .unwrap();

    let output: String = types.iter().map(|t| t.content.clone()).collect::<Vec<_>>().join("\n\n");
    assert_snapshot!("complex_nested_objects", output);
}

#[tokio::test]
async fn test_union_types() {
    let temp_dir = TempDir::new().unwrap();
    let spec_json = r#"
    {
        "openapi": "3.0.0",
        "info": {"title": "Test", "version": "1.0.0"},
        "paths": {},
        "components": {
            "schemas": {
                "UnionType": {
                    "oneOf": [
                        {"type": "string"},
                        {"type": "number"}
                    ]
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

    let types = generate_typings(&parsed.openapi, &parsed.schemas, &["UnionType".to_string()])
        .unwrap();

    let output: String = types.iter().map(|t| t.content.clone()).collect::<Vec<_>>().join("\n\n");
    assert_snapshot!("union_types", output);
}

#[tokio::test]
async fn test_enum_types() {
    let temp_dir = TempDir::new().unwrap();
    let spec_json = r#"
    {
        "openapi": "3.0.0",
        "info": {"title": "Test", "version": "1.0.0"},
        "paths": {},
        "components": {
            "schemas": {
                "StatusEnum": {
                    "type": "string",
                    "enum": ["active", "inactive", "pending"]
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

    let types = generate_typings(&parsed.openapi, &parsed.schemas, &["StatusEnum".to_string()])
        .unwrap();

    let output: String = types.iter().map(|t| t.content.clone()).collect::<Vec<_>>().join("\n\n");
    assert_snapshot!("enum_types", output);
}

#[tokio::test]
async fn test_optional_vs_required_fields() {
    let temp_dir = TempDir::new().unwrap();
    let spec_json = r#"
    {
        "openapi": "3.0.0",
        "info": {"title": "Test", "version": "1.0.0"},
        "paths": {},
        "components": {
            "schemas": {
                "User": {
                    "type": "object",
                    "required": ["id", "name"],
                    "properties": {
                        "id": {"type": "string"},
                        "name": {"type": "string"},
                        "email": {"type": "string"},
                        "age": {"type": "number"}
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

    let types = generate_typings(&parsed.openapi, &parsed.schemas, &["User".to_string()])
        .unwrap();

    let output: String = types.iter().map(|t| t.content.clone()).collect::<Vec<_>>().join("\n\n");
    assert_snapshot!("optional_required_fields", output);
}

#[tokio::test]
async fn test_array_types() {
    let temp_dir = TempDir::new().unwrap();
    let spec_json = r#"
    {
        "openapi": "3.0.0",
        "info": {"title": "Test", "version": "1.0.0"},
        "paths": {},
        "components": {
            "schemas": {
                "NestedArray": {
                    "type": "array",
                    "items": {
                        "type": "array",
                        "items": {"type": "string"}
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

    let types = generate_typings(&parsed.openapi, &parsed.schemas, &["NestedArray".to_string()])
        .unwrap();

    let output: String = types.iter().map(|t| t.content.clone()).collect::<Vec<_>>().join("\n\n");
    assert_snapshot!("array_types", output);
}

#[tokio::test]
async fn test_allof_schemas() {
    let temp_dir = TempDir::new().unwrap();
    let spec_json = r##"
    {
        "openapi": "3.0.0",
        "info": {"title": "Test", "version": "1.0.0"},
        "paths": {},
        "components": {
            "schemas": {
                "Base": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "string"}
                    }
                },
                "Extended": {
                    "allOf": [
                        {"$ref": "#/components/schemas/Base"},
                        {
                            "type": "object",
                            "properties": {
                                "name": {"type": "string"}
                            }
                        }
                    ]
                }
            }
        }
    }
    "##;

    let spec_path = temp_dir.path().join("spec.json");
    fs::write(&spec_path, spec_json).unwrap();

    let parsed = fetch_and_parse_spec(spec_path.to_str().unwrap())
        .await
        .unwrap();

    let types = generate_typings(&parsed.openapi, &parsed.schemas, &["Base".to_string(), "Extended".to_string()])
        .unwrap();

    let output: String = types.iter().map(|t| t.content.clone()).collect::<Vec<_>>().join("\n\n");
    assert_snapshot!("allof_schemas", output);
}

