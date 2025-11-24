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
        Self::detect_formatter_from_dir(Path::new("."))
    }

    pub fn detect_formatter_from_dir(dir: &Path) -> Option<Formatter> {
        // Check for prettier
        if Self::has_prettier_in_dir(dir) {
            return Some(Formatter::Prettier);
        }

        // Check for biome
        if Self::has_biome_in_dir(dir) {
            return Some(Formatter::Biome);
        }

        None
    }

    fn has_prettier_in_dir(dir: &Path) -> bool {
        // Check for package.json with prettier
        let package_json = dir.join("package.json");
        if package_json.exists() {
            if let Ok(content) = std::fs::read_to_string(&package_json) {
                if content.contains("prettier") {
                    return true;
                }
            }
        }

        // Check for prettier config files
        dir.join(".prettierrc").exists()
            || dir.join(".prettierrc.json").exists()
            || dir.join(".prettierrc.js").exists()
            || dir.join("prettier.config.js").exists()
    }

    fn has_biome_in_dir(dir: &Path) -> bool {
        // Check for biome.json
        dir.join("biome.json").exists() || dir.join("biome.jsonc").exists()
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

        // Use glob pattern if we have many files to avoid command line length issues
        let output = if file_paths.len() > 50 {
            // Use glob pattern instead
            Command::new("npx")
                .arg("prettier")
                .arg("--write")
                .arg("src/**/*.ts")
                .output()
        } else {
            Command::new("npx")
                .arg("prettier")
                .arg("--write")
                .args(&file_paths)
                .output()
        };

        match output {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    eprintln!("Warning: Prettier exited with error:");
                    eprintln!("  stderr: {}", stderr);
                    if !stdout.is_empty() {
                        eprintln!("  stdout: {}", stdout);
                    }
                }
                Ok(())
            }
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
