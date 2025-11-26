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
    pub success_map_type: String, // e.g., "OrdersControllerCreateResponses"
    pub error_map_type: String,   // e.g., "OrdersControllerCreateErrors"
    pub generic_result_type: String, // e.g., "ApiResult<OrdersControllerCreateResponses, OrdersControllerCreateErrors>"
    pub import_runtime_path: String, // Path to runtime types/client
}

impl HookContext {
    /// Calculate import path to API functions.
    /// From: src/hooks/{module}/useX.ts
    /// To: {apis_dir}/{module}/index.ts
    /// Note: apis_dir doesn't include spec_name (it's in config if needed), just like schemas
    pub fn calculate_api_import_path(
        module_name: &str,
        _spec_name: Option<&str>,
        apis_dir: Option<&str>,
    ) -> String {
        let module_depth = module_name.matches('/').count() + 1; // +1 for module directory
        let hooks_depth = 1; // hooks directory
        let total_depth = module_depth + hooks_depth;

        if let Some(apis) = apis_dir {
            // Calculate relative path from hooks/{module}/ to {apis_dir}/{module}/
            // Note: hooks_dir and apis_dir don't include spec_name (it's in config if needed)
            let hooks_path = format!("src/hooks/{}", module_name);

            let common_prefix = HookContext::find_common_prefix(&hooks_path, apis);
            let hooks_relative = hooks_path
                .strip_prefix(&common_prefix)
                .unwrap_or(&hooks_path)
                .trim_start_matches('/');
            let apis_relative = apis
                .strip_prefix(&common_prefix)
                .unwrap_or(apis)
                .trim_start_matches('/');

            let hooks_depth_from_common = if hooks_relative.is_empty() {
                0
            } else {
                hooks_relative.matches('/').count() + 1
            };

            let sanitized_module = sanitize_module_name(module_name);
            if apis_relative.is_empty() {
                format!(
                    "{}{}",
                    "../".repeat(hooks_depth_from_common),
                    sanitized_module
                )
            } else {
                format!(
                    "{}{}/{}",
                    "../".repeat(hooks_depth_from_common),
                    apis_relative,
                    sanitized_module
                )
            }
        } else {
            // Fallback: assume apis is at src/apis/{module}
            let sanitized_module = sanitize_module_name(module_name);
            format!("{}apis/{}", "../".repeat(total_depth), sanitized_module)
        }
    }

    /// Find the common prefix of two paths
    pub fn find_common_prefix(path1: &str, path2: &str) -> String {
        let parts1: Vec<&str> = path1.split('/').collect();
        let parts2: Vec<&str> = path2.split('/').collect();

        let mut common = Vec::new();
        let min_len = parts1.len().min(parts2.len());

        for i in 0..min_len {
            if parts1[i] == parts2[i] {
                common.push(parts1[i]);
            } else {
                break;
            }
        }

        common.join("/")
    }

    /// Calculate import path to query keys.
    /// From: {hooks_dir}/{module}/useX.ts
    /// To: {query_keys_dir}/{module}.ts
    /// Note: output_dir already includes spec_name if needed (from config), just like schemas/apis
    pub fn calculate_query_keys_import_path(
        module_name: &str,
        _spec_name: Option<&str>,
        hooks_dir: Option<&str>,
        query_keys_dir: Option<&str>,
    ) -> String {
        let module_depth = module_name.matches('/').count() + 1; // +1 for module directory
        let hooks_depth = 1; // hooks directory
        let total_depth = module_depth + hooks_depth;

        if let (Some(hooks), Some(query_keys)) = (hooks_dir, query_keys_dir) {
            // Calculate relative path from hooks/{module}/ to {query_keys_dir}/{module}.ts
            // Note: hooks_dir and query_keys_dir don't include spec_name (it's in config if needed)
            let hooks_path = format!("{}/{}", hooks, module_name);

            let common_prefix = HookContext::find_common_prefix(&hooks_path, query_keys);
            let hooks_relative = hooks_path
                .strip_prefix(&common_prefix)
                .unwrap_or(&hooks_path)
                .trim_start_matches('/');
            let query_keys_relative = query_keys
                .strip_prefix(&common_prefix)
                .unwrap_or(query_keys)
                .trim_start_matches('/');

            let hooks_depth_from_common = if hooks_relative.is_empty() {
                0
            } else {
                hooks_relative.matches('/').count() + 1
            };

            let sanitized_module = sanitize_module_name(module_name);
            if query_keys_relative.is_empty() {
                format!(
                    "{}{}",
                    "../".repeat(hooks_depth_from_common),
                    sanitized_module
                )
            } else {
                format!(
                    "{}{}/{}",
                    "../".repeat(hooks_depth_from_common),
                    query_keys_relative,
                    sanitized_module
                )
            }
        } else {
            // Fallback: assume query-keys is at src/query-keys/{module}
            let sanitized_module = sanitize_module_name(module_name);
            format!(
                "{}query-keys/{}",
                "../".repeat(total_depth),
                sanitized_module
            )
        }
    }

    /// Calculate import path to runtime client.
    /// From: src/hooks/{module}/useX.ts
    /// To: src/runtime/index.ts
    /// Note: hooks_dir doesn't include spec_name (it's in config if needed), just like schemas/apis
    pub fn calculate_runtime_import_path(module_name: &str, _spec_name: Option<&str>) -> String {
        // Calculate depth: hooks/{module}/ -> runtime/
        // Example: hooks/addresses/ -> ../../runtime
        let module_depth = module_name.matches('/').count() + 1; // +1 for module directory
        let hooks_depth = 1; // hooks directory
        let total_depth = module_depth + hooks_depth;

        format!("{}runtime", "../".repeat(total_depth))
    }
}
