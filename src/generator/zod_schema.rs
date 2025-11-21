use anyhow::{Context, Result};
use openapiv3::{OpenAPI, ReferenceOr, Schema, SchemaKind, Type};
use std::collections::HashMap;
use crate::generator::swagger_parser::{get_schema_name_from_ref, resolve_ref};
use crate::generator::utils::to_pascal_case;

pub struct ZodSchema {
    pub content: String,
}

pub fn generate_zod_schemas(
    openapi: &OpenAPI,
    schemas: &HashMap<String, Schema>,
    schema_names: &[String],
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
) -> Result<()> {
    if processed.contains(name) {
        return Ok(());
    }
    processed.insert(name.to_string());

    let schema_name = to_pascal_case(name);
    let zod_def = schema_to_zod(openapi, schema, zod_schemas, processed, 0)?;

    // Handle enums
    if let SchemaKind::Type(Type::String(string_type)) = &schema.schema_kind {
        if !string_type.enumeration.is_empty() {
            let enum_values: Vec<String> = string_type.enumeration.iter()
                .filter_map(|v| v.as_ref().cloned())
                .collect();
            if !enum_values.is_empty() {
                let enum_schema = generate_enum_zod(name, &enum_values);
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

fn schema_to_zod(
    openapi: &OpenAPI,
    schema: &Schema,
    zod_schemas: &mut Vec<ZodSchema>,
    processed: &mut std::collections::HashSet<String>,
    indent: usize,
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
                        let enum_values: Vec<String> = string_type.enumeration.iter()
                            .filter_map(|v| v.as_ref().cloned())
                            .collect();
                        if !enum_values.is_empty() {
                            let enum_name = format!("Enum{}", processed.len());
                            let enum_schema = generate_enum_zod(&enum_name, &enum_values);
                            zod_schemas.push(enum_schema);
                            return Ok(format!("{}z.enum([{}])", 
                                indent_str,
                                enum_values.iter()
                                    .map(|v| format!("\"{}\"", v))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ));
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
                            "date-time" | "datetime" | "date_time" => zod_expr = format!("{}.datetime()", zod_expr),
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
                                if let Some(ref_name) = get_schema_name_from_ref(&reference) {
                                    // If already processed, use lazy reference to avoid infinite recursion
                                    if processed.contains(&ref_name) {
                                        format!("{}z.lazy(() => {}Schema)", indent_str, to_pascal_case(&ref_name))
                                    } else {
                                        let resolved = resolve_ref(openapi, &reference)
                                            .context("Failed to resolve schema reference")?;
                                        if let ReferenceOr::Item(item_schema) = resolved {
                                            // Check if it's an object that needs to be extracted
                                            if matches!(&item_schema.schema_kind, SchemaKind::Type(Type::Object(_))) {
                                                generate_zod_for_schema(openapi, &ref_name, &item_schema, zod_schemas, processed)?;
                                            }
                                            schema_to_zod(openapi, &item_schema, zod_schemas, processed, indent)?
                                        } else {
                                            format!("{}z.lazy(() => {}Schema)", indent_str, to_pascal_case(&ref_name))
                                        }
                                    }
                                } else {
                                    format!("{}z.any()", indent_str)
                                }
                            }
                            ReferenceOr::Item(item_schema) => {
                                schema_to_zod(openapi, item_schema, zod_schemas, processed, indent)?
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
                        for (prop_name, prop_schema_ref) in object_type.properties.iter() {
                            let prop_zod = match prop_schema_ref {
                            ReferenceOr::Reference { reference } => {
                                if let Some(ref_name) = get_schema_name_from_ref(&reference) {
                                    // If already processed, use lazy reference to avoid infinite recursion
                                    if processed.contains(&ref_name) {
                                        format!("{}z.lazy(() => {}Schema)", indent_str, to_pascal_case(&ref_name))
                                    } else {
                                        let resolved = resolve_ref(openapi, &reference)
                                            .context("Failed to resolve schema reference")?;
                                        if let ReferenceOr::Item(ref_schema) = resolved {
                                            // Check if it's an object that needs to be extracted
                                            if matches!(&ref_schema.schema_kind, SchemaKind::Type(Type::Object(_))) {
                                                generate_zod_for_schema(openapi, &ref_name, &ref_schema, zod_schemas, processed)?;
                                            }
                                            schema_to_zod(openapi, &ref_schema, zod_schemas, processed, indent + 1)?
                                        } else {
                                            format!("{}z.lazy(() => {}Schema)", indent_str, to_pascal_case(&ref_name))
                                        }
                                    }
                                } else {
                                    format!("{}z.any()", indent_str)
                                }
                            }
                                ReferenceOr::Item(prop_schema) => {
                                    schema_to_zod(openapi, prop_schema, zod_schemas, processed, indent + 1)?
                                }
                            };

                            let required = object_type.required.contains(prop_name);

                            let nullable = prop_schema_ref.as_item()
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
                        if let Some(ref_name) = get_schema_name_from_ref(&reference) {
                            variant_schemas.push(format!("{}z.lazy(() => {}Schema)", indent_str, crate::generator::utils::to_pascal_case(&ref_name)));
                        } else {
                            variant_schemas.push(format!("{}z.any()", indent_str));
                        }
                    }
                    ReferenceOr::Item(item_schema) => {
                        let item_zod = schema_to_zod(openapi, item_schema, zod_schemas, processed, indent)?;
                        variant_schemas.push(item_zod);
                    }
                }
            }
            if variant_schemas.is_empty() {
                Ok(format!("{}z.any()", indent_str))
            } else {
                Ok(format!("{}z.union([{}])", indent_str, variant_schemas.join(", ")))
            }
        }
        SchemaKind::AllOf { all_of, .. } => {
            let mut all_schemas = Vec::new();
            for item in all_of {
                match item {
                    ReferenceOr::Reference { reference } => {
                        if let Some(ref_name) = get_schema_name_from_ref(&reference) {
                            all_schemas.push(format!("{}z.lazy(() => {}Schema)", indent_str, crate::generator::utils::to_pascal_case(&ref_name)));
                        } else {
                            all_schemas.push(format!("{}z.any()", indent_str));
                        }
                    }
                    ReferenceOr::Item(item_schema) => {
                        let item_zod = schema_to_zod(openapi, item_schema, zod_schemas, processed, indent)?;
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
                        if let Some(ref_name) = get_schema_name_from_ref(&reference) {
                            variant_schemas.push(format!("{}z.lazy(() => {}Schema)", indent_str, crate::generator::utils::to_pascal_case(&ref_name)));
                        } else {
                            variant_schemas.push(format!("{}z.any()", indent_str));
                        }
                    }
                    ReferenceOr::Item(item_schema) => {
                        let item_zod = schema_to_zod(openapi, item_schema, zod_schemas, processed, indent)?;
                        variant_schemas.push(item_zod);
                    }
                }
            }
            if variant_schemas.is_empty() {
                Ok(format!("{}z.any()", indent_str))
            } else {
                Ok(format!("{}z.union([{}])", indent_str, variant_schemas.join(", ")))
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

