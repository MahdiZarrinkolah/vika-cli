use insta::assert_snapshot;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use vika_cli::generator::api_client::generate_api_client_with_registry_and_engine_and_spec;
use vika_cli::generator::swagger_parser::fetch_and_parse_spec;
use vika_cli::generator::ts_typings::generate_typings_with_registry_and_engine_and_spec;
use vika_cli::generator::writer::{write_api_client_with_options, write_schemas_with_options};
use vika_cli::generator::zod_schema::generate_zod_schemas_with_registry_and_engine_and_spec;
use vika_cli::templates::engine::TemplateEngine;

#[tokio::test]
async fn test_multi_spec_output_structure() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Setup: Copy fixture specs
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/multi-spec");
    let specs_dir = temp_dir.path().join("specs");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::copy(fixtures_dir.join("auth.yaml"), specs_dir.join("auth.yaml")).unwrap();
    fs::copy(
        fixtures_dir.join("orders.json"),
        specs_dir.join("orders.json"),
    )
    .unwrap();

    // Parse auth spec
    let auth_parsed = fetch_and_parse_spec(specs_dir.join("auth.yaml").to_str().unwrap())
        .await
        .unwrap();

    let schemas_dir = temp_dir.path().join("schemas");
    let apis_dir = temp_dir.path().join("apis");

    // Initialize template engine
    let template_engine = TemplateEngine::new(None).unwrap();
    let mut enum_registry = std::collections::HashMap::new();

    // Generate for auth spec
    let users_module_schemas = auth_parsed
        .module_schemas
        .get("users")
        .cloned()
        .unwrap_or_default();

    if !users_module_schemas.is_empty() {
        let types = generate_typings_with_registry_and_engine_and_spec(
            &auth_parsed.openapi,
            &auth_parsed.schemas,
            &users_module_schemas,
            &mut enum_registry,
            &[],
            Some(&template_engine),
            Some("auth"),
        )
        .unwrap();

        let zod_schemas = generate_zod_schemas_with_registry_and_engine_and_spec(
            &auth_parsed.openapi,
            &auth_parsed.schemas,
            &users_module_schemas,
            &mut enum_registry,
            &[],
            Some(&template_engine),
            Some("auth"),
        )
        .unwrap();

        write_schemas_with_options(
            &schemas_dir,
            "users",
            &types,
            &zod_schemas,
            Some("auth"),
            false,
            false,
        )
        .unwrap();
    }

    // Verify directory structure
    let auth_users_dir = schemas_dir.join("auth").join("users");
    assert!(
        auth_users_dir.exists(),
        "Expected auth/users directory to exist"
    );

    // Verify files exist
    let types_file = auth_users_dir.join("types.ts");
    assert!(types_file.exists());
    let content = fs::read_to_string(&types_file).unwrap();
    assert_snapshot!("multispec_auth_users_types", content);

    std::env::set_current_dir(original_dir).unwrap();
}

#[tokio::test]
async fn test_multi_spec_api_client_structure() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Setup: Copy fixture specs
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/multi-spec");
    let specs_dir = temp_dir.path().join("specs");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::copy(
        fixtures_dir.join("orders.json"),
        specs_dir.join("orders.json"),
    )
    .unwrap();

    // Parse orders spec
    let orders_parsed = fetch_and_parse_spec(specs_dir.join("orders.json").to_str().unwrap())
        .await
        .unwrap();

    let apis_dir = temp_dir.path().join("apis");

    // Initialize template engine
    let template_engine = TemplateEngine::new(None).unwrap();
    let mut enum_registry = std::collections::HashMap::new();

    // Generate API client for orders spec
    let orders_operations = orders_parsed
        .operations_by_tag
        .get("orders")
        .cloned()
        .unwrap_or_default();

    if !orders_operations.is_empty() {
        let api_result = generate_api_client_with_registry_and_engine_and_spec(
            &orders_parsed.openapi,
            &orders_operations,
            "orders",
            &[],
            &mut enum_registry,
            Some(&template_engine),
            Some("orders"),
        )
        .unwrap();

        write_api_client_with_options(
            &apis_dir,
            "orders",
            &api_result.functions,
            Some("orders"),
            false,
            false,
        )
        .unwrap();
    }

    // Verify directory structure
    let orders_orders_dir = apis_dir.join("orders").join("orders");
    assert!(
        orders_orders_dir.exists(),
        "Expected orders/orders directory to exist"
    );

    // Verify index file exists
    let index_file = orders_orders_dir.join("index.ts");
    assert!(index_file.exists());
    let content = fs::read_to_string(&index_file).unwrap();
    assert_snapshot!("multispec_orders_api", content);

    std::env::set_current_dir(original_dir).unwrap();
}
