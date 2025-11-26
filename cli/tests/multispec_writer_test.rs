use tempfile::TempDir;
use vika_cli::generator::ts_typings::TypeScriptType;
use vika_cli::generator::writer::{write_api_client_with_options, write_schemas_with_options};
use vika_cli::generator::zod_schema::ZodSchema;

#[test]
fn test_write_schemas_with_spec_name() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("schemas");

    let types = vec![TypeScriptType {
        content: "export interface User { id: string; name: string; }".to_string(),
    }];

    let zod_schemas = vec![ZodSchema {
        content: "export const UserSchema = z.object({ id: z.string(), name: z.string() });"
            .to_string(),
    }];

    // Write with spec name (output_dir should include spec_name)
    let auth_output_dir = output_dir.join("auth");
    let files = write_schemas_with_options(
        &auth_output_dir,
        "users",
        &types,
        &zod_schemas,
        Some("auth"),
        false,
        false,
    )
    .unwrap();

    assert!(!files.is_empty());

    // Verify directory structure: schemas/auth/users/
    let expected_dir = output_dir.join("auth").join("users");
    assert!(
        expected_dir.exists(),
        "Expected directory {:?} to exist",
        expected_dir
    );

    // Verify files exist
    let types_file = expected_dir.join("types.ts");
    let schemas_file = expected_dir.join("schemas.ts");
    let index_file = expected_dir.join("index.ts");

    assert!(types_file.exists());
    assert!(schemas_file.exists());
    assert!(index_file.exists());
}

#[test]
fn test_write_schemas_without_spec_name() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("schemas");

    let types = vec![TypeScriptType {
        content: "export interface User { id: string; }".to_string(),
    }];

    let zod_schemas = vec![ZodSchema {
        content: "export const UserSchema = z.object({ id: z.string() });".to_string(),
    }];

    // Write without spec name (single-spec mode)
    let files = write_schemas_with_options(
        &output_dir,
        "users",
        &types,
        &zod_schemas,
        None,
        false,
        false,
    )
    .unwrap();

    assert!(!files.is_empty());

    // Verify directory structure: schemas/users/ (no spec name prefix)
    let expected_dir = output_dir.join("users");
    assert!(
        expected_dir.exists(),
        "Expected directory {:?} to exist",
        expected_dir
    );

    // Verify no spec-name directory exists
    let spec_dir = output_dir.join("auth");
    assert!(
        !spec_dir.exists(),
        "Spec directory should not exist in single-spec mode"
    );
}

#[test]
fn test_write_api_client_with_spec_name() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("apis");

    use vika_cli::generator::api_client::ApiFunction;
    let functions = vec![ApiFunction {
        content: r#"
export const getUser = async (id: string): Promise<User> => {
  return http.get(`/users/${id}`);
};
"#
        .trim()
        .to_string(),
    }];

    // Write with spec name (output_dir should include spec_name)
    let auth_output_dir = output_dir.join("auth");
    let files = write_api_client_with_options(
        &auth_output_dir,
        "users",
        &functions,
        Some("auth"),
        false,
        false,
    )
    .unwrap();

    assert!(!files.is_empty());

    // Verify directory structure: apis/auth/users/
    let expected_dir = output_dir.join("auth").join("users");
    assert!(
        expected_dir.exists(),
        "Expected directory {:?} to exist",
        expected_dir
    );

    // Verify index file exists
    let index_file = expected_dir.join("index.ts");
    assert!(index_file.exists());
}

#[test]
fn test_write_api_client_without_spec_name() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("apis");

    use vika_cli::generator::api_client::ApiFunction;
    let functions = vec![ApiFunction {
        content: r#"
export const getUser = async (id: string): Promise<User> => {
  return http.get(`/users/${id}`);
};
"#
        .trim()
        .to_string(),
    }];

    // Write without spec name (single-spec mode)
    let files = write_api_client_with_options(&output_dir, "users", &functions, None, false, false)
        .unwrap();

    assert!(!files.is_empty());

    // Verify directory structure: apis/users/ (no spec name prefix)
    let expected_dir = output_dir.join("users");
    assert!(
        expected_dir.exists(),
        "Expected directory {:?} to exist",
        expected_dir
    );

    // Verify no spec-name directory exists
    let spec_dir = output_dir.join("auth");
    assert!(
        !spec_dir.exists(),
        "Spec directory should not exist in single-spec mode"
    );
}

#[test]
fn test_multi_spec_directory_isolation() {
    let temp_dir = TempDir::new().unwrap();
    let schemas_dir = temp_dir.path().join("schemas");
    let _apis_dir = temp_dir.path().join("apis");

    let types = vec![TypeScriptType {
        content: "export interface User { id: string; }".to_string(),
    }];

    let zod_schemas = vec![ZodSchema {
        content: "export const UserSchema = z.object({ id: z.string() });".to_string(),
    }];

    // Write for auth spec (output_dir should include spec_name)
    let auth_schemas_dir = schemas_dir.join("auth");
    write_schemas_with_options(
        &auth_schemas_dir,
        "users",
        &types,
        &zod_schemas,
        Some("auth"),
        false,
        false,
    )
    .unwrap();

    // Write for orders spec (output_dir should include spec_name)
    let orders_schemas_dir = schemas_dir.join("orders");
    write_schemas_with_options(
        &orders_schemas_dir,
        "users",
        &types,
        &zod_schemas,
        Some("orders"),
        false,
        false,
    )
    .unwrap();

    // Verify both directories exist and are separate
    let auth_dir = schemas_dir.join("auth").join("users");
    let orders_dir = schemas_dir.join("orders").join("users");

    assert!(auth_dir.exists());
    assert!(orders_dir.exists());
    assert_ne!(auth_dir, orders_dir);
}
