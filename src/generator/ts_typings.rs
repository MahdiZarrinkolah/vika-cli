use anyhow::{Context, Result};
use openapiv3::{OpenAPI, ReferenceOr, Schema, SchemaKind, Type};
use std::collections::HashMap;
use crate::generator::swagger_parser::{get_schema_name_from_ref, resolve_ref};
use crate::generator::utils::to_pascal_case;

pub struct TypeScriptType {
    pub content: String,
}

pub fn generate_typings(
    openapi: &OpenAPI,
    schemas: &HashMap<String, Schema>,
    schema_names: &[String],
) -> Result<Vec<TypeScriptType>> {
    let mut types = Vec::new();
    let mut processed = std::collections::HashSet::new();

    for schema_name in schema_names {
        if let Some(schema) = schemas.get(schema_name) {
            generate_type_for_schema(
                openapi,
                schema_name,
                schema,
                &mut types,
                &mut processed,
            )?;
        }
    }

    Ok(types)
}

fn generate_type_for_schema(
    openapi: &OpenAPI,
    name: &str,
    schema: &Schema,
    types: &mut Vec<TypeScriptType>,
    processed: &mut std::collections::HashSet<String>,
) -> Result<()> {
    if processed.contains(name) {
        return Ok(());
    }
    processed.insert(name.to_string());

    let type_name = to_pascal_case(name);
    
    // Handle enums
    if let SchemaKind::Type(Type::String(string_type)) = &schema.schema_kind {
        if !string_type.enumeration.is_empty() {
            let enum_values: Vec<String> = string_type.enumeration.iter()
                .filter_map(|v| v.as_ref().cloned())
                .collect();
            if !enum_values.is_empty() {
                let enum_type = generate_enum_type(name, &enum_values);
                types.push(enum_type);
                return Ok(());
            }
        }
    }
    
    let content = schema_to_typescript(openapi, schema, types, processed, 0)?;
    
    // Only create interface if it's an object type
    if matches!(&schema.schema_kind, SchemaKind::Type(Type::Object(_))) {
        types.push(TypeScriptType {
            content: format!("export interface {} {{\n{}\n}}", type_name, content),
        });
    }

    Ok(())
}

fn schema_to_typescript(
    openapi: &OpenAPI,
    schema: &Schema,
    types: &mut Vec<TypeScriptType>,
    processed: &mut std::collections::HashSet<String>,
    indent: usize,
) -> Result<String> {
    // Prevent infinite recursion with a reasonable depth limit
    if indent > 100 {
        return Ok("any".to_string());
    }
    let indent_str = "  ".repeat(indent);
    
    match &schema.schema_kind {
        SchemaKind::Type(type_) => {
            match type_ {
                Type::String(_) => Ok("string".to_string()),
                Type::Number(_) => Ok("number".to_string()),
                Type::Integer(_) => Ok("number".to_string()),
                Type::Boolean(_) => Ok("boolean".to_string()),
                Type::Array(array) => {
                    let item_type = if let Some(items) = &array.items {
                        match items {
                            ReferenceOr::Reference { reference } => {
                                if let Some(ref_name) = get_schema_name_from_ref(&reference) {
                                    // If already processed, just return the type name to avoid infinite recursion
                                    if processed.contains(&ref_name) {
                                        to_pascal_case(&ref_name)
                                    } else {
                                        let resolved = resolve_ref(openapi, &reference)
                                            .context("Failed to resolve schema reference")?;
                                        if let ReferenceOr::Item(item_schema) = resolved {
                                            schema_to_typescript(openapi, &item_schema, types, processed, indent)?
                                        } else {
                                            to_pascal_case(&ref_name)
                                        }
                                    }
                                } else {
                                    "any".to_string()
                                }
                            }
                            ReferenceOr::Item(item_schema) => {
                                schema_to_typescript(openapi, item_schema, types, processed, indent)?
                            }
                        }
                    } else {
                        "any".to_string()
                    };
                    Ok(format!("{}[]", item_type))
                }
                Type::Object(object_type) => {
                    if !object_type.properties.is_empty() {
                        let mut fields = Vec::new();
                        for (prop_name, prop_schema_ref) in object_type.properties.iter() {
                            let prop_type = match prop_schema_ref {
                            ReferenceOr::Reference { reference } => {
                                if let Some(ref_name) = get_schema_name_from_ref(&reference) {
                                    // If already processed, just return the type name to avoid infinite recursion
                                    if processed.contains(&ref_name) {
                                        to_pascal_case(&ref_name)
                                    } else {
                                        let resolved = resolve_ref(openapi, &reference)
                                            .context("Failed to resolve schema reference")?;
                                        if let ReferenceOr::Item(ref_schema) = resolved {
                                            // Check if it's an object that needs to be extracted
                                            if matches!(&ref_schema.schema_kind, SchemaKind::Type(Type::Object(_))) {
                                                generate_type_for_schema(openapi, &ref_name, &ref_schema, types, processed)?;
                                            }
                                            schema_to_typescript(openapi, &ref_schema, types, processed, indent)?
                                        } else {
                                            to_pascal_case(&ref_name)
                                        }
                                    }
                                } else {
                                    "any".to_string()
                                }
                            }
                                ReferenceOr::Item(prop_schema) => {
                                    schema_to_typescript(openapi, prop_schema, types, processed, indent)?
                                }
                            };

                            let required = object_type.required.contains(prop_name);

                            let optional = if required { "" } else { "?" };
                            let nullable = prop_schema_ref.as_item()
                                .map(|s| s.schema_data.nullable)
                                .unwrap_or(false);
                            
                            let nullable_str = if nullable { " | null" } else { "" };

                            fields.push(format!(
                                "{}{}{}: {}{};",
                                indent_str, prop_name, optional, prop_type, nullable_str
                            ));
                        }
                        Ok(fields.join("\n"))
                    } else {
                        Ok("Record<string, any>".to_string())
                    }
                }
            }
        }
        SchemaKind::Any(_) => Ok("any".to_string()),
        SchemaKind::OneOf { .. } => Ok("any".to_string()),
        SchemaKind::AllOf { .. } => Ok("any".to_string()),
        SchemaKind::AnyOf { .. } => Ok("any".to_string()),
        SchemaKind::Not { .. } => Ok("any".to_string()),
    }
    .map(|base_type| {
        if schema.schema_data.nullable {
            format!("{} | null", base_type)
        } else {
            base_type
        }
    })
}

pub fn generate_enum_type(name: &str, values: &[String]) -> TypeScriptType {
    let enum_values = values
        .iter()
        .map(|v| format!("  \"{}\"", v))
        .collect::<Vec<_>>()
        .join(" |\n");

    TypeScriptType {
        content: format!("export type {} =\n{};", to_pascal_case(name), enum_values),
    }
}

