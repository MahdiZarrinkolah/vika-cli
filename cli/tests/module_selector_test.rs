use vika_cli::error::GenerationError;
use vika_cli::generator::module_selector::select_modules;

#[test]
#[ignore] // Requires interactive input - cannot be tested reliably in non-interactive environment
fn test_select_modules_filters_ignored() {
    let available = vec![
        "users".to_string(),
        "products".to_string(),
        "orders".to_string(),
    ];
    let ignored = vec!["orders".to_string()];

    // This test requires interactive input via dialoguer::MultiSelect
    // In non-interactive test environments, dialoguer may behave unpredictably
    // The filtering logic is tested indirectly through other tests
    let result = select_modules(&available, &ignored);
    // Result may be Ok or Err depending on test environment - test is ignored
    let _ = result;
}

#[test]
fn test_select_modules_no_modules_available() {
    let available: Vec<String> = vec![];
    let ignored: Vec<String> = vec![];

    let result = select_modules(&available, &ignored);
    assert!(result.is_err());
    match result.unwrap_err() {
        vika_cli::error::VikaError::Generation(GenerationError::NoModulesAvailable) => {}
        _ => panic!("Expected NoModulesAvailable error"),
    }
}

#[test]
fn test_select_modules_all_ignored() {
    let available = vec!["users".to_string(), "products".to_string()];
    let ignored = vec!["users".to_string(), "products".to_string()];

    let result = select_modules(&available, &ignored);
    assert!(result.is_err());
    match result.unwrap_err() {
        vika_cli::error::VikaError::Generation(GenerationError::NoModulesAvailable) => {}
        _ => panic!("Expected NoModulesAvailable error"),
    }
}
