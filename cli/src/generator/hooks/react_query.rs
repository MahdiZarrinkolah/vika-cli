use crate::error::Result;
use crate::generator::api_client::{
    extract_all_responses, extract_path_parameters, extract_query_parameters, extract_request_body,
    ResponseInfo,
};
use crate::generator::hooks::context::HookContext;
use crate::generator::hooks::HookFile;
use crate::generator::swagger_parser::OperationInfo;
use crate::generator::utils::{to_camel_case, to_pascal_case};
use crate::templates::context::Parameter as ApiParameter;
use crate::templates::engine::TemplateEngine;
use crate::templates::registry::TemplateId;
use openapiv3::OpenAPI;

/// Generate React Query hooks from operations.
#[allow(clippy::too_many_arguments)]
pub fn generate_react_query_hooks(
    openapi: &OpenAPI,
    operations: &[OperationInfo],
    module_name: &str,
    spec_name: Option<&str>,
    common_schemas: &[String],
    enum_registry: &mut std::collections::HashMap<String, String>,
    template_engine: &TemplateEngine,
    apis_dir: Option<&str>,
    schemas_dir: Option<&str>,
    hooks_dir: Option<&str>,
    query_keys_dir: Option<&str>,
) -> Result<Vec<HookFile>> {
    let mut hooks = Vec::new();

    for op_info in operations {
        let operation = &op_info.operation;
        let method = op_info.method.to_uppercase();

        // Determine if query or mutation
        let is_query = matches!(method.as_str(), "GET" | "HEAD");
        let is_mutation = matches!(method.as_str(), "POST" | "PUT" | "PATCH" | "DELETE");

        if !is_query && !is_mutation {
            continue; // Skip unsupported methods
        }

        // Generate operation ID (function name)
        let operation_id = if let Some(op_id) = &operation.operation_id {
            to_camel_case(op_id)
        } else {
            generate_hook_name_from_path(&op_info.path, &op_info.method)
        };

        // Generate hook name
        let hook_name = format!("use{}", to_pascal_case(&operation_id));

        // Generate key name (same as operation_id for queries, but we still need it for mutations)
        let key_name = operation_id.clone();

        // Extract parameters
        let path_params_info = extract_path_parameters(openapi, operation, enum_registry)?;
        let query_params_info = extract_query_parameters(openapi, operation, enum_registry)?;
        let request_body_info = extract_request_body(openapi, operation)?;
        let all_responses = extract_all_responses(openapi, operation)?;

        // Extract success and error responses
        let success_responses: Vec<ResponseInfo> = all_responses
            .iter()
            .filter(|r| r.status_code >= 200 && r.status_code < 300)
            .cloned()
            .collect();
        let _error_responses: Vec<ResponseInfo> = all_responses
            .iter()
            .filter(|r| r.status_code < 200 || r.status_code >= 300)
            .cloned()
            .collect();
        let response_type = success_responses
            .iter()
            .find(|r| r.status_code == 200)
            .map(|r| r.body_type.clone())
            .unwrap_or_else(|| "any".to_string());

        // Generate type names for success and error maps
        let type_name_base = to_pascal_case(&operation_id);
        let success_map_type = format!("{}Responses", type_name_base);
        let error_map_type = format!("{}Errors", type_name_base);
        let generic_result_type = format!("ApiResult<{}, {}>", success_map_type, error_map_type);

        // Build parameter list for hook
        let mut param_list_parts = Vec::new();
        let mut param_names_parts = Vec::new();

        // Add path parameters
        for param in &path_params_info {
            let param_type = match &param.param_type {
                crate::generator::api_client::ParameterType::Enum(enum_name) => enum_name.clone(),
                crate::generator::api_client::ParameterType::String => "string".to_string(),
                crate::generator::api_client::ParameterType::Number => "number".to_string(),
                crate::generator::api_client::ParameterType::Integer => "number".to_string(),
                crate::generator::api_client::ParameterType::Boolean => "boolean".to_string(),
                crate::generator::api_client::ParameterType::Array(_) => "string".to_string(),
            };
            param_list_parts.push(format!("{}: {}", param.name, param_type));
            param_names_parts.push(param.name.clone());
        }

        // Collect enum types from query parameters for imports
        let mut enum_types = Vec::new();
        let namespace_name = to_pascal_case(&module_name.replace("/", "_"));

        // Add query parameters (only for queries)
        // Query params types are now in schema files, so use namespace-qualified types
        if is_query && !query_params_info.is_empty() {
            let mut query_fields = Vec::new();
            for param in &query_params_info {
                let param_type = match &param.param_type {
                    crate::generator::api_client::ParameterType::Enum(enum_name) => {
                        enum_types.push(enum_name.clone());
                        // Use namespace-qualified enum name (e.g., Orders.SortByEnum)
                        format!("{}.{}", namespace_name, enum_name)
                    }
                    crate::generator::api_client::ParameterType::Array(item_type) => {
                        format!("{}[]", item_type)
                    }
                    crate::generator::api_client::ParameterType::String => "string".to_string(),
                    crate::generator::api_client::ParameterType::Number => "number".to_string(),
                    crate::generator::api_client::ParameterType::Integer => "number".to_string(),
                    crate::generator::api_client::ParameterType::Boolean => "boolean".to_string(),
                };
                query_fields.push(format!("{}?: {}", param.name, param_type));
            }
            let query_type = format!("{{ {} }}", query_fields.join(", "));
            param_list_parts.push(format!("query?: {}", query_type));
            param_names_parts.push("query".to_string());
        }

        // For mutations, DO NOT add body parameter to hook signature
        // Body parameter is passed via mutate(data) call, not as hook parameter
        // Only path parameters should be in the hook signature

        let param_list = param_list_parts.join(", ");
        let param_names = param_names_parts.join(", ");

        // Convert parameters to ApiParameter format
        let path_params: Vec<ApiParameter> = path_params_info
            .iter()
            .map(|p| {
                let param_type = match &p.param_type {
                    crate::generator::api_client::ParameterType::Enum(enum_name) => {
                        enum_name.clone()
                    }
                    crate::generator::api_client::ParameterType::Array(item_type) => {
                        format!("{}[]", item_type)
                    }
                    crate::generator::api_client::ParameterType::String => "string".to_string(),
                    crate::generator::api_client::ParameterType::Number => "number".to_string(),
                    crate::generator::api_client::ParameterType::Integer => "number".to_string(),
                    crate::generator::api_client::ParameterType::Boolean => "boolean".to_string(),
                };
                ApiParameter::new(p.name.clone(), param_type, false, p.description.clone())
            })
            .collect();

        let query_params: Vec<ApiParameter> = query_params_info
            .iter()
            .map(|p| {
                let param_type = match &p.param_type {
                    crate::generator::api_client::ParameterType::Enum(enum_name) => {
                        enum_name.clone()
                    }
                    crate::generator::api_client::ParameterType::Array(item_type) => {
                        format!("{}[]", item_type)
                    }
                    crate::generator::api_client::ParameterType::String => "string".to_string(),
                    crate::generator::api_client::ParameterType::Number => "number".to_string(),
                    crate::generator::api_client::ParameterType::Integer => "number".to_string(),
                    crate::generator::api_client::ParameterType::Boolean => "boolean".to_string(),
                };
                ApiParameter::new(p.name.clone(), param_type, true, p.description.clone())
            })
            .collect();

        // Get body type for mutations
        let body_type = request_body_info.as_ref().map(|(bt, _)| {
            if bt == "any" {
                "any".to_string()
            } else if common_schemas.contains(bt) {
                format!("Common.{}", bt)
            } else {
                let namespace_name = to_pascal_case(&module_name.replace("/", "_"));
                format!("{}.{}", namespace_name, bt)
            }
        });

        // Get description
        let description = operation
            .description
            .clone()
            .or_else(|| operation.summary.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_default();

        // Build path parameter names (for mutations)
        let path_param_names: Vec<String> =
            path_params_info.iter().map(|p| p.name.clone()).collect();
        let path_param_names_str = path_param_names.join(", ");

        // Generate schema imports
        let mut schema_imports = String::new();
        let mut needs_common_import = false;
        let mut needs_namespace_import = false;
        let namespace_name = to_pascal_case(&module_name.replace("/", "_"));
        let needs_enum_import = !enum_types.is_empty();

        // Check if body type needs import
        if let Some((body_type, _)) = &request_body_info {
            if body_type != "any" {
                if common_schemas.contains(body_type) {
                    needs_common_import = true;
                } else {
                    needs_namespace_import = true;
                }
            }
        }

        // Calculate schema import path using actual schemas_dir from config
        // From: src/hooks/{module}/useX.ts
        // To: {schemas_dir}/{module}/index.ts
        // Note: hooks_dir and schemas_dir don't include spec_name (it's in config if needed)
        let module_depth = module_name.matches('/').count() + 1; // +1 for module directory
        let hooks_depth = 1; // hooks directory
        let total_depth = module_depth + hooks_depth;

        let schemas_import_base = if let Some(schemas) = schemas_dir {
            // Calculate relative path from hooks/{module}/ to {schemas_dir}/{module}/
            let hooks_path = format!("src/hooks/{}", module_name);

            let common_prefix = HookContext::find_common_prefix(&hooks_path, schemas);
            let hooks_relative = hooks_path
                .strip_prefix(&common_prefix)
                .unwrap_or(&hooks_path)
                .trim_start_matches('/');
            let schemas_relative = schemas
                .strip_prefix(&common_prefix)
                .unwrap_or(schemas)
                .trim_start_matches('/');

            let hooks_depth_from_common = if hooks_relative.is_empty() {
                0
            } else {
                hooks_relative.matches('/').count() + 1
            };

            if schemas_relative.is_empty() {
                "../".repeat(hooks_depth_from_common)
            } else {
                format!(
                    "{}{}",
                    "../".repeat(hooks_depth_from_common),
                    schemas_relative
                )
            }
        } else {
            // Fallback: assume schemas is at src/schemas/{spec}/{module}
            format!("{}schemas", "../".repeat(total_depth))
        };

        // Check if schemas_dir includes spec_name
        let schemas_dir_includes_spec =
            if let (Some(schemas), Some(spec)) = (schemas_dir, spec_name) {
                let schemas_normalized = schemas.trim_end_matches('/');
                let spec_normalized = crate::generator::utils::sanitize_module_name(spec);
                schemas_normalized.ends_with(&spec_normalized)
                    || schemas_normalized.ends_with(&format!("/{}", spec_normalized))
            } else {
                false
            };

        if needs_common_import {
            let common_import = if schemas_dir_includes_spec {
                // Spec name is already in schemas_dir path
                format!("{}/common", schemas_import_base.trim_end_matches('/'))
            } else {
                // Spec name is NOT in schemas_dir path, so schemas are at {schemas_dir}/common
                // Don't add spec name to import path
                format!("{}/common", schemas_import_base.trim_end_matches('/'))
            };
            schema_imports.push_str(&format!("import * as Common from \"{}\";", common_import));
        }
        if needs_namespace_import {
            let sanitized_module_name = crate::generator::utils::sanitize_module_name(module_name);
            let schemas_import = if schemas_dir_includes_spec {
                // Spec name is already in schemas_dir path
                format!(
                    "{}/{}",
                    schemas_import_base.trim_end_matches('/'),
                    sanitized_module_name
                )
            } else {
                // Spec name is NOT in schemas_dir path, so schemas are at {schemas_dir}/{module}
                // Don't add spec name to import path
                format!(
                    "{}/{}",
                    schemas_import_base.trim_end_matches('/'),
                    sanitized_module_name
                )
            };
            if !schema_imports.is_empty() {
                schema_imports.push('\n');
            }
            schema_imports.push_str(&format!(
                "import * as {} from \"{}\";",
                namespace_name, schemas_import
            ));
        }

        // Add enum type imports if we have any
        // Enum types are now generated in schema files, so import from schemas
        if needs_enum_import {
            // Ensure we have namespace import for enums
            if !needs_namespace_import {
                let sanitized_module_name =
                    crate::generator::utils::sanitize_module_name(module_name);
                let schemas_import = if schemas_dir_includes_spec {
                    format!(
                        "{}/{}",
                        schemas_import_base.trim_end_matches('/'),
                        sanitized_module_name
                    )
                } else {
                    // Spec name is NOT in schemas_dir path
                    format!(
                        "{}/{}",
                        schemas_import_base.trim_end_matches('/'),
                        sanitized_module_name
                    )
                };
                if !schema_imports.is_empty() {
                    schema_imports.push('\n');
                }
                schema_imports.push_str(&format!(
                    "import * as {} from \"{}\";",
                    namespace_name, schemas_import
                ));
            }
            // Enums are now imported via namespace (e.g., Orders.SortByEnum)
            // No need for separate enum import since they're in the namespace
        }

        // Build hook context
        let context = HookContext {
            hook_name: hook_name.clone(),
            key_name,
            operation_id,
            http_method: op_info.method.clone(),
            path: op_info.path.clone(),
            path_params,
            query_params,
            body_type,
            response_type,
            module_name: module_name.to_string(),
            spec_name: spec_name.map(|s| s.to_string()),
            api_import_path: HookContext::calculate_api_import_path(
                module_name,
                spec_name,
                apis_dir,
            ),
            query_keys_import_path: HookContext::calculate_query_keys_import_path(
                module_name,
                spec_name,
                hooks_dir,
                query_keys_dir,
            ),
            param_list,
            param_names,
            path_param_names: path_param_names_str,
            schema_imports,
            description,
            success_map_type,
            error_map_type,
            generic_result_type,
            import_runtime_path: HookContext::calculate_runtime_import_path(module_name, spec_name),
        };

        // Render template
        let template_id = if is_query {
            TemplateId::ReactQueryQuery
        } else {
            TemplateId::ReactQueryMutation
        };

        let content = template_engine.render(template_id, &context)?;

        // Generate filename
        let filename = format!("{}.ts", hook_name);

        hooks.push(HookFile { filename, content });
    }

    Ok(hooks)
}

/// Generate hook name from path and method (fallback when operation_id is missing).
fn generate_hook_name_from_path(path: &str, method: &str) -> String {
    let path_parts: Vec<&str> = path
        .trim_start_matches('/')
        .split('/')
        .filter(|p| !p.starts_with('{'))
        .collect();

    let method_upper = method.to_uppercase();
    let method_prefix = match method_upper.as_str() {
        "GET" => "get",
        "POST" => "create",
        "PUT" => "update",
        "DELETE" => "delete",
        "PATCH" => "patch",
        _ => {
            return to_camel_case(&method.to_lowercase());
        }
    };

    let base_name = if path_parts.is_empty() {
        method_prefix.to_string()
    } else {
        let resource_name = if path_parts.len() > 1 {
            path_parts.last().unwrap_or(&"")
        } else {
            path_parts.first().unwrap_or(&"")
        };

        if resource_name.ends_with('s') && path.contains('{') {
            let singular = &resource_name[..resource_name.len() - 1];
            format!("{}{}ById", method_prefix, to_pascal_case(singular))
        } else if path.contains('{') {
            format!("{}{}ById", method_prefix, to_pascal_case(resource_name))
        } else {
            format!("{}{}", method_prefix, to_pascal_case(resource_name))
        }
    };

    to_camel_case(&base_name)
}
