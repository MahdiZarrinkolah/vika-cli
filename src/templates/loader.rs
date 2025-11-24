use crate::error::{FileSystemError, Result};
use rust_embed::RustEmbed;
use std::path::Path;

/// Embedded built-in templates.
#[derive(RustEmbed)]
#[folder = "builtin/templates"]
#[include = "*.tera"]
struct BuiltinTemplates;

/// Loads templates from embedded built-in templates or user directory.
pub struct TemplateLoader;

impl TemplateLoader {
    /// Load a template from embedded built-in templates.
    pub fn load_builtin(template_name: &str) -> Result<String> {
        let filename = format!("{}.tera", template_name);
        BuiltinTemplates::get(&filename)
            .map(|file| {
                String::from_utf8(file.data.to_vec()).map_err(|e| FileSystemError::ReadFileFailed {
                    path: filename.clone(),
                    source: std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Invalid UTF-8 in template: {}", e),
                    ),
                })
            })
            .transpose()?
            .ok_or_else(|| FileSystemError::FileNotFound {
                path: format!("builtin/templates/{}", filename),
            })
            .map_err(Into::into)
    }

    /// Load a template from user directory (`.vika/templates/`).
    pub fn load_user(template_name: &str, project_root: &Path) -> Result<Option<String>> {
        let user_template_dir = project_root.join(".vika").join("templates");
        let template_path = user_template_dir.join(format!("{}.tera", template_name));

        if !template_path.exists() {
            return Ok(None);
        }

        std::fs::read_to_string(&template_path)
            .map(Some)
            .map_err(|e| FileSystemError::ReadFileFailed {
                path: template_path.to_string_lossy().to_string(),
                source: e,
            })
            .map_err(Into::into)
    }

    /// List all available built-in templates.
    pub fn list_builtin() -> Vec<String> {
        BuiltinTemplates::iter()
            .filter_map(|path| path.strip_suffix(".tera").map(|s| s.to_string()))
            .collect()
    }

    /// List all user templates in the project directory.
    pub fn list_user(project_root: &Path) -> Result<Vec<String>> {
        let user_template_dir = project_root.join(".vika").join("templates");

        if !user_template_dir.exists() {
            return Ok(Vec::new());
        }

        let mut templates = Vec::new();
        let entries =
            std::fs::read_dir(&user_template_dir).map_err(|e| FileSystemError::ReadFileFailed {
                path: user_template_dir.to_string_lossy().to_string(),
                source: e,
            })?;

        for entry in entries {
            let entry = entry.map_err(|e| FileSystemError::ReadFileFailed {
                path: user_template_dir.to_string_lossy().to_string(),
                source: e,
            })?;

            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "tera" {
                        if let Some(stem) = path.file_stem() {
                            if let Some(name) = stem.to_str() {
                                templates.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(templates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_list_builtin() {
        let templates = TemplateLoader::list_builtin();
        // Should list all embedded templates
        assert!(!templates.is_empty());
    }

    #[test]
    fn test_list_user_empty() {
        let temp_dir = TempDir::new().unwrap();
        let templates = TemplateLoader::list_user(temp_dir.path()).unwrap();
        assert!(templates.is_empty());
    }

    #[test]
    fn test_load_user_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let result = TemplateLoader::load_user("nonexistent", temp_dir.path()).unwrap();
        assert!(result.is_none());
    }
}
