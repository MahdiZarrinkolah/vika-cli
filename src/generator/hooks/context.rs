use crate::generator::utils::sanitize_module_name;
use crate::templates::context::Parameter;
use serde::Serialize;

/// Context for hook generation.
#[derive(Debug, Clone, Serialize)]
pub struct HookContext {
    pub hook_name: String,
    pub key_name: String,
    pub operation_id: String,
    pub http_method: String,
    pub path: String,
    pub path_params: Vec<Parameter>,
    pub query_params: Vec<Parameter>,
    pub body_type: Option<String>,
    pub response_type: String,
    pub module_name: String,
    pub spec_name: Option<String>,
    pub api_import_path: String,
    pub query_keys_import_path: String,
    pub param_list: String, // Full parameter list with types: "id: string, query?: { page?: number }"
    pub param_names: String, // Just parameter names for function calls: "id, query"
    pub path_param_names: String, // Just path parameter names: "id"
    pub schema_imports: String, // Schema import statements
    pub description: String,
}

impl HookContext {
    /// Calculate import path to API functions.
    /// From: src/hooks/{spec}/{module}/useX.ts
    /// To: src/apis/{spec}/{module}/index.ts
    pub fn calculate_api_import_path(module_name: &str, spec_name: Option<&str>) -> String {
        // Calculate depth: hooks/{spec}/{module}/ -> apis/{spec}/{module}/
        // For multi-spec: hooks/orders/orders/ -> ../../../apis/orders/orders
        // For single-spec: hooks/users/ -> ../../apis/users
        let module_depth = module_name.matches('/').count() + 1; // +1 for module directory
        let spec_depth = if spec_name.is_some() { 1 } else { 0 };
        let total_depth = module_depth + spec_depth + 1; // +1 for hooks directory

        let sanitized_module = sanitize_module_name(module_name);

        if let Some(spec) = spec_name {
            let sanitized_spec = sanitize_module_name(spec);
            format!(
                "{}apis/{}/{}",
                "../".repeat(total_depth),
                sanitized_spec,
                sanitized_module
            )
        } else {
            format!("{}apis/{}", "../".repeat(total_depth), sanitized_module)
        }
    }

    /// Calculate import path to query keys.
    /// From: src/hooks/{spec}/{module}/useX.ts
    /// To: src/query-keys/{spec}/{module}.ts
    pub fn calculate_query_keys_import_path(module_name: &str, spec_name: Option<&str>) -> String {
        // Calculate depth: hooks/{spec}/{module}/ -> query-keys/{spec}/{module}.ts
        // For multi-spec: hooks/orders/orders/ -> ../../../query-keys/orders/orders
        // For single-spec: hooks/users/ -> ../../query-keys/users
        let module_depth = module_name.matches('/').count() + 1; // +1 for module directory
        let spec_depth = if spec_name.is_some() { 1 } else { 0 };
        let total_depth = module_depth + spec_depth + 1; // +1 for hooks directory

        let sanitized_module = sanitize_module_name(module_name);

        if let Some(spec) = spec_name {
            let sanitized_spec = sanitize_module_name(spec);
            format!(
                "{}query-keys/{}/{}",
                "../".repeat(total_depth),
                sanitized_spec,
                sanitized_module
            )
        } else {
            format!(
                "{}query-keys/{}",
                "../".repeat(total_depth),
                sanitized_module
            )
        }
    }
}
