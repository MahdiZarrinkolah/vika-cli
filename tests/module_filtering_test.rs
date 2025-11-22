use indexmap::IndexMap;
use openapiv3::{OpenAPI, Tag};
use vika_cli::generator::swagger_parser::extract_modules;

#[test]
fn test_filter_ignored_modules() {
    let mut openapi = OpenAPI::default();
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
    openapi.tags.push(Tag {
        name: "admin".to_string(),
        description: None,
        external_docs: None,
        extensions: IndexMap::new(),
    });

    let modules = extract_modules(&openapi);
    assert_eq!(modules.len(), 3);

    // Simulate filtering
    let ignored = vec!["admin".to_string()];
    let filtered: Vec<String> = modules
        .iter()
        .filter(|m| !ignored.contains(m))
        .cloned()
        .collect();

    assert_eq!(filtered.len(), 2);
    assert!(!filtered.contains(&"admin".to_string()));
    assert!(filtered.contains(&"users".to_string()));
    assert!(filtered.contains(&"products".to_string()));
}

#[test]
fn test_empty_module_list() {
    let openapi = OpenAPI::default();
    let modules = extract_modules(&openapi);

    // If no tags and no operations, modules should be empty
    // (or contain "default" if there are operations without tags)
    assert!(modules.is_empty() || modules.contains(&"default".to_string()));
}

#[test]
fn test_all_modules_ignored() {
    let mut openapi = OpenAPI::default();
    openapi.tags.push(Tag {
        name: "users".to_string(),
        description: None,
        external_docs: None,
        extensions: IndexMap::new(),
    });

    let modules = extract_modules(&openapi);
    let ignored = vec!["users".to_string()];
    let filtered: Vec<String> = modules
        .iter()
        .filter(|m| !ignored.contains(m))
        .cloned()
        .collect();

    assert!(filtered.is_empty());
}
