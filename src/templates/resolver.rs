use crate::error::Result;
use crate::templates::loader::TemplateLoader;
use crate::templates::registry::TemplateId;
use std::path::Path;

/// Resolves templates with priority: user override > built-in.
pub struct TemplateResolver {
    project_root: Option<std::path::PathBuf>,
}

impl TemplateResolver {
    /// Create a new template resolver.
    pub fn new(project_root: Option<&Path>) -> Self {
        Self {
            project_root: project_root.map(|p| p.to_path_buf()),
        }
    }

    /// Resolve a template by ID, checking user override first, then built-in.
    pub fn resolve(&self, template_id: TemplateId) -> Result<String> {
        let template_name = template_id.name();

        // First check for user override
        if let Some(ref project_root) = self.project_root {
            if let Some(user_template) = TemplateLoader::load_user(template_name, project_root)? {
                return Ok(user_template);
            }
        }

        // Fallback to built-in template
        TemplateLoader::load_builtin(template_name)
    }

    /// Check if a template is overridden by user.
    pub fn is_overridden(&self, template_id: TemplateId) -> bool {
        if let Some(ref project_root) = self.project_root {
            let template_name = template_id.name();
            let user_template_dir = project_root.join(".vika").join("templates");
            let template_path = user_template_dir.join(format!("{}.tera", template_name));
            template_path.exists()
        } else {
            false
        }
    }

    /// Get list of all templates with their override status.
    pub fn list_templates(&self) -> Result<Vec<(String, bool)>> {
        let builtin = TemplateLoader::list_builtin();
        let user = if let Some(ref project_root) = self.project_root {
            TemplateLoader::list_user(project_root)?
        } else {
            Vec::new()
        };

        let mut result: Vec<(String, bool)> = builtin
            .iter()
            .map(|name| {
                let overridden = user.contains(name);
                (name.clone(), overridden)
            })
            .collect();

        // Add user-only templates (if any)
        for name in user {
            if !result.iter().any(|(n, _)| n == &name) {
                result.push((name, true));
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_builtin() {
        let resolver = TemplateResolver::new(None);
        let result = resolver.resolve(TemplateId::TypeInterface);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_overridden_false() {
        let temp_dir = TempDir::new().unwrap();
        let resolver = TemplateResolver::new(Some(temp_dir.path()));
        assert!(!resolver.is_overridden(TemplateId::TypeInterface));
    }

    #[test]
    fn test_list_templates() {
        let resolver = TemplateResolver::new(None);
        let templates = resolver.list_templates().unwrap();
        assert!(!templates.is_empty());
    }
}

