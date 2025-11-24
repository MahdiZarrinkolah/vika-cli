use vika_cli::generator::api_client::generate_api_client;
use vika_cli::generator::swagger_parser::fetch_and_parse_spec;
use vika_cli::generator::ts_typings::generate_typings;
use vika_cli::generator::writer::{write_api_client_with_options, write_schemas_with_options};
use vika_cli::generator::zod_schema::generate_zod_schemas;
mod common;
use common::*;

#[tokio::test]
async fn test_full_generation_workflow() {
    let temp_dir = setup_test_env();
    let spec_json = r##"
    {
        "openapi": "3.0.0",
        "info": {
            "title": "Test API",
            "version": "1.0.0"
        },
        "tags": [
            {"name": "users"}
        ],
        "paths": {
            "/users": {
                "get": {
                    "tags": ["users"],
                    "operationId": "getUsers",
                    "responses": {
                        "200": {
                            "description": "Success",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {
                                            "$ref": "#/components/schemas/User"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        "components": {
            "schemas": {
                "User": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "string"},
                        "name": {"type": "string"}
                    },
                    "required": ["id", "name"]
                }
            }
        }
    }
    "##;

    let spec_path = temp_dir.path().join("spec.json");
    std::fs::write(&spec_path, spec_json).unwrap();

    // Parse spec
    let parsed = fetch_and_parse_spec(spec_path.to_str().unwrap())
        .await
        .unwrap();
    assert_eq!(parsed.modules.len(), 1);

    // Generate types
    let schema_names = vec!["User".to_string()];
    let types = generate_typings(&parsed.openapi, &parsed.schemas, &schema_names).unwrap();
    assert!(!types.is_empty());

    // Generate Zod schemas
    let zod_schemas =
        generate_zod_schemas(&parsed.openapi, &parsed.schemas, &schema_names).unwrap();
    assert!(!zod_schemas.is_empty());

    // Generate API client
    let operations = parsed.operations_by_tag.get("users").unwrap();
    let api_functions = generate_api_client(&parsed.openapi, operations, "users", &[]).unwrap();
    assert!(!api_functions.functions.is_empty());

    // Write files
    let output_dir = temp_dir.path().join("output");
    let schema_files =
        write_schemas_with_options(&output_dir, "users", &types, &zod_schemas, None, false, false)
            .unwrap();
    assert!(!schema_files.is_empty());

    let api_files =
        write_api_client_with_options(&output_dir, "users", &api_functions.functions, None, false, false)
            .unwrap();
    assert!(!api_files.is_empty());

    // Verify files exist
    assert!(output_dir.join("users/types.ts").exists());
    assert!(output_dir.join("users/schemas.ts").exists());
    assert!(output_dir.join("users/index.ts").exists());
}
