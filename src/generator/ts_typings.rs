use anyhow::{Context, Result};
use openapiv3::{OpenAPI, ReferenceOr, Schema, SchemaKind, Type};
use std::collections::HashMap;
use crate::generator::swagger_parser::{get_schema_name_from_ref, resolve_ref};
use crate::generator::utils::to_pascal_case;

#[derive(Clone)]
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
    // Enum registry: maps enum values (sorted, joined) to enum name
    let mut enum_registry: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    for schema_name in schema_names {
        if let Some(schema) = schemas.get(schema_name) {
            generate_type_for_schema(
                openapi,
                schema_name,
                schema,
                &mut types,
                &mut processed,
                &mut enum_registry,
                None,
            )?;
        }
    }

    Ok(types)
}

#[allow(dead_code)]
pub fn organize_types_by_module(
    types: Vec<TypeScriptType>,
    module_schemas: &std::collections::HashMap<String, Vec<String>>,
) -> std::collections::HashMap<String, Vec<TypeScriptType>> {
    let mut organized: std::collections::HashMap<String, Vec<TypeScriptType>> = std::collections::HashMap::new();
    
    // Organize types by module. Currently, all types are included in each module
    // since schemas can be shared across modules. This could be enhanced to filter
    // types based on actual usage per module for better code organization.
    for (module, _schema_names) in module_schemas {
        organized.insert(module.clone(), types.clone());
    }
    
    organized
}

fn generate_type_for_schema(
    openapi: &OpenAPI,
    name: &str,
    schema: &Schema,
    types: &mut Vec<TypeScriptType>,
    processed: &mut std::collections::HashSet<String>,
    enum_registry: &mut std::collections::HashMap<String, String>,
    parent_schema_name: Option<&str>,
) -> Result<()> {
    if processed.contains(name) {
        return Ok(());
    }
    processed.insert(name.to_string());

    let type_name = to_pascal_case(name);
    
    // Handle enums
    if let SchemaKind::Type(Type::String(string_type)) = &schema.schema_kind {
        if !string_type.enumeration.is_empty() {
            let mut enum_values: Vec<String> = string_type.enumeration.iter()
                .filter_map(|v| v.as_ref().cloned())
                .collect();
            if !enum_values.is_empty() {
                // Create a key from sorted enum values to check registry
                enum_values.sort();
                let enum_key = enum_values.join(",");
                
                // Check if this enum already exists in registry
                if enum_registry.contains_key(&enum_key) {
                    // Enum already generated, skip
                    return Ok(());
                }
                
                // Generate meaningful enum name
                let enum_name = if !name.is_empty() {
                    format!("{}Enum", to_pascal_case(name))
                } else if enum_values.len() > 0 {
                    let first_value = &enum_values[0];
                    let base_name = first_value.chars().take(1).collect::<String>().to_uppercase() 
                        + &first_value.chars().skip(1).collect::<String>();
                    format!("{}Enum", to_pascal_case(&base_name))
                } else {
                    "UnknownEnum".to_string()
                };
                
                // Store in registry
                enum_registry.insert(enum_key, enum_name.clone());
                
                let enum_type = generate_enum_type(&enum_name, &enum_values);
                types.push(enum_type);
                return Ok(());
            }
        }
    }
    
    let content = schema_to_typescript(openapi, schema, types, processed, 0, enum_registry, None, parent_schema_name)?;
    
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
    enum_registry: &mut std::collections::HashMap<String, String>,
    context: Option<(&str, &str)>, // (property_name, parent_schema_name)
    current_schema_name: Option<&str>, // Current schema being processed (for enum naming context)
) -> Result<String> {
    // Prevent infinite recursion with a reasonable depth limit
    if indent > 100 {
        return Ok("any".to_string());
    }
    let indent_str = "  ".repeat(indent);
    
    match &schema.schema_kind {
        SchemaKind::Type(type_) => {
            match type_ {
                Type::String(string_type) => {
                    // Check if it's an enum
                    if !string_type.enumeration.is_empty() {
                        let mut enum_values: Vec<String> = string_type.enumeration.iter()
                            .filter_map(|v| v.as_ref().cloned())
                            .collect();
                        enum_values.sort();
                        let enum_key = enum_values.join(",");
                        
                        // Check registry for existing enum
                        if let Some(enum_name) = enum_registry.get(&enum_key) {
                            Ok(enum_name.clone())
                        } else {
                            // Generate meaningful enum name using context (property name + parent schema) or fallback
                            let enum_name = if let Some((prop_name, parent_schema)) = context {
                                // Use property name + parent schema for meaningful name to avoid conflicts
                                // For generic names like "status", use parent schema to differentiate
                                // e.g., "status" in "KycStatusResponseDto" -> "KycStatusEnum" (parent already has "Status")
                                //      "status" in "TenantResponseDto" -> "TenantStatusEnum"
                                let prop_pascal = to_pascal_case(prop_name);
                                
                                // If property name is generic (status, type, etc.), use parent schema
                                let generic_names = ["status", "type", "state", "kind"];
                                if generic_names.contains(&prop_name.to_lowercase().as_str()) && !parent_schema.is_empty() {
                                    let parent_pascal = to_pascal_case(parent_schema);
                                    // Remove common suffixes from parent schema name
                                    let parent_clean = parent_pascal
                                        .trim_end_matches("ResponseDto")
                                        .trim_end_matches("Dto")
                                        .trim_end_matches("Response")
                                        .to_string();
                                    
                                    // Check if parent already contains the property name (e.g., "KycStatus" contains "Status")
                                    let prop_lower = prop_pascal.to_lowercase();
                                    let parent_lower = parent_clean.to_lowercase();
                                    if parent_lower.contains(&prop_lower) {
                                        // Parent already contains property name, just use parent + Enum
                                        format!("{}Enum", parent_clean)
                                    } else {
                                        // Combine parent + property
                                        format!("{}{}Enum", parent_clean, prop_pascal)
                                    }
                                } else {
                                    format!("{}Enum", prop_pascal)
                                }
                            } else if enum_values.len() > 0 {
                                // Fallback: use first value to create name
                                let first_value = &enum_values[0];
                                let base_name = first_value.chars().take(1).collect::<String>().to_uppercase() 
                                    + &first_value.chars().skip(1).collect::<String>();
                                format!("{}Enum", to_pascal_case(&base_name))
                            } else {
                                "UnknownEnum".to_string()
                            };
                            enum_registry.insert(enum_key, enum_name.clone());
                            
                            // Generate enum type
                            let enum_type = generate_enum_type(&enum_name, &enum_values);
                            types.push(enum_type);
                            
                            Ok(enum_name)
                        }
                    } else {
                        Ok("string".to_string())
                    }
                }
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
                                            // If it's an object, wrap the fields in braces
                                            if matches!(&item_schema.schema_kind, SchemaKind::Type(Type::Object(_))) {
                                                let fields = schema_to_typescript(openapi, &item_schema, types, processed, indent, enum_registry, None, current_schema_name)?;
                                                format!("{{\n{}{}\n{}}}", indent_str, fields, indent_str)
                                            } else {
                                                schema_to_typescript(openapi, &item_schema, types, processed, indent, enum_registry, None, current_schema_name)?
                                            }
                                        } else {
                                            to_pascal_case(&ref_name)
                                        }
                                    }
                                } else {
                                    "any".to_string()
                                }
                            }
                            ReferenceOr::Item(item_schema) => {
                                // If it's an object, wrap the fields in braces
                                if matches!(&item_schema.schema_kind, SchemaKind::Type(Type::Object(_))) {
                                    let fields = schema_to_typescript(openapi, item_schema, types, processed, indent, enum_registry, None, current_schema_name)?;
                                    format!("{{\n{}{}\n{}}}", indent_str, fields, indent_str)
                                } else {
                                    schema_to_typescript(openapi, item_schema, types, processed, indent, enum_registry, None, current_schema_name)?
                                }
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
                        // Get parent schema name from context if available, otherwise use current_schema_name parameter
                        // For object properties, use the current schema name as parent
                        let parent_schema_for_props = context.and_then(|(_, parent)| {
                            if !parent.is_empty() { Some(parent.to_string()) } else { None }
                        }).or_else(|| current_schema_name.map(|s| s.to_string()))
                        .unwrap_or_else(|| String::new());
                        
                        for (prop_name, prop_schema_ref) in object_type.properties.iter() {
                            let prop_type = match prop_schema_ref {
                            ReferenceOr::Reference { reference } => {
                                // For $ref properties, always use the type name (don't inline)
                                if let Some(ref_name) = get_schema_name_from_ref(&reference) {
                                    // Generate the referenced schema if not already processed
                                    if !processed.contains(&ref_name) {
                                        if let Ok(resolved) = resolve_ref(openapi, &reference) {
                                            if let ReferenceOr::Item(ref_schema) = resolved {
                                                generate_type_for_schema(openapi, &ref_name, &ref_schema, types, processed, enum_registry, Some(&parent_schema_for_props))?;
                                            }
                                        }
                                    }
                                    to_pascal_case(&ref_name)
                                } else {
                                    "any".to_string()
                                }
                            }
                                ReferenceOr::Item(prop_schema) => {
                                    schema_to_typescript(openapi, prop_schema, types, processed, indent, enum_registry, Some((prop_name, &parent_schema_for_props)), current_schema_name)?
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
        SchemaKind::OneOf { one_of, .. } => {
            let mut variant_types = Vec::new();
            for item in one_of {
                match item {
                    ReferenceOr::Reference { reference } => {
                        if let Some(ref_name) = get_schema_name_from_ref(&reference) {
                            variant_types.push(to_pascal_case(&ref_name));
                        } else {
                            variant_types.push("any".to_string());
                        }
                    }
                    ReferenceOr::Item(item_schema) => {
                        let item_type = schema_to_typescript(openapi, item_schema, types, processed, indent, enum_registry, None, current_schema_name)?;
                        variant_types.push(item_type);
                    }
                }
            }
            if variant_types.is_empty() {
                Ok("any".to_string())
            } else {
                Ok(variant_types.join(" | "))
            }
        }
        SchemaKind::AllOf { all_of, .. } => {
            let mut all_types = Vec::new();
            for item in all_of {
                match item {
                    ReferenceOr::Reference { reference } => {
                        if let Some(ref_name) = get_schema_name_from_ref(&reference) {
                            all_types.push(to_pascal_case(&ref_name));
                        } else {
                            all_types.push("any".to_string());
                        }
                    }
                    ReferenceOr::Item(item_schema) => {
                        let item_type = schema_to_typescript(openapi, item_schema, types, processed, indent, enum_registry, None, current_schema_name)?;
                        all_types.push(item_type);
                    }
                }
            }
            if all_types.is_empty() {
                Ok("any".to_string())
            } else {
                Ok(all_types.join(" & "))
            }
        }
        SchemaKind::AnyOf { any_of, .. } => {
            // AnyOf is treated same as OneOf (union type)
            let mut variant_types = Vec::new();
            for item in any_of {
                match item {
                    ReferenceOr::Reference { reference } => {
                        if let Some(ref_name) = get_schema_name_from_ref(&reference) {
                            variant_types.push(to_pascal_case(&ref_name));
                        } else {
                            variant_types.push("any".to_string());
                        }
                    }
                    ReferenceOr::Item(item_schema) => {
                        let item_type = schema_to_typescript(openapi, item_schema, types, processed, indent, enum_registry, None, current_schema_name)?;
                        variant_types.push(item_type);
                    }
                }
            }
            if variant_types.is_empty() {
                Ok("any".to_string())
            } else {
                Ok(variant_types.join(" | "))
            }
        }
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

