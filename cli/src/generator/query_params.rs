use crate::error::Result;
use crate::generator::api_client::{extract_query_parameters, ParameterType};
use crate::generator::swagger_parser::OperationInfo;
use crate::generator::ts_typings::TypeScriptType;
use crate::generator::utils::to_pascal_case;
use crate::generator::zod_schema::ZodSchema;
use crate::templates::context::{Field, TypeContext, ZodContext};
use crate::templates::engine::TemplateEngine;
use crate::templates::registry::TemplateId;
use openapiv3::OpenAPI;
use std::collections::HashMap;

pub struct QueryParamsGenerationResult {
    pub types: Vec<TypeScriptType>,
    pub zod_schemas: Vec<ZodSchema>,
}

/// Context for generating query params
pub struct QueryParamsContext<'a> {
    pub openapi: &'a OpenAPI,
    pub operations: &'a [OperationInfo],
    pub enum_registry: &'a mut HashMap<String, String>,
    pub template_engine: Option<&'a TemplateEngine>,
    pub spec_name: Option<&'a str>,
    pub existing_types: &'a [TypeScriptType],
    pub existing_zod_schemas: &'a [ZodSchema],
}

/// Generate query params types and Zod schemas for all operations in a module
pub fn generate_query_params_for_module(
    ctx: QueryParamsContext,
) -> Result<QueryParamsGenerationResult> {
    let QueryParamsContext {
        openapi,
        operations,
        enum_registry,
        template_engine,
        spec_name,
        existing_types,
        existing_zod_schemas,
    } = ctx;
    let mut types = Vec::new();
    let mut zod_schemas = Vec::new();

    // Build a set of existing type names to avoid duplicates
    let mut existing_type_names = std::collections::HashSet::new();
    for t in existing_types {
        // Extract type name from content: "export type XEnum = ..." or "export interface X { ... }"
        if let Some(name) = extract_type_name(&t.content) {
            existing_type_names.insert(name);
        }
    }

    // Build a set of existing schema names to avoid duplicates
    let mut existing_schema_names = std::collections::HashSet::new();
    for z in existing_zod_schemas {
        // Extract schema name from content: "export const XSchema = ..."
        if let Some(name) = extract_schema_name(&z.content) {
            existing_schema_names.insert(name);
        }
    }

    // Helper function to extract type name from content
    fn extract_type_name(content: &str) -> Option<String> {
        content.find("export type ").and_then(|start| {
            let after_export = &content[start + 12..];
            after_export
                .find([' ', '=', '{'])
                .map(|end| after_export[..end].trim().to_string())
        })
    }

    // Helper function to extract schema name from content
    fn extract_schema_name(content: &str) -> Option<String> {
        content.find("export const ").and_then(|start| {
            let after_export = &content[start + 13..];
            after_export
                .find([' ', '=', ':'])
                .map(|end| after_export[..end].trim().to_string())
        })
    }

    for op_info in operations {
        let operation = &op_info.operation;
        let query_params = extract_query_parameters(openapi, operation, enum_registry)?;

        if query_params.is_empty() {
            continue;
        }

        // Extract operation_id from operation
        let func_name = operation.operation_id.clone().unwrap_or_else(|| {
            // Fallback: generate from method and path
            format!(
                "{}{}",
                to_pascal_case(&op_info.method),
                to_pascal_case(
                    &op_info
                        .path
                        .replace("/", "")
                        .replace("{", "")
                        .replace("}", "")
                )
            )
        });
        let type_name_base = to_pascal_case(&func_name);
        let query_type_name = format!("{}QueryParams", type_name_base);
        // Schema name without "Schema" suffix - template will add it
        let schema_name = query_type_name.clone();

        // Collect enum types from query params and generate them
        // We need to track which enums we've already generated in this module to avoid duplicates
        // Skip enums that already exist from schema definitions
        let mut enum_types_to_generate = Vec::new();
        let mut generated_enum_names = std::collections::HashSet::new();

        for param in &query_params {
            if let ParameterType::Enum(enum_name) = &param.param_type {
                if let Some(enum_values) = &param.enum_values {
                    // Skip if enum type already exists from schema definitions
                    let enum_schema_name = format!("{}Schema", enum_name);
                    if !generated_enum_names.contains(enum_name)
                        && !existing_type_names.contains(enum_name)
                        && !existing_schema_names.contains(&enum_schema_name)
                    {
                        enum_types_to_generate.push((enum_name.clone(), enum_values.clone()));
                        generated_enum_names.insert(enum_name.clone());
                        // Register the enum to prevent duplicate generation within this module
                        let enum_key = enum_values.join(",");
                        enum_registry.insert(enum_key, enum_name.clone());
                    }
                }
            }
        }

        // Generate enum types first (before query params interface)
        for (enum_name, enum_values) in &enum_types_to_generate {
            // Generate TypeScript enum type
            let enum_type_content = format!(
                "export type {} =\n{}\n;",
                enum_name,
                enum_values
                    .iter()
                    .map(|v| format!("\"{}\"", v))
                    .collect::<Vec<_>>()
                    .join(" |\n")
            );
            types.push(TypeScriptType {
                content: enum_type_content,
            });

            // Generate Zod enum schema
            // Template adds "Schema" suffix, so we pass just the enum name
            // The final schema name will be "{enum_name}Schema"
            if let Some(engine) = template_engine {
                let zod_context = ZodContext {
                    schema_name: enum_name.clone(),
                    zod_expr: format!(
                        "z.enum([{}])",
                        enum_values
                            .iter()
                            .map(|v| format!("\"{}\"", v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    is_enum: true,
                    enum_values: Some(enum_values.clone()),
                    description: None,
                    needs_type_annotation: false,
                    spec_name: spec_name.map(|s| s.to_string()),
                };
                let zod_content = engine.render(TemplateId::ZodEnum, &zod_context)?;
                zod_schemas.push(ZodSchema {
                    content: zod_content,
                });
            } else {
                // Fallback without template
                let enum_values_str = enum_values
                    .iter()
                    .map(|v| format!("\"{}\"", v))
                    .collect::<Vec<_>>()
                    .join(", ");
                zod_schemas.push(ZodSchema {
                    content: format!(
                        "export const {}Schema = z.enum([{}]);",
                        enum_name, enum_values_str
                    ),
                });
            }
        }

        // Generate TypeScript interface fields
        let mut fields = Vec::new();
        for param in &query_params {
            let param_type = match &param.param_type {
                ParameterType::Enum(enum_name) => enum_name.clone(),
                ParameterType::Array(item_type) => format!("{}[]", item_type),
                ParameterType::String => "string".to_string(),
                ParameterType::Number => "number".to_string(),
                ParameterType::Integer => "number".to_string(),
                ParameterType::Boolean => "boolean".to_string(),
            };

            fields.push(Field {
                name: param.name.clone(),
                type_name: param_type,
                optional: true,
                description: param.description.clone(),
            });
        }

        // Generate TypeScript type using template
        if let Some(engine) = template_engine {
            let context = TypeContext::interface(
                query_type_name.clone(),
                fields,
                None,
                spec_name.map(|s| s.to_string()),
            );
            let content = engine.render(TemplateId::TypeInterface, &context)?;
            types.push(TypeScriptType { content });
        } else {
            // Fallback without template
            let mut field_strings = Vec::new();
            for field in &fields {
                let desc = field
                    .description
                    .as_ref()
                    .map(|d| format!("  /**\n   * {}\n   */\n  ", d))
                    .unwrap_or_default();
                field_strings.push(format!("{}{}?: {};", desc, field.name, field.type_name));
            }
            types.push(TypeScriptType {
                content: format!(
                    "export interface {} {{\n{}\n}}",
                    query_type_name,
                    field_strings.join("\n")
                ),
            });
        }

        // Generate Zod schema expression
        let mut zod_field_strings = Vec::new();
        for param in &query_params {
            let zod_type = match &param.param_type {
                ParameterType::Enum(enum_name) => {
                    // For enums, use the enum schema that was generated above
                    // Enum schemas are in the same file (schemas.ts), so reference directly
                    // Template adds "Schema" suffix, so enum schema name is "{enum_name}Schema"
                    format!("{}Schema", enum_name)
                }
                ParameterType::Array(item_type) => {
                    match item_type.as_str() {
                        "string" => "z.array(z.string())".to_string(),
                        "number" => "z.array(z.number())".to_string(),
                        "boolean" => "z.array(z.boolean())".to_string(),
                        _ => "z.array(z.any())".to_string(), // For custom types
                    }
                }
                ParameterType::String => "z.string()".to_string(),
                ParameterType::Number => "z.number()".to_string(),
                ParameterType::Integer => "z.number()".to_string(),
                ParameterType::Boolean => "z.boolean()".to_string(),
            };

            let optional_zod = format!("{}.optional()", zod_type);
            zod_field_strings.push(format!("  {}: {},", param.name, optional_zod));
        }

        let zod_expr = format!("z.object({{\n{}\n}})", zod_field_strings.join("\n"));

        // Generate Zod schema using template
        if let Some(engine) = template_engine {
            // Template adds "Schema" suffix, so we pass the base name without it
            let zod_context = ZodContext {
                schema_name: schema_name.clone(),
                zod_expr,
                is_enum: false,
                enum_values: None,
                description: None,
                needs_type_annotation: false,
                spec_name: spec_name.map(|s| s.to_string()),
            };
            let zod_content = engine.render(TemplateId::ZodSchema, &zod_context)?;
            zod_schemas.push(ZodSchema {
                content: zod_content,
            });
        } else {
            // Fallback without template
            zod_schemas.push(ZodSchema {
                content: format!("export const {} = {};", schema_name, zod_expr),
            });
        }
    }

    Ok(QueryParamsGenerationResult { types, zod_schemas })
}
