use crate::error::{Result, TemplateError};
use crate::templates::registry::TemplateId;
use crate::templates::resolver::TemplateResolver;
use serde::Serialize;
use std::path::Path;
use tera::{Context, Tera};

/// Template engine wrapper around Tera.
pub struct TemplateEngine {
    tera: Tera,
    resolver: TemplateResolver,
}

impl TemplateEngine {
    /// Create a new template engine.
    ///
    /// Loads all templates (built-in and user overrides) and compiles them.
    pub fn new(project_root: Option<&Path>) -> Result<Self> {
        let resolver = TemplateResolver::new(project_root);
        let mut tera = Tera::default();

        // Load all templates into Tera
        for template_id in TemplateId::all() {
            let template_content = resolver.resolve(template_id)?;
            let template_name = template_id.filename();

            tera.add_raw_template(&template_name, &template_content)
                .map_err(|e| {
                    crate::error::GenerationError::Template(TemplateError::InvalidSyntax {
                        name: template_name.to_string(),
                        message: e.to_string(),
                    })
                })?;
        }

        Ok(Self { tera, resolver })
    }

    /// Render a template with the given context.
    pub fn render<T: Serialize>(
        &self,
        template_id: TemplateId,
        context: &T,
    ) -> Result<String> {
        let template_name = template_id.filename();

        let json_value = serde_json::to_value(context)
            .map_err(|e| {
                crate::error::GenerationError::Template(TemplateError::RenderFailed {
                    name: template_name.to_string(),
                    message: format!("Failed to serialize context: {}", e),
                })
            })?;
        let tera_context = Context::from_serialize(&json_value)
            .map_err(|e| {
                crate::error::GenerationError::Template(TemplateError::RenderFailed {
                    name: template_name.to_string(),
                    message: format!("Failed to create Tera context: {}", e),
                })
            })?;

        self.tera
            .render(&template_name, &tera_context)
            .map_err(|e| {
                crate::error::GenerationError::Template(TemplateError::RenderFailed {
                    name: template_name.to_string(),
                    message: e.to_string(),
                })
            })
            .map_err(Into::into)
    }

    /// Check if a template is overridden by user.
    pub fn is_overridden(&self, template_id: TemplateId) -> bool {
        self.resolver.is_overridden(template_id)
    }

    /// List all templates with their override status.
    pub fn list_templates(&self) -> Result<Vec<(String, bool)>> {
        self.resolver.list_templates()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::context::TypeContext;

    #[test]
    fn test_template_engine_new() {
        let engine = TemplateEngine::new(None);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_template_engine_render_enum() {
        let engine = TemplateEngine::new(None).unwrap();
        let context = TypeContext::enum_type("TestEnum".to_string(), vec!["A".to_string(), "B".to_string()]);
        let result = engine.render(TemplateId::TypeEnum, &context);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("TestEnum"));
        assert!(output.contains("A"));
        assert!(output.contains("B"));
    }
}

