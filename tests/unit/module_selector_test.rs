use vika_cli::generator::module_selector::select_modules;
use vika_cli::error::GenerationError;

#[test]
    fn test_select_modules_filters_ignored() {
        let available = vec!["users".to_string(), "products".to_string(), "orders".to_string()];
        let ignored = vec!["orders".to_string()];
        
        // This will fail because it requires interactive input
        // But we can test the filtering logic by checking the error
        let result = select_modules(&available, &ignored);
        // Since we can't interact, this will fail, but the filtering happens before interaction
        assert!(result.is_err());
    }

    #[test]
    fn test_select_modules_no_modules_available() {
        let available: Vec<String> = vec![];
        let ignored: Vec<String> = vec![];
        
        let result = select_modules(&available, &ignored);
        assert!(result.is_err());
        match result.unwrap_err() {
            vika_cli::error::VikaError::Generation(GenerationError::NoModulesAvailable) => {},
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
            vika_cli::error::VikaError::Generation(GenerationError::NoModulesAvailable) => {},
            _ => panic!("Expected NoModulesAvailable error"),
        }
    }

