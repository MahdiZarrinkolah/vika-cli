use crate::error::Result;
use crate::generator::swagger_parser::{
    get_schema_name_from_ref, resolve_parameter_ref, resolve_request_body_ref,
    resolve_response_ref, OperationInfo,
};
use crate::generator::utils::{to_camel_case, to_pascal_case};
use openapiv3::OpenAPI;
use openapiv3::{Operation, Parameter, ReferenceOr};

pub struct ApiFunction {
    pub content: String,
}

pub fn generate_api_client(
    openapi: &OpenAPI,
    operations: &[OperationInfo],
    module_name: &str,
    common_schemas: &[String],
) -> Result<Vec<ApiFunction>> {
    let mut functions = Vec::new();

    for op_info in operations {
        let func = generate_function_for_operation(openapi, op_info, module_name, common_schemas)?;
        functions.push(func);
    }

    Ok(functions)
}

fn generate_function_for_operation(
    openapi: &OpenAPI,
    op_info: &OperationInfo,
    module_name: &str,
    common_schemas: &[String],
) -> Result<ApiFunction> {
    let operation = &op_info.operation;
    let method = op_info.method.to_lowercase();

    // Generate function name from operation ID or path
    let func_name = if let Some(operation_id) = &operation.operation_id {
        to_camel_case(operation_id)
    } else {
        generate_function_name_from_path(&op_info.path, &op_info.method)
    };

    // Extract path parameters
    let path_params = extract_path_parameters(openapi, operation)?;

    // Extract query parameters
    let query_params = extract_query_parameters(openapi, operation)?;

    // Extract request body
    let request_body = extract_request_body(openapi, operation)?;

    // Extract response type
    let response_type = extract_response_type(openapi, operation)?;

    // Calculate namespace name for qualified type access
    // Replace slashes with underscore and convert to PascalCase (e.g., "tenant/auth" -> "TenantAuth")
    let namespace_name = to_pascal_case(&module_name.replace("/", "_"));

    // Build function signature
    let mut params = Vec::new();
    let mut path_template = op_info.path.clone();

    // Add path parameters
    for param in &path_params {
        params.push(format!("{}: string", param));
        path_template =
            path_template.replace(&format!("{{{}}}", param), &format!("${{{}}}", param));
    }

    // Add query parameters
    if !query_params.is_empty() {
        let query_fields: Vec<String> = query_params
            .iter()
            .map(|p| format!("{}?: string", p))
            .collect();
        let query_type = format!("{{ {} }}", query_fields.join(", "));
        params.push(format!("query?: {}", query_type));
    }

    // Add request body (check if it's in common schemas)
    if let Some(body_type) = &request_body {
        // Don't qualify "any" type with namespace
        if body_type == "any" {
            params.push("body: any".to_string());
        } else {
            let qualified_body_type = if common_schemas.contains(body_type) {
                format!("Common.{}", body_type)
            } else {
                format!("{}.{}", namespace_name, body_type)
            };
            params.push(format!("body: {}", qualified_body_type));
        }
    }

    let params_str = params.join(", ");

    // Build function body
    let mut body_lines = Vec::new();

    // Build URL with path parameters
    let mut url_template = op_info.path.clone();
    for param in &path_params {
        url_template = url_template.replace(&format!("{{{}}}", param), &format!("${{{}}}", param));
    }

    // Build URL with query parameters
    if !query_params.is_empty() {
        body_lines.push("    const queryString = new URLSearchParams();".to_string());
        for param in &query_params {
            body_lines.push(format!(
                "    if (query?.{}) queryString.append(\"{}\", query.{});",
                param, param, param
            ));
        }
        body_lines.push("    const queryStr = queryString.toString();".to_string());
        body_lines.push(format!(
            "    const url = `{}` + (queryStr ? `?${{queryStr}}` : '');",
            url_template
        ));
    } else {
        body_lines.push(format!("    const url = `{}`;", url_template));
    }

    // Build HTTP call
    let http_method = match method.to_uppercase().as_str() {
        "GET" => "get",
        "POST" => "post",
        "PUT" => "put",
        "DELETE" => "delete",
        "PATCH" => "patch",
        "HEAD" => "head",
        "OPTIONS" => "options",
        _ => "get",
    };

    // Use qualified type for generic parameter (check if it's common or module-specific)
    let qualified_response_type_for_generic = if response_type != "any" {
        let is_common = common_schemas.contains(&response_type);
        if is_common {
            format!("Common.{}", response_type)
        } else {
            format!("{}.{}", namespace_name, response_type)
        }
    } else {
        response_type.clone()
    };

    if let Some(_body_type) = &request_body {
        body_lines.push(format!("    return http.{}(url, body);", http_method));
    } else {
        body_lines.push(format!(
            "    return http.{}<{}>(url);",
            http_method, qualified_response_type_for_generic
        ));
    }

    // HTTP client is at apis/http.ts, and we're generating apis/<module>/index.ts
    // So the relative path is ../http
    let http_import = "../http";

    // Determine if response type is in common schemas or module-specific
    let (type_imports, qualified_type) = if response_type != "any" {
        let is_common = common_schemas.contains(&response_type);
        if is_common {
            // Import from common module
            let common_import = "../../schemas/common";
            let common_namespace = "Common";
            let imports = format!(
                "import * as {} from \"{}\";\n",
                common_namespace, common_import
            );
            let qualified = format!("{}.{}", common_namespace, response_type);
            (imports, qualified)
        } else {
            // Import from module-specific schemas
            let schemas_import = format!("../../schemas/{}", module_name);
            let imports = format!(
                "import * as {} from \"{}\";\n",
                namespace_name, schemas_import
            );
            let qualified = format!("{}.{}", namespace_name, response_type);
            (imports, qualified)
        }
    } else {
        (String::new(), String::new())
    };

    let return_type = if response_type == "any" {
        String::new()
    } else {
        format!(": Promise<{}>", qualified_type)
    };

    let function_body = body_lines.join("\n");

    let content = if params_str.is_empty() {
        format!(
            "import {{ http }} from \"{}\";\n{}\
            export const {} = async (){} => {{\n{}\n}};",
            http_import, type_imports, func_name, return_type, function_body
        )
    } else {
        format!(
            "import {{ http }} from \"{}\";\n{}\
            export const {} = async ({}){} => {{\n{}\n}};",
            http_import, type_imports, func_name, params_str, return_type, function_body
        )
    };

    Ok(ApiFunction { content })
}

fn extract_path_parameters(openapi: &OpenAPI, operation: &Operation) -> Result<Vec<String>> {
    let mut params = Vec::new();

    for param_ref in &operation.parameters {
        match param_ref {
            ReferenceOr::Reference { reference } => {
                // Resolve parameter reference (with support for nested references up to 3 levels)
                let mut current_ref = Some(reference.clone());
                let mut depth = 0;
                while let Some(ref_path) = current_ref.take() {
                    if depth > 3 {
                        break; // Prevent infinite loops
                    }
                    match resolve_parameter_ref(openapi, &ref_path) {
                        Ok(ReferenceOr::Item(param)) => {
                            if let Parameter::Path { parameter_data, .. } = param {
                                params.push(parameter_data.name.clone());
                            }
                            break;
                        }
                        Ok(ReferenceOr::Reference {
                            reference: nested_ref,
                        }) => {
                            current_ref = Some(nested_ref);
                            depth += 1;
                        }
                        Err(_) => {
                            // Reference resolution failed - skip
                            break;
                        }
                    }
                }
            }
            ReferenceOr::Item(param) => {
                if let Parameter::Path { parameter_data, .. } = param {
                    params.push(parameter_data.name.clone());
                }
            }
        }
    }

    Ok(params)
}

fn extract_query_parameters(openapi: &OpenAPI, operation: &Operation) -> Result<Vec<String>> {
    let mut params = Vec::new();

    for param_ref in &operation.parameters {
        match param_ref {
            ReferenceOr::Reference { reference } => {
                // Resolve parameter reference (with support for nested references up to 3 levels)
                let mut current_ref = Some(reference.clone());
                let mut depth = 0;
                while let Some(ref_path) = current_ref.take() {
                    if depth > 3 {
                        break; // Prevent infinite loops
                    }
                    match resolve_parameter_ref(openapi, &ref_path) {
                        Ok(ReferenceOr::Item(param)) => {
                            if let Parameter::Query { parameter_data, .. } = param {
                                params.push(parameter_data.name.clone());
                            }
                            break;
                        }
                        Ok(ReferenceOr::Reference {
                            reference: nested_ref,
                        }) => {
                            current_ref = Some(nested_ref);
                            depth += 1;
                        }
                        Err(_) => {
                            // Reference resolution failed - skip
                            break;
                        }
                    }
                }
            }
            ReferenceOr::Item(param) => {
                if let Parameter::Query { parameter_data, .. } = param {
                    params.push(parameter_data.name.clone());
                }
            }
        }
    }

    Ok(params)
}

fn extract_request_body(openapi: &OpenAPI, operation: &Operation) -> Result<Option<String>> {
    if let Some(request_body) = &operation.request_body {
        match request_body {
            ReferenceOr::Reference { reference } => {
                // Resolve request body reference
                match resolve_request_body_ref(openapi, reference) {
                    Ok(ReferenceOr::Item(body)) => {
                        if let Some(json_media) = body.content.get("application/json") {
                            if let Some(schema_ref) = &json_media.schema {
                                match schema_ref {
                                    ReferenceOr::Reference { reference } => {
                                        if let Some(ref_name) = get_schema_name_from_ref(reference)
                                        {
                                            Ok(Some(to_pascal_case(&ref_name)))
                                        } else {
                                            Ok(Some("any".to_string()))
                                        }
                                    }
                                    ReferenceOr::Item(_schema) => {
                                        // Inline schemas: These are schema definitions embedded directly
                                        // in the request body. Generating proper types would require
                                        // recursive type generation at this point, which is complex.
                                        // For now, we use 'any' as a fallback. This can be enhanced
                                        // to generate inline types if needed.
                                        Ok(Some("any".to_string()))
                                    }
                                }
                            } else {
                                Ok(Some("any".to_string()))
                            }
                        } else {
                            Ok(Some("any".to_string()))
                        }
                    }
                    Ok(ReferenceOr::Reference { .. }) => {
                        // Nested reference - return any
                        Ok(Some("any".to_string()))
                    }
                    Err(_) => {
                        // Reference resolution failed - return any
                        Ok(Some("any".to_string()))
                    }
                }
            }
            ReferenceOr::Item(body) => {
                if let Some(json_media) = body.content.get("application/json") {
                    if let Some(schema_ref) = &json_media.schema {
                        match schema_ref {
                            ReferenceOr::Reference { reference } => {
                                if let Some(ref_name) = get_schema_name_from_ref(reference) {
                                    Ok(Some(to_pascal_case(&ref_name)))
                                } else {
                                    Ok(Some("any".to_string()))
                                }
                            }
                            ReferenceOr::Item(_schema) => {
                                // Inline schemas: These are schema definitions embedded directly
                                // in the request body. Generating proper types would require
                                // recursive type generation at this point, which is complex.
                                // For now, we use 'any' as a fallback. This can be enhanced
                                // to generate inline types if needed.
                                Ok(Some("any".to_string()))
                            }
                        }
                    } else {
                        Ok(Some("any".to_string()))
                    }
                } else {
                    Ok(Some("any".to_string()))
                }
            }
        }
    } else {
        Ok(None)
    }
}

fn extract_response_type(openapi: &OpenAPI, operation: &Operation) -> Result<String> {
    // Try to get 200 response
    if let Some(success_response) = operation
        .responses
        .responses
        .get(&openapiv3::StatusCode::Code(200))
    {
        match success_response {
            ReferenceOr::Reference { reference } => {
                // Resolve response reference
                match resolve_response_ref(openapi, reference) {
                    Ok(ReferenceOr::Item(response)) => {
                        if let Some(json_media) = response.content.get("application/json") {
                            if let Some(schema_ref) = &json_media.schema {
                                match schema_ref {
                                    ReferenceOr::Reference { reference } => {
                                        if let Some(ref_name) = get_schema_name_from_ref(reference)
                                        {
                                            Ok(to_pascal_case(&ref_name))
                                        } else {
                                            Ok("any".to_string())
                                        }
                                    }
                                    ReferenceOr::Item(_) => Ok("any".to_string()),
                                }
                            } else {
                                Ok("any".to_string())
                            }
                        } else {
                            Ok("any".to_string())
                        }
                    }
                    Ok(ReferenceOr::Reference { .. }) => {
                        // Nested reference - return any
                        Ok("any".to_string())
                    }
                    Err(_) => {
                        // Reference resolution failed - return any
                        Ok("any".to_string())
                    }
                }
            }
            ReferenceOr::Item(response) => {
                if let Some(json_media) = response.content.get("application/json") {
                    if let Some(schema_ref) = &json_media.schema {
                        match schema_ref {
                            ReferenceOr::Reference { reference } => {
                                if let Some(ref_name) = get_schema_name_from_ref(reference) {
                                    Ok(to_pascal_case(&ref_name))
                                } else {
                                    Ok("any".to_string())
                                }
                            }
                            ReferenceOr::Item(_) => Ok("any".to_string()),
                        }
                    } else {
                        Ok("any".to_string())
                    }
                } else {
                    Ok("any".to_string())
                }
            }
        }
    } else {
        Ok("any".to_string())
    }
}

fn generate_function_name_from_path(path: &str, method: &str) -> String {
    let path_parts: Vec<&str> = path
        .trim_start_matches('/')
        .split('/')
        .filter(|p| !p.starts_with('{'))
        .collect();

    // Map HTTP methods to common prefixes
    let method_upper = method.to_uppercase();
    let method_lower = method.to_lowercase();
    let method_prefix = match method_upper.as_str() {
        "GET" => "get",
        "POST" => "create",
        "PUT" => "update",
        "DELETE" => "delete",
        "PATCH" => "patch",
        _ => method_lower.as_str(),
    };

    let base_name = if path_parts.is_empty() {
        method_prefix.to_string()
    } else {
        // Extract resource name from path (usually the first or last part)
        let resource_name = if path_parts.len() > 1 {
            // For nested paths like /users/{id}/posts, use the last resource
            path_parts.last().unwrap_or(&"")
        } else {
            path_parts.first().unwrap_or(&"")
        };

        // Handle common patterns
        if resource_name.ends_with("s") && path.contains('{') {
            // Plural resource with ID: /products/{id} -> getProductById
            let singular = &resource_name[..resource_name.len() - 1];
            format!("{}{}ById", method_prefix, to_pascal_case(singular))
        } else if path.contains('{') {
            // Resource with ID: /user/{id} -> getUserById
            format!("{}{}ById", method_prefix, to_pascal_case(resource_name))
        } else {
            // No ID: /products -> getProducts
            format!("{}{}", method_prefix, to_pascal_case(resource_name))
        }
    };

    to_camel_case(&base_name)
}
