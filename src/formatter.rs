use crate::error::Result;
use std::path::Path;
use std::process::Command;

pub enum Formatter {
    Prettier,
    Biome,
}

pub struct FormatterManager;

impl FormatterManager {
    pub fn detect_formatter() -> Option<Formatter> {
        // Check for prettier
        if Self::has_prettier() {
            return Some(Formatter::Prettier);
        }

        // Check for biome
        if Self::has_biome() {
            return Some(Formatter::Biome);
        }

        None
    }

    fn has_prettier() -> bool {
        // Check for package.json with prettier
        if Path::new("package.json").exists() {
            if let Ok(content) = std::fs::read_to_string("package.json") {
                if content.contains("prettier") {
                    return true;
                }
            }
        }

        // Check for prettier config files
        Path::new(".prettierrc").exists()
            || Path::new(".prettierrc.json").exists()
            || Path::new(".prettierrc.js").exists()
            || Path::new("prettier.config.js").exists()
    }

    fn has_biome() -> bool {
        // Check for biome.json
        Path::new("biome.json").exists() || Path::new("biome.jsonc").exists()
    }

    pub fn format_files(files: &[std::path::PathBuf], formatter: Formatter) -> Result<()> {
        match formatter {
            Formatter::Prettier => Self::format_with_prettier(files),
            Formatter::Biome => Self::format_with_biome(files),
        }
    }

    fn format_with_prettier(files: &[std::path::PathBuf]) -> Result<()> {
        let file_paths: Vec<String> = files
            .iter()
            .filter_map(|p| p.to_str().map(|s| s.to_string()))
            .collect();

        if file_paths.is_empty() {
            return Ok(());
        }

        let output = Command::new("npx")
            .arg("prettier")
            .arg("--write")
            .args(&file_paths)
            .output();

        match output {
            Ok(_) => Ok(()),
            Err(e) => {
                // Silently fail if prettier is not available
                eprintln!("Warning: Failed to run prettier: {}", e);
                Ok(())
            }
        }
    }

    fn format_with_biome(files: &[std::path::PathBuf]) -> Result<()> {
        let file_paths: Vec<String> = files
            .iter()
            .filter_map(|p| p.to_str().map(|s| s.to_string()))
            .collect();

        if file_paths.is_empty() {
            return Ok(());
        }

        let output = Command::new("npx")
            .arg("@biomejs/biome")
            .arg("format")
            .arg("--write")
            .args(&file_paths)
            .output();

        match output {
            Ok(_) => Ok(()),
            Err(e) => {
                // Silently fail if biome is not available
                eprintln!("Warning: Failed to run biome: {}", e);
                Ok(())
            }
        }
    }
}
