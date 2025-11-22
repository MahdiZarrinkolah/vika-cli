use insta::assert_snapshot;
use vika_cli::generator::zod_schema::generate_zod_schemas;
use vika_cli::generator::swagger_parser::fetch_and_parse_spec;
use tempfile::TempDir;
use std::fs;

#[tokio::test]
async fn test_zod_validation_rules() {
    let temp_dir = TempDir::new().unwrap();
    let spec_json = r#"
    {
        "openapi": "3.0.0",
        "info": {"title": "Test", "version": "1.0.0"},
        "components": {
            "schemas": {
                "Validated": {
                    "type": "object",
                    "properties": {
                        "email": {
                            "type": "string",
                            "format": "email"
                        },
                        "age": {
                            "type": "integer",
                            "minimum": 0,
                            "maximum": 120
                        },
                        "name": {
                            "type": "string",
                            "minLength": 1,
                            "maxLength": 100,
                            "pattern": "^[A-Za-z]+$"
                        }
                    },
                    "required": ["email", "age"]
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

    let zod_schemas = generate_zod_schemas(&parsed.openapi, &parsed.schemas, &["Validated".to_string()])
        .unwrap();

    let output: String = zod_schemas.iter().map(|z| z.content.clone()).collect::<Vec<_>>().join("\n\n");
    assert_snapshot!("zod_validation_rules", output);
}

#[tokio::test]
async fn test_zod_nested_validation() {
    let temp_dir = TempDir::new().unwrap();
    let spec_json = r#"
    {
        "openapi": "3.0.0",
        "info": {"title": "Test", "version": "1.0.0"},
        "components": {
            "schemas": {
                "Outer": {
                    "type": "object",
                    "properties": {
                        "inner": {
                            "type": "object",
                            "properties": {
                                "value": {"type": "string"}
                            },
                            "required": ["value"]
                        }
                    },
                    "required": ["inner"]
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

    let zod_schemas = generate_zod_schemas(&parsed.openapi, &parsed.schemas, &["Outer".to_string()])
        .unwrap();

    let output: String = zod_schemas.iter().map(|z| z.content.clone()).collect::<Vec<_>>().join("\n\n");
    assert_snapshot!("zod_nested_validation", output);
}

