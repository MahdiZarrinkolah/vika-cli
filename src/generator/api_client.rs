use crate::error::Result;
use crate::generator::swagger_parser::{
    get_schema_name_from_ref, resolve_parameter_ref, resolve_request_body_ref,
    resolve_response_ref, OperationInfo,
};
use crate::generator::swagger_parser::resolve_ref;
use crate::generator::ts_typings::TypeScriptType;
use crate::generator::utils::{to_camel_case, to_pascal_case, sanitize_module_name};
use openapiv3::OpenAPI;
use openapiv3::{Operation, Parameter, ReferenceOr, SchemaKind, Type};

pub struct ApiFunction {
    pub content: String,
}

pub struct ApiGenerationResult {
    pub functions: Vec<ApiFunction>,
    pub response_types: Vec<TypeScriptType>,
}

#[derive(Clone, Debug)]
pub struct ParameterInfo {
    pub name: String,
    pub param_type: ParameterType,
    pub enum_values: Option<Vec<String>>,
    pub enum_type_name: Option<String>,
    pub is_array: bool,
    pub array_item_type: Option<String>,
    pub style: Option<String>,
    pub explode: Option<bool>,
}

#[derive(Clone, Debug)]
pub enum ParameterType {
    String,
    Number,
    Integer,
    Boolean,
    Enum(String), // enum type name
    Array(String), // array item type
}

#[derive(Clone, Debug)]
pub struct ResponseInfo {
    pub status_code: u16,
    pub body_type: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ErrorResponse {
    pub status_code: u16,
    pub body_type: String,
}

pub fn generate_api_client(
    openapi: &OpenAPI,
    operations: &[OperationInfo],
    module_name: &str,
    common_schemas: &[String],
) -> Result<ApiGenerationResult> {
    generate_api_client_with_registry(openapi, operations, module_name, common_schemas, &mut std::collections::HashMap::new())
}

pub fn generate_api_client_with_registry(
    openapi: &OpenAPI,
    operations: &[OperationInfo],
    module_name: &str,
    common_schemas: &[String],
    enum_registry: &mut std::collections::HashMap<String, String>,
) -> Result<ApiGenerationResult> {
    let mut functions = Vec::new();
    let mut response_types = Vec::new();

    for op_info in operations {
        let result = generate_function_for_operation(openapi, op_info, module_name, common_schemas, enum_registry)?;
        functions.push(result.function);
        response_types.extend(result.response_types);
    }

    Ok(ApiGenerationResult {
        functions,
        response_types,
    })
}

struct FunctionGenerationResult {
    function: ApiFunction,
    response_types: Vec<TypeScriptType>,
}

fn generate_function_for_operation(
    openapi: &OpenAPI,
    op_info: &OperationInfo,
    module_name: &str,
    common_schemas: &[String],
    enum_registry: &mut std::collections::HashMap<String, String>,
) -> Result<FunctionGenerationResult> {
    let operation = &op_info.operation;
    let method = op_info.method.to_lowercase();

    // Generate function name from operation ID or path
    let func_name = if let Some(operation_id) = &operation.operation_id {
        to_camel_case(operation_id)
    } else {
        generate_function_name_from_path(&op_info.path, &op_info.method)
    };

    // Extract path parameters
    let path_params = extract_path_parameters(openapi, operation, enum_registry)?;

    // Extract query parameters
    let query_params = extract_query_parameters(openapi, operation, enum_registry)?;

    // Extract request body
    let request_body = extract_request_body(openapi, operation)?;

    // Extract all responses (success + error)
    let all_responses = extract_all_responses(openapi, operation)?;
    
    // Separate success and error responses
    let success_responses: Vec<ResponseInfo> = all_responses
        .iter()
        .filter(|r| r.status_code >= 200 && r.status_code < 300)
        .cloned()
        .collect();
    let error_responses: Vec<ResponseInfo> = all_responses
        .iter()
        .filter(|r| r.status_code < 200 || r.status_code >= 300)
        .cloned()
        .collect();
    
    // Get primary success response type (for backward compatibility)
    let response_type = success_responses
        .iter()
        .find(|r| r.status_code == 200)
        .map(|r| r.body_type.clone())
        .unwrap_or_else(|| "any".to_string());

    // Calculate namespace name for qualified type access
    // Replace slashes with underscore and convert to PascalCase (e.g., "tenant/auth" -> "TenantAuth")
    let namespace_name = to_pascal_case(&module_name.replace("/", "_"));

    // Build function signature
    let mut params = Vec::new();
    let mut path_template = op_info.path.clone();
    let mut enum_types = Vec::new();

    // Add path parameters
    for param in &path_params {
        let param_type = match &param.param_type {
            ParameterType::Enum(enum_name) => {
                enum_types.push((enum_name.clone(), param.enum_values.clone().unwrap_or_default()));
                enum_name.clone()
            }
            ParameterType::String => "string".to_string(),
            ParameterType::Number => "number".to_string(),
            ParameterType::Integer => "number".to_string(),
            ParameterType::Boolean => "boolean".to_string(),
            ParameterType::Array(_) => "string".to_string(), // Arrays in path are serialized as strings
        };
        params.push(format!("{}: {}", param.name, param_type));
        path_template =
            path_template.replace(&format!("{{{}}}", param.name), &format!("${{{}}}", param.name));
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

    // Add query parameters (optional) AFTER any required parameters like body,
    // to satisfy TypeScript's \"required parameter cannot follow an optional parameter\" rule.
    if !query_params.is_empty() {
        let mut query_fields = Vec::new();
        for param in &query_params {
            let param_type = match &param.param_type {
                ParameterType::Enum(enum_name) => {
                    enum_types
                        .push((enum_name.clone(), param.enum_values.clone().unwrap_or_default()));
                    enum_name.clone()
                }
                ParameterType::Array(item_type) => {
                    format!("{}[]", item_type)
                }
                ParameterType::String => "string".to_string(),
                ParameterType::Number => "number".to_string(),
                ParameterType::Integer => "number".to_string(),
                ParameterType::Boolean => "boolean".to_string(),
            };
            query_fields.push(format!("{}?: {}", param.name, param_type));
        }
        let query_type = format!("{{ {} }}", query_fields.join(", "));
        params.push(format!("query?: {}", query_type));
    }

    let params_str = params.join(", ");

    // Build function body
    let mut body_lines = Vec::new();

    // Build URL with path parameters
    let mut url_template = op_info.path.clone();
    for param in &path_params {
        url_template = url_template.replace(&format!("{{{}}}", param.name), &format!("${{{}}}", param.name));
    }

    // Build URL with query parameters
    if !query_params.is_empty() {
        body_lines.push("    const queryString = new URLSearchParams();".to_string());
        for param in &query_params {
            if param.is_array {
                let explode = param.explode.unwrap_or(true);
                if explode {
                    // explode: true -> tags=one&tags=two
                    body_lines.push(format!(
                        "    if (query?.{}) {{",
                        param.name
                    ));
                    body_lines.push(format!(
                        "      query.{}.forEach((item) => queryString.append(\"{}\", String(item)));",
                        param.name, param.name
                    ));
                    body_lines.push("    }".to_string());
                } else {
                    // explode: false -> tags=one,two
                    body_lines.push(format!(
                        "    if (query?.{}) queryString.append(\"{}\", query.{}.join(\",\"));",
                        param.name, param.name, param.name
                    ));
                }
            } else {
                body_lines.push(format!(
                    "    if (query?.{}) queryString.append(\"{}\", String(query.{}));",
                    param.name, param.name, param.name
                ));
            }
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
    // Calculate relative path based on module depth
    let depth = module_name.matches('/').count();
    let http_relative_path = if depth == 0 {
        "../http"
    } else {
        &format!("{}../http", "../".repeat(depth))
    };
    let http_import = http_relative_path;

    // Determine if response type is in common schemas or module-specific
    // We still need schema imports for request/response body types
    let mut type_imports = String::new();
    let mut needs_common_import = false;
    let mut needs_namespace_import = false;
    
    // Check if response type needs import
    if response_type != "any" {
        let is_common = common_schemas.contains(&response_type);
        if is_common {
            needs_common_import = true;
        } else {
            needs_namespace_import = true;
        }
    }
    
    // Check if request body type needs import
    if let Some(body_type) = &request_body {
        if body_type != "any" {
            if common_schemas.contains(body_type) {
                needs_common_import = true;
            } else {
                needs_namespace_import = true;
            }
        }
    }
    
    // Add imports
    if needs_common_import {
        let schemas_depth = depth + 1; // +1 to go from apis/ to schemas/
        let common_import = format!("{}../schemas/common", "../".repeat(schemas_depth));
        type_imports.push_str(&format!(
            "import * as Common from \"{}\";\n",
            common_import
        ));
    }
    if needs_namespace_import {
        let schemas_depth = depth + 1; // +1 to go from apis/ to schemas/
        let sanitized_module_name = sanitize_module_name(module_name);
        let schemas_import = format!("{}../schemas/{}", "../".repeat(schemas_depth), sanitized_module_name);
        type_imports.push_str(&format!(
            "import * as {} from \"{}\";\n",
            namespace_name, schemas_import
        ));
    }

    // Generate response types (Errors, Error union, Responses)
    let response_types = generate_response_types(
        &func_name,
        &success_responses,
        &error_responses,
        &namespace_name,
        common_schemas,
        &enum_types,
    );
    
    // Add imports for response types if we have any
    let type_name_base = to_pascal_case(&func_name);
    let mut response_type_imports = Vec::new();
    
    // Only add error types if we have errors with schemas
    let errors_with_schemas: Vec<&ResponseInfo> = error_responses
        .iter()
        .filter(|r| r.status_code > 0)
        .collect();
    if !errors_with_schemas.is_empty() {
        response_type_imports.push(format!("{}Errors", type_name_base));
        response_type_imports.push(format!("{}Error", type_name_base));
    }
    
    // Only add Responses type if we have success responses with schemas
    let success_with_schemas: Vec<&ResponseInfo> = success_responses
        .iter()
        .filter(|r| r.status_code >= 200 && r.status_code < 300 && r.body_type != "any")
        .collect();
    if !success_with_schemas.is_empty() {
        response_type_imports.push(format!("{}Responses", type_name_base));
    }
    
    // Add type import if we have response types (separate line)
    if !response_type_imports.is_empty() {
        // Calculate relative path based on module depth
        let schemas_depth = depth + 1; // +1 to go from apis/ to schemas/
        let sanitized_module_name = sanitize_module_name(module_name);
        let schemas_import = format!("{}../schemas/{}", "../".repeat(schemas_depth), sanitized_module_name);
        let type_import_line = format!(
            "import type {{ {} }} from \"{}\";",
            response_type_imports.join(", "),
            schemas_import
        );
        if type_imports.is_empty() {
            type_imports = format!("{}\n", type_import_line);
        } else {
            type_imports = format!("{}\n{}", type_imports.trim_end(), type_import_line);
        }
    }
    
    // Add enum type imports if we have any
    if !enum_types.is_empty() {
        let schemas_depth = depth + 1; // +1 to go from apis/ to schemas/
        let sanitized_module_name = sanitize_module_name(module_name);
        let schemas_import = format!("{}../schemas/{}", "../".repeat(schemas_depth), sanitized_module_name);
        let enum_names: Vec<String> = enum_types.iter().map(|(name, _)| name.clone()).collect();
        let enum_import_line = format!(
            "import type {{ {} }} from \"{}\";",
            enum_names.join(", "),
            schemas_import
        );
        if type_imports.is_empty() {
            type_imports = format!("{}\n", enum_import_line);
        } else {
            type_imports = format!("{}\n{}", type_imports.trim_end(), enum_import_line);
        }
    }
    
    // Ensure type_imports ends with newline for proper separation
    if !type_imports.is_empty() && !type_imports.ends_with('\n') {
        type_imports.push('\n');
    }

    // Determine return type - use Responses type if available, otherwise fallback to direct type
    let has_responses_type = response_type_imports.iter().any(|imp| imp.contains("Responses"));
    let return_type = if has_responses_type {
        // Use Responses type with primary status code
        if let Some(_primary_response) = success_responses.iter().find(|r| r.status_code == 200 && r.body_type != "any") {
            format!(": Promise<{}Responses[200]>", type_name_base)
        } else if let Some(first_success) = success_responses.iter().find(|r| r.status_code >= 200 && r.status_code < 300 && r.body_type != "any") {
            format!(": Promise<{}Responses[{}]>", type_name_base, first_success.status_code)
        } else {
            String::new()
        }
    } else if !success_responses.is_empty() {
        // Fallback to direct type if no Responses type generated
        if let Some(primary_response) = success_responses.iter().find(|r| r.status_code == 200) {
            if primary_response.body_type != "any" {
                let qualified = if common_schemas.contains(&primary_response.body_type) {
                    format!("Common.{}", primary_response.body_type)
                } else {
                    format!("{}.{}", namespace_name, primary_response.body_type)
                };
                format!(": Promise<{}>", qualified)
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let function_body = body_lines.join("\n");

    // Remove inline type definitions - they'll be in types.ts
    // Separate imports from function definition
    let content = if params_str.is_empty() {
        format!(
            "import {{ http }} from \"{}\";\n{}{}export const {} = async (){} => {{\n{}\n}};",
            http_import, type_imports, if !type_imports.is_empty() { "\n" } else { "" }, func_name, return_type, function_body
        )
    } else {
        format!(
            "import {{ http }} from \"{}\";\n{}{}export const {} = async ({}){} => {{\n{}\n}};",
            http_import, type_imports, if !type_imports.is_empty() { "\n" } else { "" }, func_name, params_str, return_type, function_body
        )
    };

    Ok(FunctionGenerationResult {
        function: ApiFunction { content },
        response_types,
    })
}

fn extract_path_parameters(
    openapi: &OpenAPI,
    operation: &Operation,
    enum_registry: &mut std::collections::HashMap<String, String>,
) -> Result<Vec<ParameterInfo>> {
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
                                if let Some(param_info) = extract_parameter_info(openapi, &parameter_data, enum_registry)? {
                                    params.push(param_info);
                                }
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
                    if let Some(param_info) = extract_parameter_info(openapi, parameter_data, enum_registry)? {
                        params.push(param_info);
                    }
                }
            }
        }
    }

    Ok(params)
}

fn extract_query_parameters(
    openapi: &OpenAPI,
    operation: &Operation,
    enum_registry: &mut std::collections::HashMap<String, String>,
) -> Result<Vec<ParameterInfo>> {
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
                            if let Parameter::Query { parameter_data, style, .. } = param {
                                if let Some(mut param_info) = extract_parameter_info(openapi, &parameter_data, enum_registry)? {
                                    // Override style and explode for query parameters
                                    param_info.style = Some(format!("{:?}", style));
                                    // explode defaults to true for arrays, false otherwise
                                    param_info.explode = Some(parameter_data.explode.unwrap_or(false));
                                    params.push(param_info);
                                }
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
                if let Parameter::Query { parameter_data, style, .. } = param {
                    if let Some(mut param_info) = extract_parameter_info(openapi, parameter_data, enum_registry)? {
                        // Override style and explode for query parameters
                        param_info.style = Some(format!("{:?}", style));
                        // explode defaults to true for arrays, false otherwise
                        param_info.explode = Some(parameter_data.explode.unwrap_or(false));
                        params.push(param_info);
                    }
                }
            }
        }
    }

    Ok(params)
}

fn extract_parameter_info(
    openapi: &OpenAPI,
    parameter_data: &openapiv3::ParameterData,
    enum_registry: &mut std::collections::HashMap<String, String>,
) -> Result<Option<ParameterInfo>> {
    let name = parameter_data.name.clone();
    
    // Get schema from parameter
    let schema = match &parameter_data.format {
        openapiv3::ParameterSchemaOrContent::Schema(schema_ref) => {
            match schema_ref {
                ReferenceOr::Reference { reference } => {
                    resolve_ref(openapi, reference).ok().and_then(|r| match r {
                        ReferenceOr::Item(s) => Some(s),
                        _ => None,
                    })
                }
                ReferenceOr::Item(s) => Some(s.clone()),
            }
        }
        _ => None,
    };

    if let Some(schema) = schema {
        match &schema.schema_kind {
            SchemaKind::Type(type_) => {
                match type_ {
                    Type::String(string_type) => {
                        // Check for enum
                        if !string_type.enumeration.is_empty() {
                            let mut enum_values: Vec<String> = string_type
                                .enumeration
                                .iter()
                                .filter_map(|v| v.as_ref().cloned())
                                .collect();
                            enum_values.sort();
                            let enum_key = enum_values.join(",");
                            
                            // Generate enum name
                            let enum_name = format!("{}Enum", to_pascal_case(&name));
                            let context_key = format!("{}:{}", enum_key, name);
                            
                            // Check registry
                            let final_enum_name = if let Some(existing) = enum_registry.get(&context_key).or_else(|| enum_registry.get(&enum_key)) {
                                existing.clone()
                            } else {
                                enum_registry.insert(context_key.clone(), enum_name.clone());
                                enum_registry.insert(enum_key.clone(), enum_name.clone());
                                enum_name
                            };
                            
                            Ok(Some(ParameterInfo {
                                name,
                                param_type: ParameterType::Enum(final_enum_name.clone()),
                                enum_values: Some(enum_values),
                                enum_type_name: Some(final_enum_name),
                                is_array: false,
                                array_item_type: None,
                                style: Some("simple".to_string()), // default for path
                                explode: Some(false), // default for path
                            }))
                        } else {
                            Ok(Some(ParameterInfo {
                                name,
                                param_type: ParameterType::String,
                                enum_values: None,
                                enum_type_name: None,
                                is_array: false,
                                array_item_type: None,
                                style: Some("simple".to_string()),
                                explode: Some(false),
                            }))
                        }
                    }
                    Type::Number(_) => Ok(Some(ParameterInfo {
                        name,
                        param_type: ParameterType::Number,
                        enum_values: None,
                        enum_type_name: None,
                        is_array: false,
                        array_item_type: None,
                        style: Some("simple".to_string()),
                        explode: Some(false),
                    })),
                    Type::Integer(_) => Ok(Some(ParameterInfo {
                        name,
                        param_type: ParameterType::Integer,
                        enum_values: None,
                        enum_type_name: None,
                        is_array: false,
                        array_item_type: None,
                        style: Some("simple".to_string()),
                        explode: Some(false),
                    })),
                    Type::Boolean(_) => Ok(Some(ParameterInfo {
                        name,
                        param_type: ParameterType::Boolean,
                        enum_values: None,
                        enum_type_name: None,
                        is_array: false,
                        array_item_type: None,
                        style: Some("simple".to_string()),
                        explode: Some(false),
                    })),
                    Type::Object(_) => Ok(Some(ParameterInfo {
                        name,
                        param_type: ParameterType::String,
                        enum_values: None,
                        enum_type_name: None,
                        is_array: false,
                        array_item_type: None,
                        style: Some("simple".to_string()),
                        explode: Some(false),
                    })),
                    Type::Array(array) => {
                        let item_type = if let Some(items) = &array.items {
                            match items {
                                ReferenceOr::Reference { reference } => {
                                    if let Some(ref_name) = get_schema_name_from_ref(reference) {
                                        to_pascal_case(&ref_name)
                                    } else {
                                        "string".to_string()
                                    }
                                }
                                ReferenceOr::Item(item_schema) => {
                                    // Extract type from item schema
                                    match &item_schema.schema_kind {
                                        SchemaKind::Type(item_type) => {
                                            match item_type {
                                                Type::String(_) => "string".to_string(),
                                                Type::Number(_) => "number".to_string(),
                                                Type::Integer(_) => "number".to_string(),
                                                Type::Boolean(_) => "boolean".to_string(),
                                                _ => "string".to_string(),
                                            }
                                        }
                                        _ => "string".to_string(),
                                    }
                                }
                            }
                        } else {
                            "string".to_string()
                        };
                        
                        Ok(Some(ParameterInfo {
                            name,
                            param_type: ParameterType::Array(item_type.clone()),
                            enum_values: None,
                            enum_type_name: None,
                            is_array: true,
                            array_item_type: Some(item_type),
                            style: Some("form".to_string()), // default for query arrays
                            explode: Some(true), // default for query arrays
                        }))
                    }
                }
            }
            _ => Ok(Some(ParameterInfo {
                name,
                param_type: ParameterType::String,
                enum_values: None,
                enum_type_name: None,
                is_array: false,
                array_item_type: None,
                style: Some("simple".to_string()),
                explode: Some(false),
            })),
        }
    } else {
        // No schema, default to string
        Ok(Some(ParameterInfo {
            name,
            param_type: ParameterType::String,
            enum_values: None,
            enum_type_name: None,
            is_array: false,
            array_item_type: None,
            style: Some("simple".to_string()),
            explode: Some(false),
        }))
    }
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

#[allow(dead_code)]
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

fn extract_all_responses(openapi: &OpenAPI, operation: &Operation) -> Result<Vec<ResponseInfo>> {
    let mut responses = Vec::new();

    for (status_code, response_ref) in &operation.responses.responses {
        let status_num = match status_code {
            openapiv3::StatusCode::Code(code) => *code,
            openapiv3::StatusCode::Range(range) => {
                // For ranges like 4xx, 5xx, extract the range value
                // Range is an enum, check its variant
                match format!("{:?}", range).as_str() {
                    s if s.contains("4") => 400,
                    s if s.contains("5") => 500,
                    _ => 0,
                }
            }
        };

        // Extract response info (description and body type)
        let (description, body_type) = match response_ref {
            ReferenceOr::Reference { reference } => {
                match resolve_response_ref(openapi, reference) {
                    Ok(ReferenceOr::Item(response)) => {
                        let desc = response.description.clone();
                        let body = extract_response_body_type(openapi, &response);
                        (Some(desc), body)
                    }
                    _ => (None, "any".to_string()),
                }
            }
            ReferenceOr::Item(response) => {
                let desc = response.description.clone();
                let body = extract_response_body_type(openapi, response);
                (Some(desc), body)
            }
        };

        responses.push(ResponseInfo {
            status_code: status_num,
            body_type,
            description,
        });
    }

    Ok(responses)
}

#[allow(dead_code)]
fn extract_error_responses(openapi: &OpenAPI, operation: &Operation) -> Result<Vec<ErrorResponse>> {
    let all_responses = extract_all_responses(openapi, operation)?;
    let errors: Vec<ErrorResponse> = all_responses
        .iter()
        .filter(|r| r.status_code < 200 || r.status_code >= 300)
        .map(|r| ErrorResponse {
            status_code: r.status_code,
            body_type: r.body_type.clone(),
        })
        .collect();
    Ok(errors)
}

fn generate_response_types(
    func_name: &str,
    success_responses: &[ResponseInfo],
    error_responses: &[ResponseInfo],
    _namespace_name: &str,
    common_schemas: &[String],
    enum_types: &[(String, Vec<String>)],
) -> Vec<TypeScriptType> {
    let mut types = Vec::new();
    let type_name_base = to_pascal_case(func_name);
    
    // Generate enum types for parameters
    for (enum_name, enum_values) in enum_types {
        let variants = enum_values
            .iter()
            .map(|v| format!("\"{}\"", v))
            .collect::<Vec<_>>()
            .join(" |\n");
        let enum_type = format!("export type {} =\n{};", enum_name, variants);
        types.push(TypeScriptType { content: enum_type });
    }
    
    // Generate Errors type
    if !error_responses.is_empty() {
        let mut error_fields = Vec::new();
        for error in error_responses {
            if error.status_code > 0 {
                // For types in the same file (not common), use unqualified name
                // For common types, use Common.TypeName
                let qualified_type = if error.body_type != "any" {
                    if common_schemas.contains(&error.body_type) {
                        format!("Common.{}", error.body_type)
                    } else {
                        // Type is in the same file, use unqualified name
                        error.body_type.clone()
                    }
                } else {
                    "any".to_string()
                };
                
                let description = error.description.as_ref()
                    .map(|d| format!("    /**\n     * {}\n     */", d))
                    .unwrap_or_default();
                
                error_fields.push(format!("{}\n    {}: {};", description, error.status_code, qualified_type));
            }
        }
        
        if !error_fields.is_empty() {
            let errors_type = format!("export type {}Errors = {{\n{}\n}};", type_name_base, error_fields.join("\n"));
            types.push(TypeScriptType { content: errors_type });
            
            // Generate Error union type
            let error_union_type = format!("export type {}Error = {}Errors[keyof {}Errors];", 
                type_name_base, type_name_base, type_name_base);
            types.push(TypeScriptType { content: error_union_type });
        }
    }
    
    // Generate Responses type (only if we have success responses with schemas)
    let success_with_schemas: Vec<&ResponseInfo> = success_responses
        .iter()
        .filter(|r| r.status_code >= 200 && r.status_code < 300 && r.body_type != "any")
        .collect();
    
    if !success_with_schemas.is_empty() {
        let mut response_fields = Vec::new();
        for response in success_with_schemas {
            // For types in the same file (not common), use unqualified name
            // For common types, use Common.TypeName
            let qualified_type = if common_schemas.contains(&response.body_type) {
                format!("Common.{}", response.body_type)
            } else {
                // Type is in the same file, use unqualified name
                response.body_type.clone()
            };
            
            let description = response.description.as_ref()
                .map(|d| format!("    /**\n     * {}\n     */", d))
                .unwrap_or_default();
            
            response_fields.push(format!("{}\n    {}: {};", description, response.status_code, qualified_type));
        }
        
        if !response_fields.is_empty() {
            let responses_type = format!("export type {}Responses = {{\n{}\n}};", type_name_base, response_fields.join("\n"));
            types.push(TypeScriptType { content: responses_type });
        }
    }
    
    types
}

fn extract_response_body_type(_openapi: &OpenAPI, response: &openapiv3::Response) -> String {
    if let Some(json_media) = response.content.get("application/json") {
        if let Some(schema_ref) = &json_media.schema {
            match schema_ref {
                ReferenceOr::Reference { reference } => {
                    if let Some(ref_name) = get_schema_name_from_ref(reference) {
                        to_pascal_case(&ref_name)
                    } else {
                        "any".to_string()
                    }
                }
                ReferenceOr::Item(_) => "any".to_string(),
            }
        } else {
            "any".to_string()
        }
    } else {
        "any".to_string()
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

