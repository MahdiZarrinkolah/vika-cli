use std::str::FromStr;
use vika_cli::templates::context::{
    ApiContext, Parameter, RequestBody, Response, TypeContext, ZodContext,
};
use vika_cli::templates::engine::TemplateEngine;
use vika_cli::templates::loader::TemplateLoader;
use vika_cli::templates::registry::TemplateId;
use vika_cli::templates::resolver::TemplateResolver;

#[test]
fn test_template_loader_loads_builtin() {
    let result = TemplateLoader::load_builtin("type-interface");
    assert!(result.is_ok());
    let content = result.unwrap();
    assert!(content.contains("export interface"));
}

#[test]
fn test_template_loader_list_builtin() {
    let templates = TemplateLoader::list_builtin();
    assert!(!templates.is_empty());
    assert!(templates.contains(&"type-interface".to_string()));
    assert!(templates.contains(&"zod-schema".to_string()));
    assert!(templates.contains(&"api-client-fetch".to_string()));
}

#[test]
fn test_template_resolver_resolves_builtin() {
    let resolver = TemplateResolver::new(None);
    let result = resolver.resolve(TemplateId::TypeInterface);
    assert!(result.is_ok());
    let content = result.unwrap();
    assert!(content.contains("export interface"));
}

#[test]
fn test_template_resolver_list_templates() {
    let resolver = TemplateResolver::new(None);
    let templates = resolver.list_templates().unwrap();
    assert!(!templates.is_empty());
    assert!(templates.iter().any(|(name, _)| name == "type-interface"));
}

#[test]
fn test_template_engine_new() {
    let engine = TemplateEngine::new(None);
    assert!(engine.is_ok());
}

#[test]
fn test_template_engine_render_type_enum() {
    let engine = TemplateEngine::new(None).unwrap();
    let context = TypeContext::enum_type(
        "TestEnum".to_string(),
        vec!["A".to_string(), "B".to_string(), "C".to_string()],
    );
    let result = engine.render(TemplateId::TypeEnum, &context);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("TestEnum"));
    assert!(output.contains("\"A\""));
    assert!(output.contains("\"B\""));
    assert!(output.contains("\"C\""));
}

#[test]
fn test_template_engine_render_type_interface() {
    let engine = TemplateEngine::new(None).unwrap();
    let fields = vec![
        vika_cli::templates::context::Field::new(
            "name".to_string(),
            "string".to_string(),
            false,
            Some("User name".to_string()),
        ),
        vika_cli::templates::context::Field::new(
            "age".to_string(),
            "number".to_string(),
            true,
            None,
        ),
    ];
    let context = TypeContext::interface("User".to_string(), fields, None);
    let result = engine.render(TemplateId::TypeInterface, &context);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("export interface User"));
    assert!(output.contains("name"));
    assert!(output.contains("age?"));
}

#[test]
fn test_template_engine_render_zod_schema() {
    let engine = TemplateEngine::new(None).unwrap();
    let context = ZodContext::schema(
        "UserSchema".to_string(),
        "z.object({ name: z.string(), age: z.number().optional() })".to_string(),
        None,
    );
    let result = engine.render(TemplateId::ZodSchema, &context);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("UserSchema"));
    assert!(output.contains("z.object"));
}

#[test]
fn test_template_engine_render_zod_enum() {
    let engine = TemplateEngine::new(None).unwrap();
    let context = ZodContext::enum_schema(
        "StatusEnum".to_string(),
        vec!["active".to_string(), "inactive".to_string()],
    );
    let result = engine.render(TemplateId::ZodEnum, &context);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("StatusEnum"));
    assert!(output.contains("z.enum"));
    assert!(output.contains("\"active\""));
    assert!(output.contains("\"inactive\""));
}

#[test]
fn test_template_engine_render_api_client() {
    let engine = TemplateEngine::new(None).unwrap();
    let context = ApiContext::new(
        "getUser".to_string(),
        Some("getUser".to_string()),
        "get".to_string(),
        "/users/{id}".to_string(),
        vec![Parameter::new(
            "id".to_string(),
            "string".to_string(),
            false,
            Some("User ID".to_string()),
        )],
        vec![],
        None,
        vec![Response::new(200, "User".to_string())],
        "".to_string(),
        "../http".to_string(),
        ": Promise<User>".to_string(),
        "  return http.get<User>(url);".to_string(),
        "users".to_string(),
        "id: string".to_string(),
        "Get user by ID".to_string(),
    );
    let result = engine.render(TemplateId::ApiClientFetch, &context);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("export const getUser"));
    assert!(output.contains("async"));
    assert!(output.contains("http.get"));
}

#[test]
fn test_template_engine_render_type_alias() {
    let engine = TemplateEngine::new(None).unwrap();
    let context = TypeContext::alias("RecordType".to_string(), "Record<string, any>".to_string());
    let result = engine.render(TemplateId::TypeAlias, &context);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("export type RecordType"));
    assert!(output.contains("Record<string, any>"));
}

#[test]
fn test_template_registry_all() {
    let all = TemplateId::all();
    assert_eq!(all.len(), 6);
    assert!(all.contains(&TemplateId::TypeInterface));
    assert!(all.contains(&TemplateId::TypeEnum));
    assert!(all.contains(&TemplateId::TypeAlias));
    assert!(all.contains(&TemplateId::ZodSchema));
    assert!(all.contains(&TemplateId::ZodEnum));
    assert!(all.contains(&TemplateId::ApiClientFetch));
}

#[test]
fn test_template_registry_name() {
    assert_eq!(TemplateId::TypeInterface.name(), "type-interface");
    assert_eq!(TemplateId::ZodSchema.name(), "zod-schema");
    assert_eq!(TemplateId::ApiClientFetch.name(), "api-client-fetch");
}

#[test]
fn test_template_registry_filename() {
    assert_eq!(TemplateId::TypeInterface.filename(), "type-interface.tera");
    assert_eq!(TemplateId::ZodSchema.filename(), "zod-schema.tera");
}

#[test]
fn test_template_registry_from_str() {
    assert_eq!(
        TemplateId::from_str("type-interface").unwrap(),
        TemplateId::TypeInterface
    );
    assert_eq!(
        TemplateId::from_str("zod-schema").unwrap(),
        TemplateId::ZodSchema
    );
    assert!(TemplateId::from_str("unknown").is_err());
}
