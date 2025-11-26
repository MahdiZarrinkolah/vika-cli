use crate::error::{FileSystemError, Result};
use crate::templates::engine::TemplateEngine;
use crate::templates::loader::TemplateLoader;
use colored::*;
use std::fs;

/// List all available templates.
pub fn list() -> Result<()> {
    let project_root = std::env::current_dir().ok();
    let engine = TemplateEngine::new(project_root.as_deref())?;

    let templates = engine.list_templates()?;

    println!("{}", "Built-in templates:".bright_cyan());
    println!();

    let mut builtin_list = Vec::new();
    let mut user_list = Vec::new();

    for (name, overridden) in templates {
        if overridden {
            user_list.push(name.clone());
            println!("  {} {} (overridden)", "✓".green(), name.bright_green());
        } else {
            builtin_list.push(name.clone());
            println!("  - {}", name);
        }
    }

    if !user_list.is_empty() {
        println!();
        println!("{}", "User overrides:".bright_yellow());
        for name in user_list {
            println!("  - {}", name.bright_green());
        }
    }

    Ok(())
}

/// Initialize templates directory by copying built-in templates.
pub fn init() -> Result<()> {
    let project_root = std::env::current_dir().map_err(|e| FileSystemError::ReadFileFailed {
        path: ".".to_string(),
        source: e,
    })?;

    let templates_dir = project_root.join(".vika").join("templates");

    // Create .vika directory if it doesn't exist
    if let Some(parent) = templates_dir.parent() {
        fs::create_dir_all(parent).map_err(|e| FileSystemError::CreateDirectoryFailed {
            path: parent.to_string_lossy().to_string(),
            source: e,
        })?;
    }

    // Create templates directory
    fs::create_dir_all(&templates_dir).map_err(|e| FileSystemError::CreateDirectoryFailed {
        path: templates_dir.to_string_lossy().to_string(),
        source: e,
    })?;

    // Get all built-in templates
    let builtin_templates = TemplateLoader::list_builtin();

    let mut copied = 0;
    let mut skipped = 0;

    for template_name in builtin_templates {
        let template_path = templates_dir.join(format!("{}.tera", template_name));

        // Skip if already exists
        if template_path.exists() {
            println!("  {} {} (already exists)", "⊘".yellow(), template_name);
            skipped += 1;
            continue;
        }

        // Load built-in template and write to user directory
        let content = TemplateLoader::load_builtin(&template_name)?;
        fs::write(&template_path, content).map_err(|e| FileSystemError::WriteFileFailed {
            path: template_path.to_string_lossy().to_string(),
            source: e,
        })?;

        println!("  {} {}", "✓".green(), template_name.bright_green());
        copied += 1;
    }

    println!();
    println!(
        "{}",
        format!(
            "✅ Initialized templates directory: {}",
            templates_dir.display()
        )
        .green()
    );
    println!(
        "{}",
        format!("   Copied: {}, Skipped: {}", copied, skipped).bright_black()
    );

    Ok(())
}
