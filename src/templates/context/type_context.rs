use serde::Serialize;

/// Context for TypeScript interface/type generation.
#[derive(Debug, Clone, Serialize)]
pub struct TypeContext {
    pub type_name: String,
    pub fields: Vec<Field>,
    pub is_enum: bool,
    pub enum_values: Option<Vec<String>>,
    pub is_alias: bool,
    pub alias_target: Option<String>,
    pub description: Option<String>,
    /// Spec name (for multi-spec mode)
    pub spec_name: Option<String>,
}

/// Field information for TypeScript interfaces.
#[derive(Debug, Clone, Serialize)]
pub struct Field {
    pub name: String,
    pub type_name: String,
    pub optional: bool,
    pub description: Option<String>,
}

impl TypeContext {
    /// Create a new TypeContext for an interface.
    pub fn interface(
        type_name: String,
        fields: Vec<Field>,
        description: Option<String>,
        spec_name: Option<String>,
    ) -> Self {
        Self {
            type_name,
            fields,
            is_enum: false,
            enum_values: None,
            is_alias: false,
            alias_target: None,
            description,
            spec_name,
        }
    }

    /// Create a new TypeContext for an enum type.
    pub fn enum_type(
        type_name: String,
        enum_values: Vec<String>,
        spec_name: Option<String>,
    ) -> Self {
        Self {
            type_name,
            fields: Vec::new(),
            is_enum: true,
            enum_values: Some(enum_values),
            is_alias: false,
            alias_target: None,
            description: None,
            spec_name,
        }
    }

    /// Create a new TypeContext for a type alias.
    pub fn alias(type_name: String, alias_target: String, spec_name: Option<String>) -> Self {
        Self {
            type_name,
            fields: Vec::new(),
            is_enum: false,
            enum_values: None,
            is_alias: true,
            alias_target: Some(alias_target),
            description: None,
            spec_name,
        }
    }
}

impl Field {
    /// Create a new Field.
    pub fn new(
        name: String,
        type_name: String,
        optional: bool,
        description: Option<String>,
    ) -> Self {
        Self {
            name,
            type_name,
            optional,
            description,
        }
    }
}
