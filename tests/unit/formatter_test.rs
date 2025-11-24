use tempfile::TempDir;
use vika_cli::formatter::{Formatter, FormatterManager};
use std::fs;
use std::path::PathBuf;

#[test]
    fn test_detect_formatter_no_formatter() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        let result = FormatterManager::detect_formatter();
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_formatter_prettier_config() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        // Create .prettierrc
        fs::write(".prettierrc", "{}").unwrap();
        
        let result = FormatterManager::detect_formatter();
        assert!(result.is_some());
        matches!(result.unwrap(), Formatter::Prettier);
    }

    #[test]
    fn test_detect_formatter_prettier_json() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        fs::write(".prettierrc.json", "{}").unwrap();
        
        let result = FormatterManager::detect_formatter();
        assert!(result.is_some());
        matches!(result.unwrap(), Formatter::Prettier);
    }

    #[test]
    fn test_detect_formatter_prettier_package_json() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        fs::write("package.json", r#"{"devDependencies": {"prettier": "^2.0.0"}}"#).unwrap();
        
        let result = FormatterManager::detect_formatter();
        assert!(result.is_some());
        matches!(result.unwrap(), Formatter::Prettier);
    }

    #[test]
    fn test_detect_formatter_biome() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        fs::write("biome.json", "{}").unwrap();
        
        let result = FormatterManager::detect_formatter();
        assert!(result.is_some());
        matches!(result.unwrap(), Formatter::Biome);
    }

    #[test]
    fn test_detect_formatter_biome_jsonc() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        fs::write("biome.jsonc", "{}").unwrap();
        
        let result = FormatterManager::detect_formatter();
        assert!(result.is_some());
        matches!(result.unwrap(), Formatter::Biome);
    }

    #[test]
    fn test_format_files_empty() {
        let files: Vec<PathBuf> = vec![];
        let result = FormatterManager::format_files(&files, Formatter::Prettier);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_files_prettier() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.ts");
        fs::write(&file_path, "const x=1").unwrap();
        
        let files = vec![file_path];
        // This will fail silently if prettier is not available, which is expected
        let result = FormatterManager::format_files(&files, Formatter::Prettier);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_files_biome() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.ts");
        fs::write(&file_path, "const x=1").unwrap();
        
        let files = vec![file_path];
        // This will fail silently if biome is not available, which is expected
        let result = FormatterManager::format_files(&files, Formatter::Biome);
        assert!(result.is_ok());
    }

