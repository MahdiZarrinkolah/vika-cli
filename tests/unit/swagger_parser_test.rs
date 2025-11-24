use tempfile::TempDir;
use std::fs;
use vika_cli::generator::swagger_parser::{
    extract_modules, extract_operations_by_tag, extract_schemas, 
    fetch_and_parse_spec, fetch_and_parse_spec_with_cache,
    get_schema_name_from_ref, resolve_ref, resolve_parameter_ref,
    resolve_request_body_ref, resolve_response_ref,
    filter_common_schemas, collect_all_dependencies
};
use vika_cli::generator::swagger_parser::ParsedSpec;

#[tokio::test]
async fn test_fetch_and_parse_spec_local_json() {
    let temp_dir = TempDir::new().unwrap();
    let spec_content = r#"
{
  "openapi": "3.0.0",
  "info": {
    "title": "Test API",
    "version": "1.0.0"
  },
  "paths": {
    "/test": {
      "get": {
        "tags": ["test"],
        "responses": {
          "200": {
            "description": "Success"
          }
        }
      }
    }
  }
}
"#;
    let spec_path = temp_dir.path().join("spec.json");
    fs::write(&spec_path, spec_content).unwrap();
    
    let result = fetch_and_parse_spec(spec_path.to_str().unwrap()).await;
    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert_eq!(parsed.modules.len(), 1);
    assert!(parsed.modules.contains(&"test".to_string()));
}

#[tokio::test]
async fn test_fetch_and_parse_spec_local_yaml() {
    let temp_dir = TempDir::new().unwrap();
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
    
    let result = fetch_and_parse_spec(spec_path.to_str().unwrap()).await;
    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert_eq!(parsed.modules.len(), 1);
}

#[tokio::test]
async fn test_fetch_and_parse_spec_with_cache() {
    let temp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();
    
    let spec_content = r#"
{
  "openapi": "3.0.0",
  "info": {"title": "Test", "version": "1.0.0"},
  "paths": {}
}
"#;
    let spec_path = temp_dir.path().join("spec.json");
    fs::write(&spec_path, spec_content).unwrap();
    
    // Test without cache
    let result1 = fetch_and_parse_spec_with_cache(spec_path.to_str().unwrap(), false).await;
    assert!(result1.is_ok());
    
    // Test with cache (should work the same for local files)
    let result2 = fetch_and_parse_spec_with_cache(spec_path.to_str().unwrap(), true).await;
    assert!(result2.is_ok());
}

#[test]
fn test_extract_modules_from_tags() {
    use openapiv3::OpenAPI;
    use serde_json;
    
    let json = r#"
{
  "openapi": "3.0.0",
  "info": {"title": "Test", "version": "1.0.0"},
  "tags": [
    {"name": "users"},
    {"name": "products"}
  ],
  "paths": {}
}
"#;
    let openapi: OpenAPI = serde_json::from_str(json).unwrap();
    let modules = extract_modules(&openapi);
    assert_eq!(modules.len(), 2);
    assert!(modules.contains(&"users".to_string()));
    assert!(modules.contains(&"products".to_string()));
}

#[test]
fn test_extract_modules_from_operations() {
    use openapiv3::OpenAPI;
    use serde_json;
    
    let json = r#"
{
  "openapi": "3.0.0",
  "info": {"title": "Test", "version": "1.0.0"},
  "paths": {
    "/users": {
      "get": {
        "tags": ["users"],
        "responses": {"200": {"description": "OK"}}
      }
    },
    "/products": {
      "get": {
        "tags": ["products"],
        "responses": {"200": {"description": "OK"}}
      }
    }
  }
}
"#;
    let openapi: OpenAPI = serde_json::from_str(json).unwrap();
    let modules = extract_modules(&openapi);
    assert!(modules.len() >= 2);
    assert!(modules.contains(&"users".to_string()));
    assert!(modules.contains(&"products".to_string()));
}

#[test]
fn test_extract_operations_by_tag() {
    use openapiv3::OpenAPI;
    use serde_json;
    
    let json = r#"
{
  "openapi": "3.0.0",
  "info": {"title": "Test", "version": "1.0.0"},
  "paths": {
    "/users": {
      "get": {
        "tags": ["users"],
        "responses": {"200": {"description": "OK"}}
      },
      "post": {
        "tags": ["users"],
        "responses": {"201": {"description": "Created"}}
      }
    }
  }
}
"#;
    let openapi: OpenAPI = serde_json::from_str(json).unwrap();
    let operations = extract_operations_by_tag(&openapi);
    assert!(operations.contains_key("users"));
    assert_eq!(operations.get("users").unwrap().len(), 2);
}

#[test]
fn test_extract_schemas() {
    use openapiv3::OpenAPI;
    use serde_json;
    
    let json = r#"
{
  "openapi": "3.0.0",
  "info": {"title": "Test", "version": "1.0.0"},
  "components": {
    "schemas": {
      "User": {
        "type": "object",
        "properties": {
          "id": {"type": "string"},
          "name": {"type": "string"}
        }
      },
      "Product": {
        "type": "object",
        "properties": {
          "id": {"type": "string"}
        }
      }
    }
  },
  "paths": {}
}
"#;
    let openapi: OpenAPI = serde_json::from_str(json).unwrap();
    let schemas = extract_schemas(&openapi);
    assert_eq!(schemas.len(), 2);
    assert!(schemas.contains_key("User"));
    assert!(schemas.contains_key("Product"));
}

#[test]
fn test_get_schema_name_from_ref() {
    let ref_path = "#/components/schemas/User";
    let name = get_schema_name_from_ref(ref_path);
    assert_eq!(name, Some("User".to_string()));
}

#[test]
fn test_get_schema_name_from_ref_invalid() {
    let ref_path = "#/invalid/path";
    let name = get_schema_name_from_ref(ref_path);
    assert!(name.is_none());
}

#[tokio::test]
async fn test_resolve_ref() {
    let temp_dir = TempDir::new().unwrap();
    let spec_content = r#"
{
  "openapi": "3.0.0",
  "info": {"title": "Test", "version": "1.0.0"},
  "components": {
    "schemas": {
      "User": {
        "type": "object",
        "properties": {
          "id": {"type": "string"}
        }
      }
    }
  },
  "paths": {}
}
"#;
    let spec_path = temp_dir.path().join("spec.json");
    fs::write(&spec_path, spec_content).unwrap();
    
    let parsed = fetch_and_parse_spec(spec_path.to_str().unwrap()).await.unwrap();
    let result = resolve_ref(&parsed.openapi, "#/components/schemas/User");
    assert!(result.is_ok());
}

#[test]
fn test_filter_common_schemas() {
    use std::collections::HashMap;
    
    let mut module_schemas = HashMap::new();
    module_schemas.insert("users".to_string(), vec!["User".to_string(), "Common".to_string()]);
    module_schemas.insert("products".to_string(), vec!["Product".to_string(), "Common".to_string()]);
    module_schemas.insert("orders".to_string(), vec!["Order".to_string()]);
    
    let selected = vec!["users".to_string(), "products".to_string()];
    let (filtered, common) = filter_common_schemas(&module_schemas, &selected);
    
    assert!(common.contains(&"Common".to_string()));
    assert!(!common.contains(&"User".to_string()));
    assert!(!common.contains(&"Product".to_string()));
}

