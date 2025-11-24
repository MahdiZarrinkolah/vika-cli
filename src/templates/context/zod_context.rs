use serde::Serialize;

/// Context for Zod schema generation.
#[derive(Debug, Clone, Serialize)]
pub struct ZodContext {
    pub schema_name: String,
    pub zod_expr: String,
    pub is_enum: bool,
    pub enum_values: Option<Vec<String>>,
    pub description: Option<String>,
    pub needs_type_annotation: bool,
}

impl ZodContext {
    /// Create a new ZodContext for a regular schema.
    pub fn schema(schema_name: String, zod_expr: String, description: Option<String>) -> Self {
        Self {
            schema_name,
            zod_expr,
            is_enum: false,
            enum_values: None,
            description,
            needs_type_annotation: false,
        }
    }

    /// Create a new ZodContext for a schema that needs type annotation (e.g., circular references).
    pub fn schema_with_annotation(
        schema_name: String,
        zod_expr: String,
        description: Option<String>,
    ) -> Self {
        Self {
            schema_name,
            zod_expr,
            is_enum: false,
            enum_values: None,
            description,
            needs_type_annotation: true,
        }
    }

    /// Create a new ZodContext for an enum schema.
    pub fn enum_schema(schema_name: String, enum_values: Vec<String>) -> Self {
        let enum_values_str = enum_values
            .iter()
            .map(|v| format!("\"{}\"", v))
            .collect::<Vec<_>>()
            .join(", ");
        let zod_expr = format!("z.enum([{}])", enum_values_str);

        Self {
            schema_name,
            zod_expr,
            is_enum: true,
            enum_values: Some(enum_values),
            description: None,
            needs_type_annotation: false,
        }
    }
}
