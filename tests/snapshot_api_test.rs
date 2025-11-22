use insta::assert_snapshot;
use std::fs;
use tempfile::TempDir;
use vika_cli::generator::api_client::generate_api_client;
use vika_cli::generator::swagger_parser::fetch_and_parse_spec;

#[tokio::test]
async fn test_all_http_methods() {
    let temp_dir = TempDir::new().unwrap();
    let spec_json = r#"
    {
        "openapi": "3.0.0",
        "info": {"title": "Test", "version": "1.0.0"},
        "tags": [{"name": "test"}],
        "paths": {
            "/test": {
                "get": {"tags": ["test"], "operationId": "getTest", "responses": {"200": {"description": "OK"}}},
                "post": {"tags": ["test"], "operationId": "postTest", "responses": {"200": {"description": "OK"}}},
                "put": {"tags": ["test"], "operationId": "putTest", "responses": {"200": {"description": "OK"}}},
                "delete": {"tags": ["test"], "operationId": "deleteTest", "responses": {"200": {"description": "OK"}}},
                "patch": {"tags": ["test"], "operationId": "patchTest", "responses": {"200": {"description": "OK"}}},
                "head": {"tags": ["test"], "operationId": "headTest", "responses": {"200": {"description": "OK"}}},
                "options": {"tags": ["test"], "operationId": "optionsTest", "responses": {"200": {"description": "OK"}}}
            }
        }
    }
    "#;

    let spec_path = temp_dir.path().join("spec.json");
    fs::write(&spec_path, spec_json).unwrap();

    let parsed = fetch_and_parse_spec(spec_path.to_str().unwrap())
        .await
        .unwrap();

    let operations = parsed.operations_by_tag.get("test").unwrap();
    let api_result = generate_api_client(&parsed.openapi, operations, "test", &[]).unwrap();

    let output: String = api_result.functions
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n\n");
    assert_snapshot!("all_http_methods", output);
}

#[tokio::test]
async fn test_query_parameters() {
    let temp_dir = TempDir::new().unwrap();
    let spec_json = r#"
    {
        "openapi": "3.0.0",
        "info": {"title": "Test", "version": "1.0.0"},
        "tags": [{"name": "test"}],
        "paths": {
            "/test": {
                "get": {
                    "tags": ["test"],
                    "operationId": "getTest",
                    "parameters": [
                        {"name": "page", "in": "query", "schema": {"type": "integer"}},
                        {"name": "limit", "in": "query", "schema": {"type": "integer"}},
                        {"name": "search", "in": "query", "schema": {"type": "string"}}
                    ],
                    "responses": {"200": {"description": "OK"}}
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

    let operations = parsed.operations_by_tag.get("test").unwrap();
    let api_result = generate_api_client(&parsed.openapi, operations, "test", &[]).unwrap();

    let output: String = api_result.functions
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n\n");
    assert_snapshot!("query_parameters", output);
}

#[tokio::test]
async fn test_path_parameters() {
    let temp_dir = TempDir::new().unwrap();
    let spec_json = r#"
    {
        "openapi": "3.0.0",
        "info": {"title": "Test", "version": "1.0.0"},
        "tags": [{"name": "test"}],
        "paths": {
            "/test/{id}": {
                "get": {
                    "tags": ["test"],
                    "operationId": "getTestById",
                    "parameters": [
                        {"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}
                    ],
                    "responses": {"200": {"description": "OK"}}
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

    let operations = parsed.operations_by_tag.get("test").unwrap();
    let api_result = generate_api_client(&parsed.openapi, operations, "test", &[]).unwrap();

    let output: String = api_result.functions
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n\n");
    assert_snapshot!("path_parameters", output);
}

#[tokio::test]
async fn test_request_bodies() {
    let temp_dir = TempDir::new().unwrap();
    let spec_json = r##"
    {
        "openapi": "3.0.0",
        "info": {"title": "Test", "version": "1.0.0"},
        "tags": [{"name": "test"}],
        "components": {
            "schemas": {
                "User": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "email": {"type": "string"}
                    }
                }
            }
        },
        "paths": {
            "/test": {
                "post": {
                    "tags": ["test"],
                    "operationId": "createTest",
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/User"}
                            }
                        }
                    },
                    "responses": {"200": {"description": "OK"}}
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

    let operations = parsed.operations_by_tag.get("test").unwrap();
    let api_result = generate_api_client(&parsed.openapi, operations, "test", &[]).unwrap();

    let output: String = api_result.functions
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n\n");
    assert_snapshot!("request_bodies", output);
}

#[tokio::test]
async fn test_enum_parameters() {
    let temp_dir = TempDir::new().unwrap();
    let spec_json = r#"
    {
        "openapi": "3.0.0",
        "info": {"title": "Test", "version": "1.0.0"},
        "tags": [{"name": "test"}],
        "paths": {
            "/products": {
                "get": {
                    "tags": ["test"],
                    "operationId": "getProducts",
                    "parameters": [
                        {
                            "name": "sort",
                            "in": "query",
                            "schema": {
                                "type": "string",
                                "enum": ["price", "rating", "name"]
                            }
                        },
                        {
                            "name": "status",
                            "in": "path",
                            "required": true,
                            "schema": {
                                "type": "string",
                                "enum": ["active", "inactive"]
                            }
                        }
                    ],
                    "responses": {"200": {"description": "OK"}}
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

    let operations = parsed.operations_by_tag.get("test").unwrap();
    let api_result = generate_api_client(&parsed.openapi, operations, "test", &[]).unwrap();

    let output: String = api_result.functions
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n\n");
    assert_snapshot!("enum_parameters", output);
}

#[tokio::test]
async fn test_array_query_parameters() {
    let temp_dir = TempDir::new().unwrap();
    let spec_json = r#"
    {
        "openapi": "3.0.0",
        "info": {"title": "Test", "version": "1.0.0"},
        "tags": [{"name": "test"}],
        "paths": {
            "/products": {
                "get": {
                    "tags": ["test"],
                    "operationId": "getProducts",
                    "parameters": [
                        {
                            "name": "tags",
                            "in": "query",
                            "style": "form",
                            "explode": true,
                            "schema": {
                                "type": "array",
                                "items": {"type": "string"}
                            }
                        },
                        {
                            "name": "ids",
                            "in": "query",
                            "style": "form",
                            "explode": false,
                            "schema": {
                                "type": "array",
                                "items": {"type": "integer"}
                            }
                        }
                    ],
                    "responses": {"200": {"description": "OK"}}
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

    let operations = parsed.operations_by_tag.get("test").unwrap();
    let api_result = generate_api_client(&parsed.openapi, operations, "test", &[]).unwrap();

    let output: String = api_result.functions
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n\n");
    assert_snapshot!("array_query_parameters", output);
}

#[tokio::test]
async fn test_error_responses() {
    let temp_dir = TempDir::new().unwrap();
    let spec_json = r##"
    {
        "openapi": "3.0.0",
        "info": {"title": "Test", "version": "1.0.0"},
        "tags": [{"name": "test"}],
        "components": {
            "schemas": {
                "Error": {
                    "type": "object",
                    "properties": {
                        "message": {"type": "string"}
                    }
                },
                "NotFoundError": {
                    "type": "object",
                    "properties": {
                        "error": {"type": "string"}
                    }
                }
            }
        },
        "paths": {
            "/products/{id}": {
                "get": {
                    "tags": ["test"],
                    "operationId": "getProduct",
                    "parameters": [
                        {"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}
                    ],
                    "responses": {
                        "200": {
                            "description": "OK",
                            "content": {
                                "application/json": {
                                    "schema": {"type": "object"}
                                }
                            }
                        },
                        "400": {
                            "description": "Bad Request",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/Error"}
                                }
                            }
                        },
                        "404": {
                            "description": "Not Found",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/NotFoundError"}
                                }
                            }
                        },
                        "500": {
                            "description": "Internal Server Error"
                        }
                    }
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

    let operations = parsed.operations_by_tag.get("test").unwrap();
    let api_result = generate_api_client(&parsed.openapi, operations, "test", &[]).unwrap();

    // Combine function output with response types for snapshot
    let mut output_parts = Vec::new();
    
    // Add response types
    for response_type in &api_result.response_types {
        output_parts.push(response_type.content.clone());
    }
    
    // Add functions
    for func in &api_result.functions {
        output_parts.push(func.content.clone());
    }
    
    let output = output_parts.join("\n\n");
    assert_snapshot!("error_responses", output);
}
