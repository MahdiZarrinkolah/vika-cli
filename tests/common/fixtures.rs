use openapiv3::{OpenAPI, Schema, SchemaKind, Type, ObjectType, SchemaData, Tag};
use indexmap::IndexMap;

/// Create a minimal valid OpenAPI 3.0 spec for testing
pub fn create_minimal_openapi_spec() -> OpenAPI {
    let mut openapi = OpenAPI::default();
    openapi.openapi = "3.0.0".to_string();
    openapi.info.title = "Test API".to_string();
    openapi.info.version = "1.0.0".to_string();
    openapi
}

/// Create an OpenAPI spec with multiple modules/tags
pub fn create_multi_module_spec() -> OpenAPI {
    let mut openapi = create_minimal_openapi_spec();
    
    // Add tags
    openapi.tags.push(Tag {
        name: "users".to_string(),
        description: None,
        external_docs: None,
        extensions: IndexMap::new(),
    });
    openapi.tags.push(Tag {
        name: "products".to_string(),
        description: None,
        external_docs: None,
        extensions: IndexMap::new(),
    });
    
    // Add a simple schema
    let mut components = openapiv3::Components::default();
    let user_schema = create_user_schema();
    components.schemas.insert("User".to_string(), openapiv3::ReferenceOr::Item(user_schema));
    openapi.components = Some(components);
    
    openapi
}

/// Create an OpenAPI spec with common schemas
pub fn create_common_schema_spec() -> OpenAPI {
    let mut openapi = create_minimal_openapi_spec();
    
    // Add tags
    openapi.tags.push(Tag {
        name: "users".to_string(),
        description: None,
        external_docs: None,
        extensions: IndexMap::new(),
    });
    openapi.tags.push(Tag {
        name: "products".to_string(),
        description: None,
        external_docs: None,
        extensions: IndexMap::new(),
    });
    
    // Add common schema (used by both modules)
    let mut components = openapiv3::Components::default();
    let common_schema = create_common_schema();
    components.schemas.insert("CommonResponse".to_string(), openapiv3::ReferenceOr::Item(common_schema));
    
    let user_schema = create_user_schema();
    components.schemas.insert("User".to_string(), openapiv3::ReferenceOr::Item(user_schema));
    
    openapi.components = Some(components);
    
    openapi
}

/// Create a schema with enum values
pub fn create_enum_schema() -> Schema {
    let mut string_type = openapiv3::StringType::default();
    string_type.enumeration = vec![
        Some("active".to_string()),
        Some("inactive".to_string()),
        Some("pending".to_string()),
    ];
    
    Schema {
        schema_data: SchemaData::default(),
        schema_kind: SchemaKind::Type(Type::String(string_type)),
    }
}

/// Create a simple object schema
pub fn create_user_schema() -> Schema {
    let mut object_type = ObjectType::default();
    object_type.properties.insert(
        "id".to_string(),
        openapiv3::ReferenceOr::Item(Box::new(Schema {
            schema_data: SchemaData::default(),
            schema_kind: SchemaKind::Type(Type::String(openapiv3::StringType::default())),
        })),
    );
    object_type.properties.insert(
        "name".to_string(),
        openapiv3::ReferenceOr::Item(Box::new(Schema {
            schema_data: SchemaData::default(),
            schema_kind: SchemaKind::Type(Type::String(openapiv3::StringType::default())),
        })),
    );
    object_type.required.push("id".to_string());
    object_type.required.push("name".to_string());
    
    Schema {
        schema_data: SchemaData::default(),
        schema_kind: SchemaKind::Type(Type::Object(object_type)),
    }
}

/// Create a common schema (used by multiple modules)
pub fn create_common_schema() -> Schema {
    let mut object_type = ObjectType::default();
    object_type.properties.insert(
        "status".to_string(),
        openapiv3::ReferenceOr::Item(Box::new(Schema {
            schema_data: SchemaData::default(),
            schema_kind: SchemaKind::Type(Type::String(openapiv3::StringType::default())),
        })),
    );
    object_type.required.push("status".to_string());
    
    Schema {
        schema_data: SchemaData::default(),
        schema_kind: SchemaKind::Type(Type::Object(object_type)),
    }
}

/// Create an array schema
pub fn create_array_schema() -> Schema {
    let array_type = openapiv3::ArrayType {
        items: Some(openapiv3::ReferenceOr::Item(Box::new(Schema {
            schema_data: SchemaData::default(),
            schema_kind: SchemaKind::Type(Type::String(openapiv3::StringType::default())),
        }))),
        min_items: None,
        max_items: None,
        unique_items: false,
    };
    
    Schema {
        schema_data: SchemaData::default(),
        schema_kind: SchemaKind::Type(Type::Array(array_type)),
    }
}

/// Create a nested object schema
pub fn create_nested_schema() -> Schema {
    let mut inner_object = ObjectType::default();
    inner_object.properties.insert(
        "value".to_string(),
        openapiv3::ReferenceOr::Item(Box::new(Schema {
            schema_data: SchemaData::default(),
            schema_kind: SchemaKind::Type(Type::Integer(openapiv3::IntegerType::default())),
        })),
    );
    
    let mut outer_object = ObjectType::default();
    outer_object.properties.insert(
        "nested".to_string(),
        openapiv3::ReferenceOr::Item(Box::new(Schema {
            schema_data: SchemaData::default(),
            schema_kind: SchemaKind::Type(Type::Object(inner_object)),
        })),
    );
    
    Schema {
        schema_data: SchemaData::default(),
        schema_kind: SchemaKind::Type(Type::Object(outer_object)),
    }
}

/// Create a schema with nullable field
pub fn create_nullable_schema() -> Schema {
    let mut object_type = ObjectType::default();
    let mut nullable_schema = Schema {
        schema_data: SchemaData {
            nullable: true,
            ..Default::default()
        },
        schema_kind: SchemaKind::Type(Type::String(openapiv3::StringType::default())),
    };
    
    object_type.properties.insert(
        "optional_field".to_string(),
        openapiv3::ReferenceOr::Item(Box::new(nullable_schema)),
    );
    
    Schema {
        schema_data: SchemaData::default(),
        schema_kind: SchemaKind::Type(Type::Object(object_type)),
    }
}

