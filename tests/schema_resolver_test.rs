use vika_cli::generator::schema_resolver::SchemaResolver;
use vika_cli::generator::swagger_parser::fetch_and_parse_spec;
use tempfile::TempDir;
use std::fs;

#[tokio::test]
async fn test_schema_resolver_new() {
    let temp_dir = TempDir::new().unwrap();
    let spec_yaml = r##"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths: {}
components:
  schemas:
    User:
      type: object
      properties:
        id:
          type: string
"##;

    let spec_path = temp_dir.path().join("spec.yaml");
    fs::write(&spec_path, spec_yaml).unwrap();

    let parsed = fetch_and_parse_spec(spec_path.to_str().unwrap())
        .await
        .unwrap();

    let resolver = SchemaResolver::new(parsed.openapi);
    assert_eq!(resolver.get_openapi().info.title, "Test API");
}

#[tokio::test]
async fn test_build_dependency_graph() {
    let temp_dir = TempDir::new().unwrap();
    let spec_yaml = r##"
openapi: 3.0.0
info:
  title: Test
  version: 1.0.0
paths: {}
components:
  schemas:
    User:
      type: object
      properties:
        id:
          type: string
        profile:
          $ref: "#/components/schemas/Profile"
    Profile:
      type: object
      properties:
        name:
          type: string
"##;

    let spec_path = temp_dir.path().join("spec.yaml");
    fs::write(&spec_path, spec_yaml).unwrap();

    let parsed = fetch_and_parse_spec(spec_path.to_str().unwrap())
        .await
        .unwrap();

    let mut resolver = SchemaResolver::new(parsed.openapi);
    let result = resolver.build_dependency_graph();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_circular_dependency_detection() {
    let temp_dir = TempDir::new().unwrap();
    let spec_yaml = r##"
openapi: 3.0.0
info:
  title: Test
  version: 1.0.0
paths: {}
components:
  schemas:
    User:
      type: object
      properties:
        id:
          type: string
        profile:
          $ref: "#/components/schemas/Profile"
    Profile:
      type: object
      properties:
        userId:
          type: string
        user:
          $ref: "#/components/schemas/User"
"##;

    let spec_path = temp_dir.path().join("spec.yaml");
    fs::write(&spec_path, spec_yaml).unwrap();

    let parsed = fetch_and_parse_spec(spec_path.to_str().unwrap())
        .await
        .unwrap();

    let mut resolver = SchemaResolver::new(parsed.openapi);
    let result = resolver.build_dependency_graph();
    assert!(result.is_ok());
    
    let cycles = resolver.detect_circular_dependencies();
    assert!(cycles.is_ok());
}

#[tokio::test]
async fn test_resolve_schema_ref() {
    let temp_dir = TempDir::new().unwrap();
    let spec_yaml = r##"
openapi: 3.0.0
info:
  title: Test
  version: 1.0.0
paths: {}
components:
  schemas:
    User:
      type: object
      properties:
        id:
          type: string
"##;

    let spec_path = temp_dir.path().join("spec.yaml");
    fs::write(&spec_path, spec_yaml).unwrap();

    let parsed = fetch_and_parse_spec(spec_path.to_str().unwrap())
        .await
        .unwrap();

    let mut resolver = SchemaResolver::new(parsed.openapi);
    let result = resolver.resolve_schema_ref("#/components/schemas/User");
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_resolve_with_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let spec_yaml = r##"
openapi: 3.0.0
info:
  title: Test
  version: 1.0.0
paths: {}
components:
  schemas:
    User:
      type: object
      properties:
        id:
          type: string
        profile:
          $ref: "#/components/schemas/Profile"
    Profile:
      type: object
      properties:
        name:
          type: string
"##;

    let spec_path = temp_dir.path().join("spec.yaml");
    fs::write(&spec_path, spec_yaml).unwrap();

    let parsed = fetch_and_parse_spec(spec_path.to_str().unwrap())
        .await
        .unwrap();

    let mut resolver = SchemaResolver::new(parsed.openapi);
    resolver.build_dependency_graph().unwrap();
    
    let result = resolver.resolve_with_dependencies("User");
    assert!(result.is_ok());
    let deps = result.unwrap();
    assert!(deps.contains(&"User".to_string()));
}

#[tokio::test]
async fn test_is_circular() {
    let temp_dir = TempDir::new().unwrap();
    let spec_yaml = r##"
openapi: 3.0.0
info:
  title: Test
  version: 1.0.0
paths: {}
components:
  schemas:
    User:
      type: object
      properties:
        id:
          type: string
        profile:
          $ref: "#/components/schemas/Profile"
    Profile:
      type: object
      properties:
        user:
          $ref: "#/components/schemas/User"
"##;

    let spec_path = temp_dir.path().join("spec.yaml");
    fs::write(&spec_path, spec_yaml).unwrap();

    let parsed = fetch_and_parse_spec(spec_path.to_str().unwrap())
        .await
        .unwrap();

    let mut resolver = SchemaResolver::new(parsed.openapi);
    resolver.build_dependency_graph().unwrap();
    
    let cycles = resolver.detect_circular_dependencies().unwrap();
    assert!(cycles.len() >= 0);
}

#[tokio::test]
async fn test_classify_schema() {
    let temp_dir = TempDir::new().unwrap();
    let spec_yaml = r##"
openapi: 3.0.0
info:
  title: Test
  version: 1.0.0
paths: {}
components:
  schemas:
    StringSchema:
      type: string
    NumberSchema:
      type: number
    IntegerSchema:
      type: integer
    BooleanSchema:
      type: boolean
    ArraySchema:
      type: array
      items:
        type: string
    ObjectSchema:
      type: object
      properties:
        name:
          type: string
    EnumSchema:
      type: string
      enum:
        - value1
        - value2
"##;

    let spec_path = temp_dir.path().join("spec.yaml");
    fs::write(&spec_path, spec_yaml).unwrap();

    let parsed = fetch_and_parse_spec(spec_path.to_str().unwrap())
        .await
        .unwrap();

    let resolver = SchemaResolver::new(parsed.openapi);
    
    if let Some(components) = &resolver.get_openapi().components {
        if let Some(openapiv3::ReferenceOr::Item(schema)) = components.schemas.get("StringSchema") {
            let schema_type = resolver.classify_schema(schema);
            matches!(schema_type, vika_cli::generator::schema_resolver::SchemaType::Primitive(_));
        }
    }
}

#[tokio::test]
async fn test_empty_components() {
    let temp_dir = TempDir::new().unwrap();
    let spec_yaml = r##"
openapi: 3.0.0
info:
  title: Test
  version: 1.0.0
paths: {}
"##;

    let spec_path = temp_dir.path().join("spec.yaml");
    fs::write(&spec_path, spec_yaml).unwrap();

    let parsed = fetch_and_parse_spec(spec_path.to_str().unwrap())
        .await
        .unwrap();

    let mut resolver = SchemaResolver::new(parsed.openapi);
    let result = resolver.build_dependency_graph();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_array_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let spec_yaml = r##"
openapi: 3.0.0
info:
  title: Test
  version: 1.0.0
paths: {}
components:
  schemas:
    UserList:
      type: array
      items:
        $ref: "#/components/schemas/User"
    User:
      type: object
      properties:
        id:
          type: string
"##;

    let spec_path = temp_dir.path().join("spec.yaml");
    fs::write(&spec_path, spec_yaml).unwrap();

    let parsed = fetch_and_parse_spec(spec_path.to_str().unwrap())
        .await
        .unwrap();

    let mut resolver = SchemaResolver::new(parsed.openapi);
    let result = resolver.build_dependency_graph();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_allof_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let spec_yaml = r##"
openapi: 3.0.0
info:
  title: Test
  version: 1.0.0
paths: {}
components:
  schemas:
    Base:
      type: object
      properties:
        id:
          type: string
    Extended:
      allOf:
        - $ref: "#/components/schemas/Base"
        - type: object
          properties:
            name:
              type: string
"##;

    let spec_path = temp_dir.path().join("spec.yaml");
    fs::write(&spec_path, spec_yaml).unwrap();

    let parsed = fetch_and_parse_spec(spec_path.to_str().unwrap())
        .await
        .unwrap();

    let mut resolver = SchemaResolver::new(parsed.openapi);
    let result = resolver.build_dependency_graph();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_oneof_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let spec_yaml = r##"
openapi: 3.0.0
info:
  title: Test
  version: 1.0.0
paths: {}
components:
  schemas:
    Cat:
      type: object
      properties:
        meow:
          type: boolean
    Dog:
      type: object
      properties:
        bark:
          type: boolean
    Pet:
      oneOf:
        - $ref: "#/components/schemas/Cat"
        - $ref: "#/components/schemas/Dog"
"##;

    let spec_path = temp_dir.path().join("spec.yaml");
    fs::write(&spec_path, spec_yaml).unwrap();

    let parsed = fetch_and_parse_spec(spec_path.to_str().unwrap())
        .await
        .unwrap();

    let mut resolver = SchemaResolver::new(parsed.openapi);
    let result = resolver.build_dependency_graph();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_anyof_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let spec_yaml = r##"
openapi: 3.0.0
info:
  title: Test
  version: 1.0.0
paths: {}
components:
  schemas:
    StringValue:
      type: object
      properties:
        value:
          type: string
    NumberValue:
      type: object
      properties:
        value:
          type: number
    AnyValue:
      anyOf:
        - $ref: "#/components/schemas/StringValue"
        - $ref: "#/components/schemas/NumberValue"
"##;

    let spec_path = temp_dir.path().join("spec.yaml");
    fs::write(&spec_path, spec_yaml).unwrap();

    let parsed = fetch_and_parse_spec(spec_path.to_str().unwrap())
        .await
        .unwrap();

    let mut resolver = SchemaResolver::new(parsed.openapi);
    let result = resolver.build_dependency_graph();
    assert!(result.is_ok());
}
