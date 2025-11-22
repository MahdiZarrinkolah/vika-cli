use vika_cli::generator::swagger_parser::filter_common_schemas;
use std::collections::HashMap;

#[test]
fn test_detect_common_schemas() {
    let mut module_schemas = HashMap::new();
    module_schemas.insert("users".to_string(), vec!["User".to_string(), "CommonResponse".to_string()]);
    module_schemas.insert("products".to_string(), vec!["Product".to_string(), "CommonResponse".to_string()]);
    
    let selected = vec!["users".to_string(), "products".to_string()];
    let (filtered, common_schemas) = filter_common_schemas(&module_schemas, &selected);
    
    // CommonResponse should be in common schemas
    assert!(common_schemas.contains(&"CommonResponse".to_string()));
    
    // CommonResponse should be removed from individual modules
    assert!(!filtered.get("users").unwrap().contains(&"CommonResponse".to_string()));
    assert!(!filtered.get("products").unwrap().contains(&"CommonResponse".to_string()));
    
    // Other schemas should remain
    assert!(filtered.get("users").unwrap().contains(&"User".to_string()));
    assert!(filtered.get("products").unwrap().contains(&"Product".to_string()));
}

#[test]
fn test_no_common_schemas_single_module() {
    let mut module_schemas = HashMap::new();
    module_schemas.insert("users".to_string(), vec!["User".to_string()]);
    
    let selected = vec!["users".to_string()];
    let (_, common_schemas) = filter_common_schemas(&module_schemas, &selected);
    
    assert!(common_schemas.is_empty());
}

#[test]
fn test_no_common_schemas_different_schemas() {
    let mut module_schemas = HashMap::new();
    module_schemas.insert("users".to_string(), vec!["User".to_string()]);
    module_schemas.insert("products".to_string(), vec!["Product".to_string()]);
    
    let selected = vec!["users".to_string(), "products".to_string()];
    let (_, common_schemas) = filter_common_schemas(&module_schemas, &selected);
    
    assert!(common_schemas.is_empty());
}

#[test]
fn test_common_schemas_three_modules() {
    let mut module_schemas = HashMap::new();
    module_schemas.insert("users".to_string(), vec!["User".to_string(), "Common".to_string()]);
    module_schemas.insert("products".to_string(), vec!["Product".to_string(), "Common".to_string()]);
    module_schemas.insert("orders".to_string(), vec!["Order".to_string(), "Common".to_string()]);
    
    let selected = vec!["users".to_string(), "products".to_string(), "orders".to_string()];
    let (filtered, common_schemas) = filter_common_schemas(&module_schemas, &selected);
    
    assert!(common_schemas.contains(&"Common".to_string()));
    
    // Common should be removed from all modules
    for module in &selected {
        assert!(!filtered.get(module).unwrap().contains(&"Common".to_string()));
    }
}

