use crate::error::{FileSystemError, NetworkError, Result, SchemaError};
use openapiv3::{OpenAPI, Operation, Parameter, PathItem, ReferenceOr, Schema};
use std::collections::HashMap;

pub struct ParsedSpec {
    pub openapi: OpenAPI,
    pub modules: Vec<String>,
    pub operations_by_tag: HashMap<String, Vec<OperationInfo>>,
    pub schemas: HashMap<String, Schema>,
    pub module_schemas: HashMap<String, Vec<String>>,
    pub common_schemas: Vec<String>, // Schemas shared across multiple modules
}

#[derive(Debug, Clone)]
pub struct OperationInfo {
    pub method: String,
    pub path: String,
    pub operation: Operation,
}

pub async fn fetch_and_parse_spec(spec_path: &str) -> Result<ParsedSpec> {
    fetch_and_parse_spec_with_cache(spec_path, false).await
}

pub async fn fetch_and_parse_spec_with_cache(
    spec_path: &str,
    use_cache: bool,
) -> Result<ParsedSpec> {
    let content = if spec_path.starts_with("http://") || spec_path.starts_with("https://") {
        // Try cache first if enabled
        if use_cache {
            if let Some(cached) = crate::cache::CacheManager::get_cached_spec(spec_path)? {
                return parse_spec_content(&cached, spec_path);
            }
        }

        let content = fetch_remote_spec(spec_path).await?;

        // Cache the content
        if use_cache {
            crate::cache::CacheManager::cache_spec(spec_path, &content)?;
        }

        content
    } else {
        // Check if file exists before trying to read
        if !std::path::Path::new(spec_path).exists() {
            return Err(FileSystemError::FileNotFound {
                path: spec_path.to_string(),
            }
            .into());
        }
        std::fs::read_to_string(spec_path).map_err(|e| FileSystemError::ReadFileFailed {
            path: spec_path.to_string(),
            source: e,
        })?
    };

    parse_spec_content(&content, spec_path)
}

fn parse_spec_content(content: &str, spec_path: &str) -> Result<ParsedSpec> {
    let openapi: OpenAPI = if spec_path.ends_with(".yaml") || spec_path.ends_with(".yml") {
        serde_yaml::from_str(content).map_err(|e| SchemaError::UnsupportedType {
            schema_type: format!("Failed to parse YAML spec: {}", e),
        })?
    } else {
        serde_json::from_str(content).map_err(|e| SchemaError::UnsupportedType {
            schema_type: format!("Failed to parse JSON spec: {}", e),
        })?
    };

    let modules = extract_modules(&openapi);
    let operations_by_tag = extract_operations_by_tag(&openapi);
    let schemas = extract_schemas(&openapi);
    let (module_schemas, _) = map_modules_to_schemas(&openapi, &operations_by_tag, &schemas)?;

    Ok(ParsedSpec {
        openapi,
        modules,
        operations_by_tag,
        schemas,
        module_schemas,
        common_schemas: Vec::new(), // Will be filtered based on selected modules
    })
}

async fn fetch_remote_spec(url: &str) -> Result<String> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| NetworkError::FetchFailed {
            url: url.to_string(),
            source: e,
        })?;

    response.text().await.map_err(|e| {
        NetworkError::ReadResponseFailed {
            url: url.to_string(),
            source: e,
        }
        .into()
    })
}

pub fn extract_modules(openapi: &OpenAPI) -> Vec<String> {
    if !openapi.tags.is_empty() {
        openapi.tags.iter().map(|tag| tag.name.clone()).collect()
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

fn extract_tags_from_path_item(
    path_item: &PathItem,
    tag_set: &mut std::collections::HashSet<String>,
) {
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
                .or_default()
                .push(OperationInfo {
                    method: method.to_string(),
                    path: path.to_string(),
                    operation: op.clone(),
                });
        } else {
            for tag in tags {
                result.entry(tag.clone()).or_default().push(OperationInfo {
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
        return Err(SchemaError::InvalidReference {
            ref_path: ref_path.to_string(),
        }
        .into());
    }

    let parts: Vec<&str> = ref_path.trim_start_matches("#/").split('/').collect();

    match parts.as_slice() {
        ["components", "schemas", name] => {
            if let Some(components) = &openapi.components {
                if let Some(schema_ref) = components.schemas.get(*name) {
                    return Ok(schema_ref.clone());
                }
            }
            Err(SchemaError::NotFound {
                name: name.to_string(),
            }
            .into())
        }
        _ => Err(SchemaError::UnsupportedReferencePath {
            ref_path: ref_path.to_string(),
        }
        .into()),
    }
}

pub fn resolve_parameter_ref(openapi: &OpenAPI, ref_path: &str) -> Result<ReferenceOr<Parameter>> {
    if !ref_path.starts_with("#/") {
        return Err(SchemaError::InvalidReference {
            ref_path: ref_path.to_string(),
        }
        .into());
    }

    let parts: Vec<&str> = ref_path.trim_start_matches("#/").split('/').collect();

    match parts.as_slice() {
        ["components", "parameters", name] => {
            if let Some(components) = &openapi.components {
                if let Some(param_ref) = components.parameters.get(*name) {
                    return Ok(param_ref.clone());
                }
            }
            Err(SchemaError::ParameterNotFound {
                name: name.to_string(),
            }
            .into())
        }
        _ => Err(SchemaError::UnsupportedReferencePath {
            ref_path: ref_path.to_string(),
        }
        .into()),
    }
}

pub fn resolve_request_body_ref(
    openapi: &OpenAPI,
    ref_path: &str,
) -> Result<ReferenceOr<openapiv3::RequestBody>> {
    if !ref_path.starts_with("#/") {
        return Err(SchemaError::InvalidReference {
            ref_path: ref_path.to_string(),
        }
        .into());
    }

    let parts: Vec<&str> = ref_path.trim_start_matches("#/").split('/').collect();

    match parts.as_slice() {
        ["components", "requestBodies", name] => {
            if let Some(components) = &openapi.components {
                if let Some(body_ref) = components.request_bodies.get(*name) {
                    return Ok(body_ref.clone());
                }
            }
            Err(SchemaError::RequestBodyNotFound {
                name: name.to_string(),
            }
            .into())
        }
        _ => Err(SchemaError::UnsupportedReferencePath {
            ref_path: ref_path.to_string(),
        }
        .into()),
    }
}

pub fn resolve_response_ref(
    openapi: &OpenAPI,
    ref_path: &str,
) -> Result<ReferenceOr<openapiv3::Response>> {
    if !ref_path.starts_with("#/") {
        return Err(SchemaError::InvalidReference {
            ref_path: ref_path.to_string(),
        }
        .into());
    }

    let parts: Vec<&str> = ref_path.trim_start_matches("#/").split('/').collect();

    match parts.as_slice() {
        ["components", "responses", name] => {
            if let Some(components) = &openapi.components {
                if let Some(response_ref) = components.responses.get(*name) {
                    return Ok(response_ref.clone());
                }
            }
            Err(SchemaError::ResponseNotFound {
                name: name.to_string(),
            }
            .into())
        }
        _ => Err(SchemaError::UnsupportedReferencePath {
            ref_path: ref_path.to_string(),
        }
        .into()),
    }
}

pub fn get_schema_name_from_ref(ref_path: &str) -> Option<String> {
    if ref_path.starts_with("#/components/schemas/") {
        Some(
            ref_path
                .trim_start_matches("#/components/schemas/")
                .to_string(),
        )
    } else {
        None
    }
}

pub fn extract_schemas_for_operation(
    operation: &Operation,
    openapi: &OpenAPI,
) -> Result<Vec<String>> {
    let mut schema_names = Vec::new();

    // Extract request body schema
    if let Some(request_body) = &operation.request_body {
        match request_body {
            ReferenceOr::Reference { reference } => {
                if let Some(ref_name) = get_schema_name_from_ref(reference) {
                    schema_names.push(ref_name);
                }
            }
            ReferenceOr::Item(body) => {
                if let Some(json_media) = body.content.get("application/json") {
                    if let Some(schema_ref) = &json_media.schema {
                        match schema_ref {
                            ReferenceOr::Reference { reference } => {
                                if let Some(ref_name) = get_schema_name_from_ref(reference) {
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
                                    if let Some(ref_name) = get_schema_name_from_ref(reference) {
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
                                if let Some(ref_name) = get_schema_name_from_ref(reference) {
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

/// Recursively collect all schema dependencies for a given set of schema names
pub fn collect_all_dependencies(
    schema_names: &[String],
    schemas: &HashMap<String, Schema>,
    openapi: &OpenAPI,
) -> Result<Vec<String>> {
    let mut all_schemas = std::collections::HashSet::new();
    let mut to_process: Vec<String> = schema_names.to_vec();
    let mut processed = std::collections::HashSet::new();

    while let Some(schema_name) = to_process.pop() {
        if processed.contains(&schema_name) {
            continue;
        }
        processed.insert(schema_name.clone());
        all_schemas.insert(schema_name.clone());

        // Get dependencies of this schema
        if let Some(schema) = schemas.get(&schema_name) {
            let deps = extract_schema_dependencies(schema, openapi)?;
            for dep in deps {
                if schemas.contains_key(&dep) && !processed.contains(&dep) {
                    to_process.push(dep);
                }
            }
        }
    }

    Ok(all_schemas.into_iter().collect())
}

/// Extract all schema references from a schema
fn extract_schema_dependencies(schema: &Schema, openapi: &OpenAPI) -> Result<Vec<String>> {
    let mut deps = Vec::new();
    let mut visited = std::collections::HashSet::new();
    extract_schema_refs_recursive(schema, openapi, &mut deps, &mut visited)?;
    Ok(deps)
}

fn extract_schema_refs_recursive(
    schema: &Schema,
    openapi: &OpenAPI,
    deps: &mut Vec<String>,
    visited: &mut std::collections::HashSet<String>,
) -> Result<()> {
    match &schema.schema_kind {
        openapiv3::SchemaKind::Type(type_) => match type_ {
            openapiv3::Type::Array(array) => {
                if let Some(items) = &array.items {
                    match items {
                        ReferenceOr::Reference { reference } => {
                            if let Some(ref_name) = get_schema_name_from_ref(reference) {
                                if !visited.contains(&ref_name) {
                                    visited.insert(ref_name.clone());
                                    deps.push(ref_name.clone());
                                    if let Ok(ReferenceOr::Item(dep_schema)) =
                                        resolve_ref(openapi, reference)
                                    {
                                        extract_schema_refs_recursive(
                                            &dep_schema,
                                            openapi,
                                            deps,
                                            visited,
                                        )?;
                                    }
                                }
                            }
                        }
                        ReferenceOr::Item(item_schema) => {
                            extract_schema_refs_recursive(item_schema, openapi, deps, visited)?;
                        }
                    }
                }
            }
            openapiv3::Type::Object(object_type) => {
                for (_, prop_schema_ref) in object_type.properties.iter() {
                    match prop_schema_ref {
                        ReferenceOr::Reference { reference } => {
                            if let Some(ref_name) = get_schema_name_from_ref(reference) {
                                if !visited.contains(&ref_name) {
                                    visited.insert(ref_name.clone());
                                    deps.push(ref_name.clone());
                                    if let Ok(ReferenceOr::Item(dep_schema)) =
                                        resolve_ref(openapi, reference)
                                    {
                                        extract_schema_refs_recursive(
                                            &dep_schema,
                                            openapi,
                                            deps,
                                            visited,
                                        )?;
                                    }
                                }
                            }
                        }
                        ReferenceOr::Item(prop_schema) => {
                            extract_schema_refs_recursive(prop_schema, openapi, deps, visited)?;
                        }
                    }
                }
            }
            _ => {}
        },
        openapiv3::SchemaKind::OneOf { one_of, .. } => {
            for item in one_of {
                match item {
                    ReferenceOr::Reference { reference } => {
                        if let Some(ref_name) = get_schema_name_from_ref(reference) {
                            if !visited.contains(&ref_name) {
                                visited.insert(ref_name.clone());
                                deps.push(ref_name.clone());
                                if let Ok(ReferenceOr::Item(dep_schema)) =
                                    resolve_ref(openapi, reference)
                                {
                                    extract_schema_refs_recursive(
                                        &dep_schema,
                                        openapi,
                                        deps,
                                        visited,
                                    )?;
                                }
                            }
                        }
                    }
                    ReferenceOr::Item(item_schema) => {
                        extract_schema_refs_recursive(item_schema, openapi, deps, visited)?;
                    }
                }
            }
        }
        openapiv3::SchemaKind::AllOf { all_of, .. } => {
            for item in all_of {
                match item {
                    ReferenceOr::Reference { reference } => {
                        if let Some(ref_name) = get_schema_name_from_ref(reference) {
                            if !visited.contains(&ref_name) {
                                visited.insert(ref_name.clone());
                                deps.push(ref_name.clone());
                                if let Ok(ReferenceOr::Item(dep_schema)) =
                                    resolve_ref(openapi, reference)
                                {
                                    extract_schema_refs_recursive(
                                        &dep_schema,
                                        openapi,
                                        deps,
                                        visited,
                                    )?;
                                }
                            }
                        }
                    }
                    ReferenceOr::Item(item_schema) => {
                        extract_schema_refs_recursive(item_schema, openapi, deps, visited)?;
                    }
                }
            }
        }
        openapiv3::SchemaKind::AnyOf { any_of, .. } => {
            for item in any_of {
                match item {
                    ReferenceOr::Reference { reference } => {
                        if let Some(ref_name) = get_schema_name_from_ref(reference) {
                            if !visited.contains(&ref_name) {
                                visited.insert(ref_name.clone());
                                deps.push(ref_name.clone());
                                if let Ok(ReferenceOr::Item(dep_schema)) =
                                    resolve_ref(openapi, reference)
                                {
                                    extract_schema_refs_recursive(
                                        &dep_schema,
                                        openapi,
                                        deps,
                                        visited,
                                    )?;
                                }
                            }
                        }
                    }
                    ReferenceOr::Item(item_schema) => {
                        extract_schema_refs_recursive(item_schema, openapi, deps, visited)?;
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

#[allow(clippy::type_complexity)]
pub fn map_modules_to_schemas(
    openapi: &OpenAPI,
    operations_by_tag: &HashMap<String, Vec<OperationInfo>>,
    schemas: &HashMap<String, Schema>,
) -> Result<(HashMap<String, Vec<String>>, Vec<String>)> {
    let mut module_schemas: HashMap<String, Vec<String>> = HashMap::new();
    let mut schema_usage: HashMap<String, Vec<String>> = HashMap::new(); // Track which modules use each schema

    // First pass: collect all schemas per module
    for (module, operations) in operations_by_tag {
        let mut module_schema_set = std::collections::HashSet::new();

        for op_info in operations {
            let op_schemas = extract_schemas_for_operation(&op_info.operation, openapi)?;
            for schema_name in op_schemas {
                if schemas.contains_key(&schema_name) {
                    module_schema_set.insert(schema_name.clone());
                    // Track schema usage
                    schema_usage
                        .entry(schema_name.clone())
                        .or_default()
                        .push(module.clone());
                }
            }
        }

        // Collect all dependencies for the schemas used by this module
        let initial_schemas: Vec<String> = module_schema_set.into_iter().collect();
        let all_dependencies = collect_all_dependencies(&initial_schemas, schemas, openapi)?;

        // Track dependencies usage too
        for dep in &all_dependencies {
            schema_usage
                .entry(dep.clone())
                .or_default()
                .push(module.clone());
        }

        module_schemas.insert(module.clone(), all_dependencies);
    }

    // Return without filtering - filtering will be done based on selected modules
    Ok((module_schemas, Vec::new()))
}

/// Filter common schemas based on selected modules only
/// Only creates common schemas when 2+ modules are selected
pub fn filter_common_schemas(
    module_schemas: &HashMap<String, Vec<String>>,
    selected_modules: &[String],
) -> (HashMap<String, Vec<String>>, Vec<String>) {
    let mut filtered_module_schemas = module_schemas.clone();

    // Only create common module if 2+ modules are selected
    if selected_modules.len() < 2 {
        return (filtered_module_schemas, Vec::new());
    }

    let mut schema_usage: HashMap<String, Vec<String>> = HashMap::new();

    // Track schema usage only for selected modules
    for module in selected_modules {
        if let Some(schemas) = filtered_module_schemas.get(module) {
            for schema_name in schemas {
                schema_usage
                    .entry(schema_name.clone())
                    .or_default()
                    .push(module.clone());
            }
        }
    }

    // Identify shared schemas (used by 2+ selected modules)
    // Only include schemas that are used by ALL selected modules or at least 2 of them
    let mut shared_schemas = std::collections::HashSet::new();
    for (schema_name, modules_using_it) in &schema_usage {
        // Count unique modules using this schema
        let unique_modules: std::collections::HashSet<String> =
            modules_using_it.iter().cloned().collect();
        if unique_modules.len() >= 2 {
            shared_schemas.insert(schema_name.clone());
        }
    }

    // Remove shared schemas from individual selected modules
    let common_schemas: Vec<String> = shared_schemas.iter().cloned().collect();
    if !shared_schemas.is_empty() {
        for module in selected_modules {
            if let Some(module_schema_list) = filtered_module_schemas.get_mut(module) {
                module_schema_list.retain(|s| !shared_schemas.contains(s));
            }
        }
    }

    (filtered_module_schemas, common_schemas)
}
