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
pub fn generate_react_query_hooks(
    openapi: &OpenAPI,
    operations: &[OperationInfo],
    module_name: &str,
    spec_name: Option<&str>,
    common_schemas: &[String],
    enum_registry: &mut std::collections::HashMap<String, String>,
    template_engine: &TemplateEngine,
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

        // Get response type
        let success_responses: Vec<ResponseInfo> = all_responses
            .iter()
            .filter(|r| r.status_code >= 200 && r.status_code < 300)
            .cloned()
            .collect();
        let response_type = success_responses
            .iter()
            .find(|r| r.status_code == 200)
            .map(|r| r.body_type.clone())
            .unwrap_or_else(|| "any".to_string());

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

        // Add query parameters (only for queries)
        if is_query && !query_params_info.is_empty() {
            let mut query_fields = Vec::new();
            for param in &query_params_info {
                let param_type = match &param.param_type {
                    crate::generator::api_client::ParameterType::Enum(enum_name) => {
                        enum_types.push(enum_name.clone());
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

        // Calculate schema import depth
        // From: src/hooks/{spec}/{module}/useX.ts
        // To: src/schemas/{spec}/{module}/
        // Structure: hooks/{spec}/{module}/useX.ts -> go up to hooks/, then to src/, then into schemas/
        let module_depth = module_name.matches('/').count() + 1; // +1 for module directory
        let spec_depth = if spec_name.is_some() { 1 } else { 0 };
        let total_depth = module_depth + spec_depth + 1; // +1 for hooks directory
        let schemas_depth = total_depth; // hooks/ and schemas/ are both under src/, so same depth

        if needs_common_import {
            let common_import = if let Some(spec) = spec_name {
                format!(
                    "{}schemas/{}/common",
                    "../".repeat(schemas_depth),
                    crate::generator::utils::sanitize_module_name(spec)
                )
            } else {
                format!("{}schemas/common", "../".repeat(schemas_depth))
            };
            schema_imports.push_str(&format!("import * as Common from \"{}\";", common_import));
        }
        if needs_namespace_import {
            let sanitized_module_name = crate::generator::utils::sanitize_module_name(module_name);
            let schemas_import = if let Some(spec) = spec_name {
                format!(
                    "{}schemas/{}/{}",
                    "../".repeat(schemas_depth),
                    crate::generator::utils::sanitize_module_name(spec),
                    sanitized_module_name
                )
            } else {
                format!(
                    "{}schemas/{}",
                    "../".repeat(schemas_depth),
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
        if needs_enum_import {
            let sanitized_module_name = crate::generator::utils::sanitize_module_name(module_name);
            let schemas_import = if let Some(spec) = spec_name {
                format!(
                    "{}schemas/{}/{}",
                    "../".repeat(schemas_depth),
                    crate::generator::utils::sanitize_module_name(spec),
                    sanitized_module_name
                )
            } else {
                format!(
                    "{}schemas/{}",
                    "../".repeat(schemas_depth),
                    sanitized_module_name
                )
            };
            if !schema_imports.is_empty() {
                schema_imports.push('\n');
            }
            let enum_names: Vec<String> = enum_types.to_vec();
            schema_imports.push_str(&format!(
                "import type {{ {} }} from \"{}\";",
                enum_names.join(", "),
                schemas_import
            ));
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
            api_import_path: HookContext::calculate_api_import_path(module_name, spec_name),
            query_keys_import_path: HookContext::calculate_query_keys_import_path(
                module_name,
                spec_name,
            ),
            param_list,
            param_names,
            path_param_names: path_param_names_str,
            schema_imports,
            description,
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
