use crate::error::Result;
use crate::generator::swagger_parser::{get_schema_name_from_ref, resolve_ref};
use crate::generator::utils::to_pascal_case;
use openapiv3::{OpenAPI, ReferenceOr, Schema, SchemaKind, Type};
use std::collections::HashMap;

pub struct ZodSchema {
    pub content: String,
}

pub fn generate_zod_schemas(
    openapi: &OpenAPI,
    schemas: &HashMap<String, Schema>,
    schema_names: &[String],
) -> Result<Vec<ZodSchema>> {
    generate_zod_schemas_with_registry(
        openapi,
        schemas,
        schema_names,
        &mut std::collections::HashMap::new(),
    )
}

pub fn generate_zod_schemas_with_registry(
    openapi: &OpenAPI,
    schemas: &HashMap<String, Schema>,
    schema_names: &[String],
    enum_registry: &mut std::collections::HashMap<String, String>,
) -> Result<Vec<ZodSchema>> {
    let mut zod_schemas = Vec::new();
    let mut processed = std::collections::HashSet::new();

    for schema_name in schema_names {
        if let Some(schema) = schemas.get(schema_name) {
            generate_zod_for_schema(
                openapi,
                schema_name,
                schema,
                &mut zod_schemas,
                &mut processed,
                enum_registry,
                None,
            )?;
        }
    }

    Ok(zod_schemas)
}

fn generate_zod_for_schema(
    openapi: &OpenAPI,
    name: &str,
    schema: &Schema,
    zod_schemas: &mut Vec<ZodSchema>,
    processed: &mut std::collections::HashSet<String>,
    enum_registry: &mut std::collections::HashMap<String, String>,
    parent_schema_name: Option<&str>,
) -> Result<()> {
    if processed.contains(name) {
        return Ok(());
    }
    processed.insert(name.to_string());

    let schema_name = to_pascal_case(name);
    let zod_def = schema_to_zod(
        openapi,
        schema,
        zod_schemas,
        processed,
        0,
        enum_registry,
        None,
        parent_schema_name,
    )?;

    // Handle enums at top level (when schema itself is an enum)
    // Note: For property-level enums, they're handled in schema_to_zod with context
    if let SchemaKind::Type(Type::String(string_type)) = &schema.schema_kind {
        if !string_type.enumeration.is_empty() {
            let mut enum_values: Vec<String> = string_type
                .enumeration
                .iter()
                .filter_map(|v| v.as_ref().cloned())
                .collect();
            if !enum_values.is_empty() {
                enum_values.sort();
                let enum_key = enum_values.join(",");

                // Check if this enum already exists in registry
                if enum_registry.get(&enum_key).is_some() {
                    // Enum already generated, skip
                    return Ok(());
                }

                // Use schema name if available, otherwise generate from values
                let enum_name = if !name.is_empty() {
                    format!("{}Enum", to_pascal_case(name))
                } else {
                    // Generate unique name from values
                    let value_hash: String = enum_values
                        .iter()
                        .take(3)
                        .map(|v| v.chars().next().unwrap_or('X'))
                        .collect();
                    format!("Enum{}", value_hash)
                };

                // Store in registry
                enum_registry.insert(enum_key, enum_name.clone());

                let enum_schema = generate_enum_zod(&enum_name, &enum_values);
                zod_schemas.push(enum_schema);
                return Ok(());
            }
        }
    }

    // Only create object schema if it's an object type
    if matches!(&schema.schema_kind, SchemaKind::Type(Type::Object(_))) {
        zod_schemas.push(ZodSchema {
            content: format!(
                "export const {}Schema = z.object({{\n{}\n}});",
                schema_name, zod_def
            ),
        });
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn schema_to_zod(
    openapi: &OpenAPI,
    schema: &Schema,
    zod_schemas: &mut Vec<ZodSchema>,
    processed: &mut std::collections::HashSet<String>,
    indent: usize,
    enum_registry: &mut std::collections::HashMap<String, String>,
    context: Option<(&str, &str)>, // (property_name, parent_schema_name)
    current_schema_name: Option<&str>, // Current schema being processed (for enum naming context)
) -> Result<String> {
    // Prevent infinite recursion with a reasonable depth limit
    if indent > 100 {
        return Ok(format!("{}z.any()", "  ".repeat(indent)));
    }
    let indent_str = "  ".repeat(indent);

    match &schema.schema_kind {
        SchemaKind::Type(type_) => {
            match type_ {
                Type::String(string_type) => {
                    if !string_type.enumeration.is_empty() {
                        let mut enum_values: Vec<String> = string_type
                            .enumeration
                            .iter()
                            .filter_map(|v| v.as_ref().cloned())
                            .collect();
                        if !enum_values.is_empty() {
                            // Create a key from sorted enum values to check registry
                            enum_values.sort();
                            let enum_key = enum_values.join(",");

                            // For generic property names, include context in the key to avoid conflicts
                            let context_key = if let Some((prop_name, parent_schema)) = context {
                                let generic_names = ["status", "type", "state", "kind"];
                                if generic_names.contains(&prop_name.to_lowercase().as_str())
                                    && !parent_schema.is_empty()
                                {
                                    // Include parent schema in key for generic properties to avoid conflicts
                                    format!("{}:{}", enum_key, parent_schema)
                                } else {
                                    enum_key.clone()
                                }
                            } else {
                                enum_key.clone()
                            };

                            // Check if this enum already exists in registry
                            // First check context_key (for context-aware enums)
                            // Then check base enum_key to deduplicate enums with same values
                            let existing_enum_name = enum_registry
                                .get(&context_key)
                                .or_else(|| enum_registry.get(&enum_key))
                                .cloned();

                            if let Some(existing_name) = existing_enum_name {
                                // Store in registry with context_key for future lookups
                                enum_registry.insert(context_key.clone(), existing_name.clone());

                                // Check if enum schema has already been generated
                                let schema_already_generated = zod_schemas.iter().any(|s| {
                                    s.content.contains(&format!("{}Schema", existing_name))
                                });

                                // If not generated yet, generate it now
                                if !schema_already_generated {
                                    let enum_schema =
                                        generate_enum_zod(&existing_name, &enum_values);
                                    zod_schemas.push(enum_schema);
                                }

                                // Enums don't need z.lazy(), use directly
                                return Ok(format!("{}{}Schema", indent_str, existing_name));
                            }

                            // Generate meaningful enum name using context (property name + parent schema) or fallback
                            let enum_name = if let Some((prop_name, parent_schema)) = context {
                                // Use property name + parent schema for meaningful name to avoid conflicts
                                // For generic names like "status", use parent schema to differentiate
                                let prop_pascal = to_pascal_case(prop_name);

                                // If property name is generic (status, type, etc.), use parent schema
                                let generic_names = ["status", "type", "state", "kind"];
                                if generic_names.contains(&prop_name.to_lowercase().as_str())
                                    && !parent_schema.is_empty()
                                {
                                    let parent_pascal = to_pascal_case(parent_schema);
                                    // Remove common suffixes from parent schema name
                                    let parent_clean = parent_pascal
                                        .trim_end_matches("ResponseDto")
                                        .trim_end_matches("Dto")
                                        .trim_end_matches("Response")
                                        .to_string();

                                    // Check if parent already contains the property name (e.g., "KycStatus" contains "Status")
                                    // Use case-insensitive matching and check if property name is a suffix or contained
                                    let prop_lower = prop_pascal.to_lowercase();
                                    let parent_lower = parent_clean.to_lowercase();

                                    // Check if parent ends with property name (e.g., "KycStatus" ends with "Status")
                                    // or if property is contained in parent (case-insensitive)
                                    if parent_lower.ends_with(&prop_lower)
                                        || parent_lower.contains(&prop_lower)
                                    {
                                        // Parent already contains property name, just use parent + Enum
                                        format!("{}Enum", parent_clean)
                                    } else {
                                        // Combine parent + property
                                        format!("{}{}Enum", parent_clean, prop_pascal)
                                    }
                                } else {
                                    format!("{}Enum", prop_pascal)
                                }
                            } else if !enum_values.is_empty() {
                                // Fallback: use first value to create name
                                let first_value = &enum_values[0];
                                let base_name = first_value
                                    .chars()
                                    .take(1)
                                    .collect::<String>()
                                    .to_uppercase()
                                    + &first_value.chars().skip(1).collect::<String>();
                                format!("{}Enum", to_pascal_case(&base_name))
                            } else {
                                "UnknownEnum".to_string()
                            };

                            // Store in registry using context_key (includes context for generic properties)
                            // Also store with base enum_key for deduplication
                            enum_registry.insert(context_key.clone(), enum_name.clone());
                            enum_registry.insert(enum_key.clone(), enum_name.clone());

                            // Generate enum schema
                            let enum_schema = generate_enum_zod(&enum_name, &enum_values);
                            zod_schemas.push(enum_schema);

                            // Enums don't need z.lazy(), use directly
                            return Ok(format!("{}{}Schema", indent_str, enum_name));
                        }
                    }
                    let mut zod_expr = format!("{}z.string()", indent_str);

                    // Add string constraints
                    if let Some(min_length) = string_type.min_length {
                        zod_expr = format!("{}.min({})", zod_expr, min_length);
                    }
                    if let Some(max_length) = string_type.max_length {
                        zod_expr = format!("{}.max({})", zod_expr, max_length);
                    }
                    if let Some(pattern) = &string_type.pattern {
                        // Escape regex pattern for JavaScript
                        let escaped_pattern = pattern.replace('\\', "\\\\").replace('"', "\\\"");
                        zod_expr = format!("{}.regex(/^{}$/)", zod_expr, escaped_pattern);
                    }
                    // Handle format - it's a VariantOrUnknownOrEmpty
                    // Try to extract format as string from both Item and Unknown variants
                    let format_str_opt = match &string_type.format {
                        openapiv3::VariantOrUnknownOrEmpty::Item(format) => {
                            // Convert enum variant to string using Debug
                            let debug_str = format!("{:?}", format);
                            // Extract format name (e.g., "Email" from "Email" or "StringFormat::Email")
                            let format_name = debug_str
                                .split("::")
                                .last()
                                .unwrap_or(&debug_str)
                                .to_lowercase();
                            Some(format_name)
                        }
                        openapiv3::VariantOrUnknownOrEmpty::Unknown(s) => Some(s.clone()),
                        _ => None,
                    };

                    if let Some(format_str) = format_str_opt {
                        match format_str.as_str() {
                            "email" => zod_expr = format!("{}.email()", zod_expr),
                            "uri" | "url" => zod_expr = format!("{}.url()", zod_expr),
                            "uuid" => zod_expr = format!("{}.uuid()", zod_expr),
                            "date-time" | "datetime" | "date_time" => {
                                zod_expr = format!("{}.datetime()", zod_expr)
                            }
                            _ => {}
                        }
                    }

                    Ok(zod_expr)
                }
                Type::Number(number_type) => {
                    let mut zod_expr = format!("{}z.number()", indent_str);

                    // Add number constraints
                    if let Some(minimum) = number_type.minimum {
                        zod_expr = format!("{}.min({})", zod_expr, minimum);
                    }
                    if let Some(maximum) = number_type.maximum {
                        zod_expr = format!("{}.max({})", zod_expr, maximum);
                    }
                    if let Some(multiple_of) = number_type.multiple_of {
                        zod_expr = format!("{}.multipleOf({})", zod_expr, multiple_of);
                    }

                    Ok(zod_expr)
                }
                Type::Integer(integer_type) => {
                    let mut zod_expr = format!("{}z.number().int()", indent_str);

                    // Add integer constraints
                    if let Some(minimum) = integer_type.minimum {
                        zod_expr = format!("{}.min({})", zod_expr, minimum);
                    }
                    if let Some(maximum) = integer_type.maximum {
                        zod_expr = format!("{}.max({})", zod_expr, maximum);
                    }
                    if let Some(multiple_of) = integer_type.multiple_of {
                        zod_expr = format!("{}.multipleOf({})", zod_expr, multiple_of);
                    }

                    Ok(zod_expr)
                }
                Type::Boolean(_) => Ok(format!("{}z.boolean()", indent_str)),
                Type::Array(array_type) => {
                    let item_zod = if let Some(items) = &array_type.items {
                        match items {
                            ReferenceOr::Reference { reference } => {
                                if let Some(ref_name) = get_schema_name_from_ref(reference) {
                                    // If already processed, use lazy reference to avoid infinite recursion
                                    if processed.contains(&ref_name) {
                                        format!(
                                            "{}z.lazy(() => {}Schema)",
                                            indent_str,
                                            to_pascal_case(&ref_name)
                                        )
                                    } else {
                                        let resolved =
                                            resolve_ref(openapi, reference).map_err(|e| {
                                                crate::error::SchemaError::InvalidReference {
                                                    ref_path: format!("Failed to resolve: {}", e),
                                                }
                                            })?;
                                        if let ReferenceOr::Item(item_schema) = resolved {
                                            // Check if it's an object that needs to be extracted
                                            if matches!(
                                                &item_schema.schema_kind,
                                                SchemaKind::Type(Type::Object(_))
                                            ) {
                                                generate_zod_for_schema(
                                                    openapi,
                                                    &ref_name,
                                                    &item_schema,
                                                    zod_schemas,
                                                    processed,
                                                    enum_registry,
                                                    None,
                                                )?;
                                                format!(
                                                    "{}z.lazy(() => {}Schema)",
                                                    indent_str,
                                                    to_pascal_case(&ref_name)
                                                )
                                            } else {
                                                schema_to_zod(
                                                    openapi,
                                                    &item_schema,
                                                    zod_schemas,
                                                    processed,
                                                    indent,
                                                    enum_registry,
                                                    None,
                                                    current_schema_name,
                                                )?
                                            }
                                        } else {
                                            format!(
                                                "{}z.lazy(() => {}Schema)",
                                                indent_str,
                                                to_pascal_case(&ref_name)
                                            )
                                        }
                                    }
                                } else {
                                    format!("{}z.any()", indent_str)
                                }
                            }
                            ReferenceOr::Item(item_schema) => {
                                // If it's an object, we need to generate it inline or as a separate schema
                                if matches!(
                                    &item_schema.schema_kind,
                                    SchemaKind::Type(Type::Object(_))
                                ) {
                                    // Generate object fields
                                    let object_fields = schema_to_zod(
                                        openapi,
                                        item_schema,
                                        zod_schemas,
                                        processed,
                                        indent + 1,
                                        enum_registry,
                                        None,
                                        current_schema_name,
                                    )?;
                                    // Wrap in z.object()
                                    format!(
                                        "{}z.object({{\n{}\n{}}})",
                                        indent_str, object_fields, indent_str
                                    )
                                } else {
                                    schema_to_zod(
                                        openapi,
                                        item_schema,
                                        zod_schemas,
                                        processed,
                                        indent,
                                        enum_registry,
                                        None,
                                        current_schema_name,
                                    )?
                                }
                            }
                        }
                    } else {
                        format!("{}z.any()", indent_str)
                    };

                    let mut array_zod = format!("{}z.array({})", indent_str, item_zod.trim_start());

                    // Add array constraints
                    if let Some(min_items) = array_type.min_items {
                        array_zod = format!("{}.min({})", array_zod, min_items);
                    }
                    if let Some(max_items) = array_type.max_items {
                        array_zod = format!("{}.max({})", array_zod, max_items);
                    }

                    Ok(array_zod)
                }
                Type::Object(object_type) => {
                    if !object_type.properties.is_empty() {
                        let mut fields = Vec::new();
                        // Get parent schema name from context if available, otherwise use current_schema_name parameter
                        // For object properties, use the current schema name as parent
                        let parent_schema_for_props = if let Some((_, parent)) = context {
                            if !parent.is_empty() {
                                parent.to_string()
                            } else if let Some(current) = current_schema_name {
                                current.to_string()
                            } else {
                                String::new()
                            }
                        } else if let Some(current) = current_schema_name {
                            current.to_string()
                        } else {
                            String::new()
                        };

                        for (prop_name, prop_schema_ref) in object_type.properties.iter() {
                            let prop_zod = match prop_schema_ref {
                                ReferenceOr::Reference { reference } => {
                                    // For $ref properties, always use the schema name (don't inline)
                                    if let Some(ref_name) = get_schema_name_from_ref(reference) {
                                        // Generate the referenced schema if not already processed
                                        if !processed.contains(&ref_name) {
                                            if let Ok(ReferenceOr::Item(ref_schema)) =
                                                resolve_ref(openapi, reference)
                                            {
                                                if matches!(
                                                    &ref_schema.schema_kind,
                                                    SchemaKind::Type(Type::Object(_))
                                                ) {
                                                    generate_zod_for_schema(
                                                        openapi,
                                                        &ref_name,
                                                        &ref_schema,
                                                        zod_schemas,
                                                        processed,
                                                        enum_registry,
                                                        Some(&parent_schema_for_props),
                                                    )?;
                                                }
                                            }
                                        }
                                        format!(
                                            "{}z.lazy(() => {}Schema)",
                                            indent_str,
                                            to_pascal_case(&ref_name)
                                        )
                                    } else {
                                        format!("{}z.any()", indent_str)
                                    }
                                }
                                ReferenceOr::Item(prop_schema) => schema_to_zod(
                                    openapi,
                                    prop_schema,
                                    zod_schemas,
                                    processed,
                                    indent + 1,
                                    enum_registry,
                                    Some((prop_name, &parent_schema_for_props)),
                                    current_schema_name,
                                )?,
                            };

                            let required = object_type.required.contains(prop_name);

                            let nullable = prop_schema_ref
                                .as_item()
                                .map(|s| s.schema_data.nullable)
                                .unwrap_or(false);

                            let mut zod_expr = prop_zod.trim_start().to_string();
                            if nullable {
                                zod_expr = format!("{}.nullable()", zod_expr);
                            }
                            if !required {
                                zod_expr = format!("{}.optional()", zod_expr);
                            }

                            fields.push(format!(
                                "{}{}: {},",
                                "  ".repeat(indent + 1),
                                prop_name,
                                zod_expr
                            ));
                        }
                        Ok(fields.join("\n"))
                    } else {
                        Ok(format!("{}z.record(z.string(), z.any())", indent_str))
                    }
                }
            }
        }
        SchemaKind::Any(_) => Ok(format!("{}z.any()", indent_str)),
        SchemaKind::OneOf { one_of, .. } => {
            let mut variant_schemas = Vec::new();
            for item in one_of {
                match item {
                    ReferenceOr::Reference { reference } => {
                        if let Some(ref_name) = get_schema_name_from_ref(reference) {
                            variant_schemas.push(format!(
                                "{}z.lazy(() => {}Schema)",
                                indent_str,
                                crate::generator::utils::to_pascal_case(&ref_name)
                            ));
                        } else {
                            variant_schemas.push(format!("{}z.any()", indent_str));
                        }
                    }
                    ReferenceOr::Item(item_schema) => {
                        let item_zod = schema_to_zod(
                            openapi,
                            item_schema,
                            zod_schemas,
                            processed,
                            indent,
                            enum_registry,
                            None,
                            current_schema_name,
                        )?;
                        variant_schemas.push(item_zod);
                    }
                }
            }
            if variant_schemas.is_empty() {
                Ok(format!("{}z.any()", indent_str))
            } else {
                Ok(format!(
                    "{}z.union([{}])",
                    indent_str,
                    variant_schemas.join(", ")
                ))
            }
        }
        SchemaKind::AllOf { all_of, .. } => {
            let mut all_schemas = Vec::new();
            for item in all_of {
                match item {
                    ReferenceOr::Reference { reference } => {
                        if let Some(ref_name) = get_schema_name_from_ref(reference) {
                            all_schemas.push(format!(
                                "{}z.lazy(() => {}Schema)",
                                indent_str,
                                crate::generator::utils::to_pascal_case(&ref_name)
                            ));
                        } else {
                            all_schemas.push(format!("{}z.any()", indent_str));
                        }
                    }
                    ReferenceOr::Item(item_schema) => {
                        let item_zod = schema_to_zod(
                            openapi,
                            item_schema,
                            zod_schemas,
                            processed,
                            indent,
                            enum_registry,
                            None,
                            current_schema_name,
                        )?;
                        all_schemas.push(item_zod);
                    }
                }
            }
            if all_schemas.is_empty() {
                Ok(format!("{}z.any()", indent_str))
            } else if all_schemas.len() == 1 {
                Ok(all_schemas[0].clone())
            } else {
                // AllOf represents intersection: all schemas must be satisfied
                // Zod uses .and() for intersection. Chain them: schema1.and(schema2).and(schema3)
                let mut result = all_schemas[0].clone();
                for schema in all_schemas.iter().skip(1) {
                    result = format!("{}.and({})", result.trim(), schema.trim());
                }
                Ok(result)
            }
        }
        SchemaKind::AnyOf { any_of, .. } => {
            // AnyOf is treated same as OneOf (union)
            let mut variant_schemas = Vec::new();
            for item in any_of {
                match item {
                    ReferenceOr::Reference { reference } => {
                        if let Some(ref_name) = get_schema_name_from_ref(reference) {
                            variant_schemas.push(format!(
                                "{}z.lazy(() => {}Schema)",
                                indent_str,
                                crate::generator::utils::to_pascal_case(&ref_name)
                            ));
                        } else {
                            variant_schemas.push(format!("{}z.any()", indent_str));
                        }
                    }
                    ReferenceOr::Item(item_schema) => {
                        let item_zod = schema_to_zod(
                            openapi,
                            item_schema,
                            zod_schemas,
                            processed,
                            indent,
                            enum_registry,
                            None,
                            current_schema_name,
                        )?;
                        variant_schemas.push(item_zod);
                    }
                }
            }
            if variant_schemas.is_empty() {
                Ok(format!("{}z.any()", indent_str))
            } else {
                Ok(format!(
                    "{}z.union([{}])",
                    indent_str,
                    variant_schemas.join(", ")
                ))
            }
        }
        SchemaKind::Not { .. } => Ok(format!("{}z.any()", indent_str)),
    }
}

fn generate_enum_zod(name: &str, values: &[String]) -> ZodSchema {
    let enum_name = to_pascal_case(name);
    let enum_values = values
        .iter()
        .map(|v| format!("\"{}\"", v))
        .collect::<Vec<_>>()
        .join(", ");

    ZodSchema {
        content: format!(
            "export const {}Schema = z.enum([{}]);",
            enum_name, enum_values
        ),
    }
}
