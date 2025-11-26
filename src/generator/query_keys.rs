use crate::generator::swagger_parser::OperationInfo;
use crate::generator::utils::to_camel_case;
use serde::Serialize;

/// Context for query keys generation.
#[derive(Debug, Clone, Serialize)]
pub struct QueryKeyContext {
    pub module_name: String,
    pub spec_name: Option<String>,
    pub keys: Vec<QueryKeyEntry>,
}

/// Entry for a single query key.
#[derive(Debug, Clone, Serialize)]
pub struct QueryKeyEntry {
    pub key_name: String,
    pub has_params: bool,
    pub param_list: String,      // Full parameter list with types: "id: string, name?: string"
    pub param_names: String,      // Just parameter names: "id, name"
}

/// Generate query keys context from operations.
pub fn generate_query_keys(
    operations: &[OperationInfo],
    module_name: &str,
    spec_name: Option<&str>,
) -> QueryKeyContext {
    let mut keys = Vec::new();

    for op_info in operations {
        // Generate key name from operation ID or path/method
        let key_name = if let Some(operation_id) = &op_info.operation.operation_id {
            to_camel_case(operation_id)
        } else {
            generate_key_name_from_path(&op_info.path, &op_info.method)
        };

        // Extract parameters for the key
        let mut params = Vec::new();
        let mut param_names = Vec::new();
        
        // Add path parameters
        for param_ref in &op_info.operation.parameters {
            if let openapiv3::ReferenceOr::Item(openapiv3::Parameter::Path { parameter_data, .. }) = param_ref {
                let param_type = extract_param_type(parameter_data);
                params.push(format!("{}: {}", parameter_data.name, param_type));
                param_names.push(parameter_data.name.clone());
            }
        }

        // Add query parameters as a single object (only for queries, not mutations)
        let is_query = matches!(op_info.method.to_uppercase().as_str(), "GET" | "HEAD");
        let mut query_fields = Vec::new();
        if is_query {
            for param_ref in &op_info.operation.parameters {
                if let openapiv3::ReferenceOr::Item(openapiv3::Parameter::Query { parameter_data, .. }) = param_ref {
                    let param_type = extract_param_type(parameter_data);
                    query_fields.push(format!("{}?: {}", parameter_data.name, param_type));
                }
            }
        }

        // If we have query parameters, add them as a single query object
        if !query_fields.is_empty() {
            let query_type = format!("{{ {} }}", query_fields.join(", "));
            params.push(format!("query?: {}", query_type));
            param_names.push("query".to_string());
        }

        let has_params = !params.is_empty();
        let param_list = params.join(", ");
        let param_names_str = param_names.join(", ");

        keys.push(QueryKeyEntry {
            key_name,
            has_params,
            param_list,
            param_names: param_names_str,
        });
    }

    QueryKeyContext {
        module_name: module_name.to_string(),
        spec_name: spec_name.map(|s| s.to_string()),
        keys,
    }
}

/// Generate key name from path and method (fallback when operation_id is missing).
fn generate_key_name_from_path(path: &str, method: &str) -> String {
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

/// Extract parameter type from parameter data.
fn extract_param_type(parameter_data: &openapiv3::ParameterData) -> String {
    match &parameter_data.format {
        openapiv3::ParameterSchemaOrContent::Schema(schema_ref) => {
            match schema_ref {
                openapiv3::ReferenceOr::Item(schema) => {
                    match &schema.schema_kind {
                        openapiv3::SchemaKind::Type(type_) => match type_ {
                            openapiv3::Type::String(_) => "string".to_string(),
                            openapiv3::Type::Number(_) => "number".to_string(),
                            openapiv3::Type::Integer(_) => "number".to_string(),
                            openapiv3::Type::Boolean(_) => "boolean".to_string(),
                            openapiv3::Type::Array(_) => "string[]".to_string(),
                            openapiv3::Type::Object(_) => "string".to_string(),
                        },
                        _ => "string".to_string(),
                    }
                }
                openapiv3::ReferenceOr::Reference { reference } => {
                    // Try to extract type name from reference
                    if let Some(ref_name) = reference.strip_prefix("#/components/schemas/") {
                        crate::generator::utils::to_pascal_case(ref_name)
                    } else {
                        "string".to_string()
                    }
                }
            }
        }
        _ => "string".to_string(),
    }
}

/// Helper to convert to PascalCase (reuse from utils).
fn to_pascal_case(s: &str) -> String {
    crate::generator::utils::to_pascal_case(s)
}

