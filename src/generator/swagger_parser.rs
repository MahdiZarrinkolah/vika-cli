use anyhow::{Context, Result};
use openapiv3::{OpenAPI, Operation, Parameter, PathItem, ReferenceOr, Schema};
use std::collections::HashMap;

pub struct ParsedSpec {
    pub openapi: OpenAPI,
    pub modules: Vec<String>,
    pub operations_by_tag: HashMap<String, Vec<OperationInfo>>,
    pub schemas: HashMap<String, Schema>,
    pub module_schemas: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct OperationInfo {
    pub method: String,
    pub path: String,
    pub operation: Operation,
}

pub async fn fetch_and_parse_spec(spec_path: &str) -> Result<ParsedSpec> {
    let content = if spec_path.starts_with("http://") || spec_path.starts_with("https://") {
        fetch_remote_spec(spec_path).await?
    } else {
        std::fs::read_to_string(spec_path)
            .with_context(|| format!("Failed to read spec file: {}", spec_path))?
    };

    let openapi: OpenAPI = if spec_path.ends_with(".yaml") || spec_path.ends_with(".yml") {
        serde_yaml::from_str(&content)
            .context("Failed to parse YAML spec")?
    } else {
        serde_json::from_str(&content)
            .context("Failed to parse JSON spec")?
    };

    let modules = extract_modules(&openapi);
    let operations_by_tag = extract_operations_by_tag(&openapi);
    let schemas = extract_schemas(&openapi);
    let module_schemas = map_modules_to_schemas(&openapi, &operations_by_tag, &schemas)?;

    Ok(ParsedSpec {
        openapi,
        modules,
        operations_by_tag,
        schemas,
        module_schemas,
    })
}

async fn fetch_remote_spec(url: &str) -> Result<String> {
    let response = reqwest::get(url)
        .await
        .with_context(|| format!("Failed to fetch spec from: {}", url))?;
    
    response.text()
        .await
        .with_context(|| format!("Failed to read response from: {}", url))
}

pub fn extract_modules(openapi: &OpenAPI) -> Vec<String> {
    if !openapi.tags.is_empty() {
        openapi.tags.iter()
            .map(|tag| tag.name.clone())
            .collect()
    } else {
        // Extract tags from operations if tags section is missing
        let mut tag_set = std::collections::HashSet::new();
        for (_, path_item) in openapi.paths.iter() {
            if let ReferenceOr::Item(path_item) = path_item {
                extract_tags_from_path_item(path_item, &mut tag_set);
            }
        }
        tag_set.into_iter().collect()
    }
}

fn extract_tags_from_path_item(path_item: &PathItem, tag_set: &mut std::collections::HashSet<String>) {
    let operations = [
        path_item.get.as_ref(),
        path_item.post.as_ref(),
        path_item.put.as_ref(),
        path_item.delete.as_ref(),
        path_item.patch.as_ref(),
        path_item.head.as_ref(),
        path_item.options.as_ref(),
    ];

    for op in operations.iter().flatten() {
        for tag in &op.tags {
            tag_set.insert(tag.clone());
        }
    }
}

pub fn extract_operations_by_tag(openapi: &OpenAPI) -> HashMap<String, Vec<OperationInfo>> {
    let mut result: HashMap<String, Vec<OperationInfo>> = HashMap::new();

    for (path, path_item) in openapi.paths.iter() {
        if let ReferenceOr::Item(path_item) = path_item {
            add_operation(&mut result, "GET", path, path_item.get.as_ref());
            add_operation(&mut result, "POST", path, path_item.post.as_ref());
            add_operation(&mut result, "PUT", path, path_item.put.as_ref());
            add_operation(&mut result, "DELETE", path, path_item.delete.as_ref());
            add_operation(&mut result, "PATCH", path, path_item.patch.as_ref());
            add_operation(&mut result, "HEAD", path, path_item.head.as_ref());
            add_operation(&mut result, "OPTIONS", path, path_item.options.as_ref());
        }
    }

    result
}

fn add_operation(
    result: &mut HashMap<String, Vec<OperationInfo>>,
    method: &str,
    path: &str,
    operation: Option<&Operation>,
) {
    if let Some(op) = operation {
        let tags = op.tags.clone();
        
        if tags.is_empty() {
            // If no tags, use "default" module
            result
                .entry("default".to_string())
                .or_insert_with(Vec::new)
                .push(OperationInfo {
                    method: method.to_string(),
                    path: path.to_string(),
                    operation: op.clone(),
                });
        } else {
            for tag in tags {
                result
                    .entry(tag.clone())
                    .or_insert_with(Vec::new)
                    .push(OperationInfo {
                        method: method.to_string(),
                        path: path.to_string(),
                        operation: op.clone(),
                    });
            }
        }
    }
}

pub fn extract_schemas(openapi: &OpenAPI) -> HashMap<String, Schema> {
    let mut schemas = HashMap::new();
    
    if let Some(components) = &openapi.components {
        for (name, schema_ref) in components.schemas.iter() {
            if let ReferenceOr::Item(schema) = schema_ref {
                schemas.insert(name.clone(), schema.clone());
            }
        }
    }
    
    schemas
}

pub fn resolve_ref(openapi: &OpenAPI, ref_path: &str) -> Result<ReferenceOr<Schema>> {
    if !ref_path.starts_with("#/") {
        return Err(anyhow::anyhow!("Invalid reference path: {}", ref_path));
    }

    let parts: Vec<&str> = ref_path.trim_start_matches("#/").split('/').collect();
    
    match parts.as_slice() {
        ["components", "schemas", name] => {
            if let Some(components) = &openapi.components {
                if let Some(schema_ref) = components.schemas.get(*name) {
                    return Ok(schema_ref.clone());
                }
            }
            Err(anyhow::anyhow!("Schema not found: {}", name))
        }
        _ => Err(anyhow::anyhow!("Unsupported reference path: {}", ref_path)),
    }
}

pub fn resolve_parameter_ref(openapi: &OpenAPI, ref_path: &str) -> Result<ReferenceOr<Parameter>> {
    if !ref_path.starts_with("#/") {
        return Err(anyhow::anyhow!("Invalid reference path: {}", ref_path));
    }

    let parts: Vec<&str> = ref_path.trim_start_matches("#/").split('/').collect();
    
    match parts.as_slice() {
        ["components", "parameters", name] => {
            if let Some(components) = &openapi.components {
                if let Some(param_ref) = components.parameters.get(*name) {
                    return Ok(param_ref.clone());
                }
            }
            Err(anyhow::anyhow!("Parameter not found: {}", name))
        }
        _ => Err(anyhow::anyhow!("Unsupported reference path: {}", ref_path)),
    }
}

pub fn resolve_request_body_ref(openapi: &OpenAPI, ref_path: &str) -> Result<ReferenceOr<openapiv3::RequestBody>> {
    if !ref_path.starts_with("#/") {
        return Err(anyhow::anyhow!("Invalid reference path: {}", ref_path));
    }

    let parts: Vec<&str> = ref_path.trim_start_matches("#/").split('/').collect();
    
    match parts.as_slice() {
        ["components", "requestBodies", name] => {
            if let Some(components) = &openapi.components {
                if let Some(body_ref) = components.request_bodies.get(*name) {
                    return Ok(body_ref.clone());
                }
            }
            Err(anyhow::anyhow!("Request body not found: {}", name))
        }
        _ => Err(anyhow::anyhow!("Unsupported reference path: {}", ref_path)),
    }
}

pub fn resolve_response_ref(openapi: &OpenAPI, ref_path: &str) -> Result<ReferenceOr<openapiv3::Response>> {
    if !ref_path.starts_with("#/") {
        return Err(anyhow::anyhow!("Invalid reference path: {}", ref_path));
    }

    let parts: Vec<&str> = ref_path.trim_start_matches("#/").split('/').collect();
    
    match parts.as_slice() {
        ["components", "responses", name] => {
            if let Some(components) = &openapi.components {
                if let Some(response_ref) = components.responses.get(*name) {
                    return Ok(response_ref.clone());
                }
            }
            Err(anyhow::anyhow!("Response not found: {}", name))
        }
        _ => Err(anyhow::anyhow!("Unsupported reference path: {}", ref_path)),
    }
}

pub fn get_schema_name_from_ref(ref_path: &str) -> Option<String> {
    if ref_path.starts_with("#/components/schemas/") {
        Some(ref_path.trim_start_matches("#/components/schemas/").to_string())
    } else {
        None
    }
}

pub fn extract_schemas_for_operation(operation: &Operation, openapi: &OpenAPI) -> Result<Vec<String>> {
    let mut schema_names = Vec::new();

    // Extract request body schema
    if let Some(request_body) = &operation.request_body {
        match request_body {
            ReferenceOr::Reference { reference } => {
                if let Some(ref_name) = get_schema_name_from_ref(&reference) {
                    schema_names.push(ref_name);
                }
            }
            ReferenceOr::Item(body) => {
                if let Some(json_media) = body.content.get("application/json") {
                    if let Some(schema_ref) = &json_media.schema {
                        match schema_ref {
                            ReferenceOr::Reference { reference } => {
                                if let Some(ref_name) = get_schema_name_from_ref(&reference) {
                                    schema_names.push(ref_name);
                                }
                            }
                            ReferenceOr::Item(_) => {
                                // Inline schemas: These are schema definitions embedded directly
                                // in the operation. We only track referenced schemas (from #/components/schemas)
                                // to avoid generating duplicate types. Inline schemas are handled
                                // at generation time where they appear.
                            }
                        }
                    }
                }
            }
        }
    }

    // Extract response schemas
    for (_, response_ref) in operation.responses.responses.iter() {
        match response_ref {
            ReferenceOr::Reference { reference } => {
                // Resolve response reference
                if let Ok(ReferenceOr::Item(response)) = resolve_response_ref(openapi, reference) {
                    if let Some(json_media) = response.content.get("application/json") {
                        if let Some(schema_ref) = &json_media.schema {
                            match schema_ref {
                                ReferenceOr::Reference { reference } => {
                                    if let Some(ref_name) = get_schema_name_from_ref(&reference) {
                                        if !schema_names.contains(&ref_name) {
                                            schema_names.push(ref_name);
                                        }
                                    }
                                }
                                ReferenceOr::Item(_) => {
                                    // Inline schema - skip for now
                                }
                            }
                        }
                    }
                }
            }
            ReferenceOr::Item(response) => {
                if let Some(json_media) = response.content.get("application/json") {
                    if let Some(schema_ref) = &json_media.schema {
                        match schema_ref {
                            ReferenceOr::Reference { reference } => {
                                if let Some(ref_name) = get_schema_name_from_ref(&reference) {
                                    if !schema_names.contains(&ref_name) {
                                        schema_names.push(ref_name);
                                    }
                                }
                            }
                            ReferenceOr::Item(_) => {
                                // Inline schemas: These are schema definitions embedded directly
                                // in the operation. We only track referenced schemas (from #/components/schemas)
                                // to avoid generating duplicate types. Inline schemas are handled
                                // at generation time where they appear.
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(schema_names)
}

pub fn map_modules_to_schemas(
    openapi: &OpenAPI,
    operations_by_tag: &HashMap<String, Vec<OperationInfo>>,
    schemas: &HashMap<String, Schema>,
) -> Result<HashMap<String, Vec<String>>> {
    let mut module_schemas: HashMap<String, Vec<String>> = HashMap::new();

    for (module, operations) in operations_by_tag {
        let mut module_schema_set = std::collections::HashSet::new();

        for op_info in operations {
            let op_schemas = extract_schemas_for_operation(&op_info.operation, openapi)?;
            for schema_name in op_schemas {
                if schemas.contains_key(&schema_name) {
                    module_schema_set.insert(schema_name);
                }
            }
        }

        module_schemas.insert(module.clone(), module_schema_set.into_iter().collect());
    }

    Ok(module_schemas)
}

