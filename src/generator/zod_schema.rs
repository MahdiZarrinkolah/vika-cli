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
                    Ok(format!("{}z.string()", indent_str))
                }
                Type::Number(_) => Ok(format!("{}z.number()", indent_str)),
                Type::Integer(_) => Ok(format!("{}z.number().int()", indent_str)),
                Type::Boolean(_) => Ok(format!("{}z.boolean()", indent_str)),
                Type::Array(array) => {
                    let item_zod = if let Some(items) = &array.items {
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
                    Ok(format!("{}z.array({})", indent_str, item_zod.trim_start()))
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
        SchemaKind::OneOf { .. } => Ok(format!("{}z.any()", indent_str)),
        SchemaKind::AllOf { .. } => Ok(format!("{}z.any()", indent_str)),
        SchemaKind::AnyOf { .. } => Ok(format!("{}z.any()", indent_str)),
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

