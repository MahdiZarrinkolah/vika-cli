use insta::assert_snapshot;
use vika_cli::generator::api_client::generate_api_client;
use vika_cli::generator::swagger_parser::fetch_and_parse_spec;
use tempfile::TempDir;
use std::fs;

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
    let api_functions = generate_api_client(&parsed.openapi, operations, "test", &[]).unwrap();

    let output: String = api_functions.iter().map(|f| f.content.clone()).collect::<Vec<_>>().join("\n\n");
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
    let api_functions = generate_api_client(&parsed.openapi, operations, "test", &[]).unwrap();

    let output: String = api_functions.iter().map(|f| f.content.clone()).collect::<Vec<_>>().join("\n\n");
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
    let api_functions = generate_api_client(&parsed.openapi, operations, "test", &[]).unwrap();

    let output: String = api_functions.iter().map(|f| f.content.clone()).collect::<Vec<_>>().join("\n\n");
    assert_snapshot!("path_parameters", output);
}

#[tokio::test]
async fn test_request_bodies() {
    let temp_dir = TempDir::new().unwrap();
    let spec_json = r#"
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
    "#;

    let spec_path = temp_dir.path().join("spec.json");
    fs::write(&spec_path, spec_json).unwrap();

    let parsed = fetch_and_parse_spec(spec_path.to_str().unwrap())
        .await
        .unwrap();

    let operations = parsed.operations_by_tag.get("test").unwrap();
    let api_functions = generate_api_client(&parsed.openapi, operations, "test", &[]).unwrap();

    let output: String = api_functions.iter().map(|f| f.content.clone()).collect::<Vec<_>>().join("\n\n");
    assert_snapshot!("request_bodies", output);
}

