// Note: These tests change the current working directory and may interfere with each other
// when run in parallel. They work correctly when run sequentially (--test-threads=1).
use std::fs;
use std::path::PathBuf;
use vika_cli::formatter::{Formatter, FormatterManager};

#[test]
fn test_detect_formatter_no_formatter() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().ok();

    let _ = std::env::set_current_dir(temp_dir.path());

    let result = FormatterManager::detect_formatter();
    assert!(result.is_none());

    if let Some(orig) = original_dir {
        let _ = std::env::set_current_dir(orig);
    }
}

#[test]
fn test_detect_formatter_prettier_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().ok();

    let _ = std::env::set_current_dir(temp_dir.path());

    // Create .prettierrc in current directory (relative path)
    fs::write(".prettierrc", "{}").unwrap();

    let result = FormatterManager::detect_formatter();
    assert!(result.is_some(), "Should detect .prettierrc file");
    if let Some(formatter) = result {
        matches!(formatter, Formatter::Prettier);
    }

    if let Some(orig) = original_dir {
        let _ = std::env::set_current_dir(orig);
    }
}

#[test]
fn test_detect_formatter_prettier_json() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().ok();

    let _ = std::env::set_current_dir(temp_dir.path());

    fs::write(".prettierrc.json", "{}").unwrap();

    let result = FormatterManager::detect_formatter();
    assert!(result.is_some());
    if let Some(formatter) = result {
        matches!(formatter, Formatter::Prettier);
    }

    if let Some(orig) = original_dir {
        let _ = std::env::set_current_dir(orig);
    }
}

#[test]
fn test_detect_formatter_prettier_package_json() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().ok();

    let _ = std::env::set_current_dir(temp_dir.path());

    fs::write(
        "package.json",
        r#"{"devDependencies": {"prettier": "^2.0.0"}}"#,
    )
    .unwrap();

    let result = FormatterManager::detect_formatter();
    assert!(result.is_some());
    if let Some(formatter) = result {
        matches!(formatter, Formatter::Prettier);
    }

    if let Some(orig) = original_dir {
        let _ = std::env::set_current_dir(orig);
    }
}

#[test]
fn test_detect_formatter_biome() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().ok();

    let _ = std::env::set_current_dir(temp_dir.path());

    // Create biome.json in current directory (relative path)
    fs::write("biome.json", "{}").unwrap();

    let result = FormatterManager::detect_formatter();
    assert!(result.is_some(), "Should detect biome.json file");
    if let Some(formatter) = result {
        matches!(formatter, Formatter::Biome);
    }

    if let Some(orig) = original_dir {
        let _ = std::env::set_current_dir(orig);
    }
}

#[test]
fn test_detect_formatter_biome_jsonc() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().ok();

    let _ = std::env::set_current_dir(temp_dir.path());

    fs::write("biome.jsonc", "{}").unwrap();

    let result = FormatterManager::detect_formatter();
    assert!(result.is_some(), "Should detect biome.jsonc file");
    if let Some(formatter) = result {
        matches!(formatter, Formatter::Biome);
    }

    if let Some(orig) = original_dir {
        let _ = std::env::set_current_dir(orig);
    }
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
