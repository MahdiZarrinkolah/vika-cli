use anyhow::Result;
use openapiv3::{Operation, Parameter, ReferenceOr};
use crate::generator::swagger_parser::{get_schema_name_from_ref, OperationInfo};
use crate::generator::utils::{to_camel_case, to_pascal_case};
use openapiv3::OpenAPI;

pub struct ApiFunction {
    pub content: String,
}

pub fn generate_api_client(
    openapi: &OpenAPI,
    operations: &[OperationInfo],
) -> Result<Vec<ApiFunction>> {
    let mut functions = Vec::new();

    for op_info in operations {
        let func = generate_function_for_operation(openapi, op_info)?;
        functions.push(func);
    }

    Ok(functions)
}

fn generate_function_for_operation(
    openapi: &OpenAPI,
    op_info: &OperationInfo,
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
    let path_params = extract_path_parameters(operation)?;
    
    // Extract query parameters
    let query_params = extract_query_parameters(operation)?;
    
    // Extract request body
    let request_body = extract_request_body(openapi, operation)?;
    
    // Extract response type
    let response_type = extract_response_type(openapi, operation)?;

    // Build function signature
    let mut params = Vec::new();
    let mut path_template = op_info.path.clone();

    // Add path parameters
    for param in &path_params {
        params.push(format!("{}: string", param));
        path_template = path_template.replace(&format!("{{{}}}", param), &format!("${{{}}}", param));
    }

    // Add query parameters
    if !query_params.is_empty() {
        let query_type = if query_params.len() == 1 {
            format!("{{ {}: string }}", query_params[0])
        } else {
            let query_fields: Vec<String> = query_params
                .iter()
                .map(|p| format!("{}?: string", p))
                .collect();
            format!("{{ {} }}", query_fields.join(", "))
        };
        params.push(format!("query?: {}", query_type));
    }

    // Add request body
    if let Some(body_type) = &request_body {
        params.push(format!("body: {}", body_type));
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
            body_lines.push(format!("    if (query?.{}) queryString.append(\"{}\", query.{});", param, param, param));
        }
        body_lines.push(format!("    const url = `{}` + (queryString.toString() ? `?${{queryString.toString()}}` : '');", url_template));
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
        _ => "get",
    };
    
    if let Some(_body_type) = &request_body {
        body_lines.push(format!("    return http.{}(url, body);", http_method));
    } else {
        body_lines.push(format!("    return http.{}<{}>(url);", http_method, response_type));
    }

    let function_body = body_lines.join("\n");
    
    let return_type = if response_type == "any" {
        String::new()
    } else {
        format!(": Promise<{}>", response_type)
    };

    // HTTP client is at apis/http.ts, and we're generating apis/<module>/index.ts
    // So the relative path is ../http
    let http_import = "../http";
    
    let content = if params_str.is_empty() {
        format!(
            "import {{ http }} from \"{}\";\n\n\
            export const {} = async (){} => {{\n{}\n}};",
            http_import, func_name, return_type, function_body
        )
    } else {
        format!(
            "import {{ http }} from \"{}\";\n\n\
            export const {} = async ({}){} => {{\n{}\n}};",
            http_import, func_name, params_str, return_type, function_body
        )
    };

    Ok(ApiFunction {
        content,
    })
}

fn extract_path_parameters(operation: &Operation) -> Result<Vec<String>> {
    let mut params = Vec::new();
    
    for param_ref in &operation.parameters {
        match param_ref {
            ReferenceOr::Reference { reference: _ } => {
                // TODO: Resolve reference
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

fn extract_query_parameters(operation: &Operation) -> Result<Vec<String>> {
    let mut params = Vec::new();
    
    for param_ref in &operation.parameters {
        match param_ref {
            ReferenceOr::Reference { reference: _ } => {
                // TODO: Resolve reference
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

fn extract_request_body(
    _openapi: &OpenAPI,
    operation: &Operation,
) -> Result<Option<String>> {
    if let Some(request_body) = &operation.request_body {
        match request_body {
            ReferenceOr::Reference { reference: _ } => {
                // TODO: Resolve reference
                Ok(Some("any".to_string()))
            }
            ReferenceOr::Item(body) => {
                if let Some(json_media) = body.content.get("application/json") {
                    if let Some(schema_ref) = &json_media.schema {
                        match schema_ref {
                            ReferenceOr::Reference { reference } => {
                                if let Some(ref_name) = get_schema_name_from_ref(&reference) {
                                    Ok(Some(to_pascal_case(&ref_name)))
                                } else {
                                    Ok(Some("any".to_string()))
                                }
                            }
                            ReferenceOr::Item(_schema) => {
                                // For inline schemas, we'd need to generate a type
                                // For now, return a generic type name
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

fn extract_response_type(
    _openapi: &OpenAPI,
    operation: &Operation,
) -> Result<String> {
    // Try to get 200 response
    if let Some(success_response) = operation.responses.responses.get(&openapiv3::StatusCode::Code(200)) {
        match success_response {
            ReferenceOr::Reference { reference: _ } => {
                Ok("any".to_string())
            }
            ReferenceOr::Item(response) => {
                if let Some(json_media) = response.content.get("application/json") {
                    if let Some(schema_ref) = &json_media.schema {
                        match schema_ref {
                            ReferenceOr::Reference { reference } => {
                                if let Some(ref_name) = get_schema_name_from_ref(&reference) {
                                    Ok(to_pascal_case(&ref_name))
                                } else {
                                    Ok("any".to_string())
                                }
                            }
                            ReferenceOr::Item(_) => {
                                Ok("any".to_string())
                            }
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
    
    let base_name = if path_parts.is_empty() {
        method.to_lowercase()
    } else {
        format!("{}{}", method.to_lowercase(), to_pascal_case(&path_parts.join("")))
    };
    
    to_camel_case(&base_name)
}

