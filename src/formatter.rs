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
            .filter_map(|p| p.to_str().filter(|s| !s.is_empty()).map(|s| s.to_string()))
            .collect();

        if file_paths.is_empty() {
            return Ok(());
        }

        // Always use explicit file paths to avoid glob pattern issues
        // Modern systems can handle long command lines, and if not, we'll handle it gracefully
        let output = Command::new("npx")
            .arg("prettier")
            .arg("--write")
            .args(&file_paths)
            .output();

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
            .filter_map(|p| p.to_str().filter(|s| !s.is_empty()).map(|s| s.to_string()))
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

    /// Format a single content string using the specified formatter
    /// Returns the formatted content, or original content if formatting fails
    /// Uses stdin/stdout to format, ensuring formatter config is found
    pub fn format_content(content: &str, formatter: Formatter, file_path: &Path) -> Result<String> {
        use std::process::Stdio;

        // Determine the working directory (where config files are likely located)
        let work_dir = file_path.parent().unwrap_or(Path::new("."));

        // Use stdin/stdout for formatting to ensure config files are found
        let format_result = match formatter {
            Formatter::Prettier => {
                let mut cmd = Command::new("npx");
                cmd.arg("prettier")
                    .arg("--stdin-filepath")
                    .arg(file_path.to_str().unwrap_or("file.ts"))
                    .current_dir(work_dir)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());

                let mut child = cmd.spawn().map_err(|e| {
                    crate::error::VikaError::from(crate::error::FileSystemError::ReadFileFailed {
                        path: file_path.display().to_string(),
                        source: e,
                    })
                })?;

                // Write content to stdin
                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    stdin.write_all(content.as_bytes()).map_err(|e| {
                        crate::error::VikaError::from(
                            crate::error::FileSystemError::WriteFileFailed {
                                path: "stdin".to_string(),
                                source: e,
                            },
                        )
                    })?;
                }

                child.wait_with_output()
            }
            Formatter::Biome => {
                let mut cmd = Command::new("npx");
                cmd.arg("@biomejs/biome")
                    .arg("format")
                    .arg("--stdin-file-path")
                    .arg(file_path.to_str().unwrap_or("file.ts"))
                    .current_dir(work_dir)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());

                let mut child = cmd.spawn().map_err(|e| {
                    crate::error::VikaError::from(crate::error::FileSystemError::ReadFileFailed {
                        path: file_path.display().to_string(),
                        source: e,
                    })
                })?;

                // Write content to stdin
                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    stdin.write_all(content.as_bytes()).map_err(|e| {
                        crate::error::VikaError::from(
                            crate::error::FileSystemError::WriteFileFailed {
                                path: "stdin".to_string(),
                                source: e,
                            },
                        )
                    })?;
                }

                child.wait_with_output()
            }
        };

        // Read the formatted content from stdout
        match format_result {
            Ok(output) if output.status.success() => {
                String::from_utf8(output.stdout).map_err(|e| {
                    crate::error::VikaError::from(crate::error::FileSystemError::ReadFileFailed {
                        path: "stdout".to_string(),
                        source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
                    })
                })
            }
            _ => {
                // Formatting failed, return original content
                Ok(content.to_string())
            }
        }
    }
}
