use crate::config::loader::load_config;
use crate::config::validator::validate_config;
use crate::error::{FileSystemError, Result};
use crate::formatter::FormatterManager;
use crate::generator::swagger_parser::filter_common_schemas;
use crate::generator::writer::{write_api_client_with_options, write_schemas_with_options};
use crate::specs::manager::{is_multi_spec_mode, resolve_spec_selection};
use crate::specs::runner::{run_all_specs, GenerateOptions};
use colored::*;
use std::path::{Path, PathBuf};

pub async fn run(spec_name: Option<String>, all_specs: bool) -> Result<()> {
    println!("{}", "🔄 Updating generated code...".bright_cyan());
    println!();

    // Load config
    let config = load_config()?;
    validate_config(&config)?;

    use crate::error::{FileSystemError, GenerationError};

    // Check if multi-spec mode
    let is_multi_spec = is_multi_spec_mode(&config);

    if is_multi_spec {
        // Resolve which specs to update
        let specs_to_update = resolve_spec_selection(&config, spec_name.clone(), all_specs)?;

        let options = GenerateOptions {
            use_cache: config.generation.enable_cache,
            use_backup: config.generation.enable_backup,
            use_force: config.generation.conflict_strategy == "force",
            verbose: false,
        };

        if specs_to_update.len() > 1 {
            // Update all selected specs
            println!("{}", "🔄 Updating all selected specs...".bright_cyan());
            println!();
            run_all_specs(&specs_to_update, &config, &options).await?;
        } else {
            // Update single spec
            let spec_entry = &specs_to_update[0];
            println!(
                "{}",
                format!("🔄 Updating spec: {}", spec_entry.name).bright_cyan()
            );
            println!();
            update_single_spec(spec_entry, &config).await?;
        }
        return Ok(());
    }

    // Single-spec mode (backward compatible)
    let spec_path = config.spec_path.ok_or(GenerationError::SpecPathRequired)?;

    // Check if spec path is a temporary file that might not exist anymore
    if !spec_path.starts_with("http://")
        && !spec_path.starts_with("https://")
        && !std::path::Path::new(&spec_path).exists()
    {
        return Err(FileSystemError::FileNotFound {
            path: format!(
                "{}\n  The spec file no longer exists. Please run 'vika-cli generate --spec <path-or-url>' again with a valid spec path.",
                spec_path
            ),
        }
        .into());
    }

    // Get selected modules from config
    let selected_modules = if config.modules.selected.is_empty() {
        return Err(GenerationError::NoModulesSelected.into());
    } else {
        config.modules.selected.clone()
    };

    println!(
        "{}",
        format!("📥 Fetching spec from: {}", spec_path).bright_blue()
    );
    // Use caching for update command (same as generate)
    let use_cache = config.generation.enable_cache;
    let parsed =
        crate::generator::swagger_parser::fetch_and_parse_spec_with_cache(&spec_path, use_cache)
            .await?;
    println!(
        "{}",
        format!("✅ Parsed spec with {} modules", parsed.modules.len()).green()
    );
    println!();
    println!(
        "{}",
        format!(
            "📦 Updating {} module(s): {}",
            selected_modules.len(),
            selected_modules.join(", ")
        )
        .bright_green()
    );
    println!();

    // Filter common schemas based on selected modules only
    let (filtered_module_schemas, common_schemas) =
        filter_common_schemas(&parsed.module_schemas, &selected_modules);

    // Initialize template engine once for all modules
    let project_root = std::env::current_dir().ok();
    let template_engine = crate::templates::engine::TemplateEngine::new(project_root.as_deref())?;

    // Generate code for each module
    let schemas_dir = PathBuf::from(&config.schemas.output);
    let apis_dir = PathBuf::from(&config.apis.output);

    let mut total_files = 0;
    let mut module_summary: Vec<(String, usize)> = Vec::new();

    // Get force and backup settings from config
    let use_force = config.generation.conflict_strategy == "force";
    let use_backup = config.generation.enable_backup;

    // Generate common module first if there are shared schemas
    if !common_schemas.is_empty() {
        println!("{}", "🔨 Regenerating common schemas...".bright_cyan());

        // Shared enum registry to ensure consistent naming between TypeScript and Zod
        let mut shared_enum_registry = std::collections::HashMap::new();

        // Generate TypeScript typings for common schemas
        let common_types = crate::generator::ts_typings::generate_typings_with_registry_and_engine(
            &parsed.openapi,
            &parsed.schemas,
            &common_schemas,
            &mut shared_enum_registry,
            &common_schemas,
            Some(&template_engine),
        )?;

        // Generate Zod schemas for common schemas (using same registry)
        let common_zod_schemas =
            crate::generator::zod_schema::generate_zod_schemas_with_registry_and_engine(
                &parsed.openapi,
                &parsed.schemas,
                &common_schemas,
                &mut shared_enum_registry,
                &common_schemas,
                Some(&template_engine),
            )?;

        // Write common schemas (use force if config says so)
        let common_files = write_schemas_with_options(
            &schemas_dir,
            "common",
            &common_types,
            &common_zod_schemas,
            None, // spec_name is None for single-spec mode
            use_backup,
            use_force,
        )?;
        total_files += common_files.len();
        module_summary.push(("common".to_string(), common_files.len()));
    }

    for module in &selected_modules {
        println!(
            "{}",
            format!("🔨 Regenerating code for module: {}", module).bright_cyan()
        );

        // Get operations for this module
        let operations = parsed
            .operations_by_tag
            .get(module)
            .cloned()
            .unwrap_or_default();

        if operations.is_empty() {
            println!(
                "{}",
                format!("⚠️  No operations found for module: {}", module).yellow()
            );
            continue;
        }

        // Get schema names used by this module (from filtered schemas)
        let module_schema_names = filtered_module_schemas
            .get(module)
            .cloned()
            .unwrap_or_default();

        // Shared enum registry to ensure consistent naming between TypeScript and Zod
        let mut shared_enum_registry = std::collections::HashMap::new();

        // Generate TypeScript typings
        let types = if !module_schema_names.is_empty() {
            crate::generator::ts_typings::generate_typings_with_registry_and_engine(
                &parsed.openapi,
                &parsed.schemas,
                &module_schema_names,
                &mut shared_enum_registry,
                &common_schemas,
                Some(&template_engine),
            )?
        } else {
            Vec::new()
        };

        // Generate Zod schemas (using same registry)
        let zod_schemas = if !module_schema_names.is_empty() {
            crate::generator::zod_schema::generate_zod_schemas_with_registry_and_engine(
                &parsed.openapi,
                &parsed.schemas,
                &module_schema_names,
                &mut shared_enum_registry,
                &common_schemas,
                Some(&template_engine),
            )?
        } else {
            Vec::new()
        };

        // Generate API client (using same enum registry as schemas)
        let api_result =
            crate::generator::api_client::generate_api_client_with_registry_and_engine(
                &parsed.openapi,
                &operations,
                module,
                &common_schemas,
                &mut shared_enum_registry,
                Some(&template_engine),
            )?;

        // Combine response types with schema types
        let mut all_types = types;
        all_types.extend(api_result.response_types);

        // Write schemas (use force if config says so)
        let schema_files = write_schemas_with_options(
            &schemas_dir,
            module,
            &all_types,
            &zod_schemas,
            None, // spec_name is None for single-spec mode
            use_backup,
            use_force,
        )?;
        total_files += schema_files.len();

        // Write API client (use force if config says so)
        let api_files = write_api_client_with_options(
            &apis_dir,
            module,
            &api_result.functions,
            None, // spec_name is None for single-spec mode
            use_backup,
            use_force,
        )?;
        total_files += api_files.len();

        let module_file_count = schema_files.len() + api_files.len();
        module_summary.push((module.clone(), module_file_count));
        println!(
            "{}",
            format!(
                "✅ Regenerated {} files for module: {}",
                module_file_count, module
            )
            .green()
        );
    }

    println!();
    println!(
        "{}",
        format!("✨ Successfully updated {} files!", total_files).bright_green()
    );
    println!();
    println!("{}", "Updated files:".bright_cyan());
    println!("  📁 Schemas: {}", config.schemas.output);
    println!("  📁 APIs: {}", config.apis.output);
    println!();
    if !module_summary.is_empty() {
        println!("{}", "Module breakdown:".bright_cyan());
        for (module, count) in &module_summary {
            println!("  • {}: {} files", module, count);
        }
    }

    // Format all generated files with prettier/biome if available
    let mut all_generated_files = Vec::new();

    // Collect schema files recursively
    if schemas_dir.exists() {
        collect_ts_files(&schemas_dir, &mut all_generated_files)?;
    }

    // Collect API files recursively
    if apis_dir.exists() {
        collect_ts_files(&apis_dir, &mut all_generated_files)?;
    }

    // Format files if formatter is available
    if !all_generated_files.is_empty() {
        // Find the common parent directory (where config files are likely located)
        let output_base = schemas_dir
            .parent()
            .and_then(|p| p.parent())
            .or_else(|| apis_dir.parent().and_then(|p| p.parent()));

        let formatter = if let Some(base_dir) = output_base {
            FormatterManager::detect_formatter_from_dir(base_dir)
                .or_else(FormatterManager::detect_formatter)
        } else {
            FormatterManager::detect_formatter()
        };

        if let Some(formatter) = formatter {
            println!("{}", "Formatting generated files...".bright_cyan());
            let original_dir =
                std::env::current_dir().map_err(|e| FileSystemError::ReadFileFailed {
                    path: ".".to_string(),
                    source: e,
                })?;

            if let Some(output_base) = output_base {
                std::env::set_current_dir(output_base).map_err(|e| {
                    FileSystemError::ReadFileFailed {
                        path: output_base.display().to_string(),
                        source: e,
                    }
                })?;

                let relative_files: Vec<PathBuf> = all_generated_files
                    .iter()
                    .filter_map(|p| p.strip_prefix(output_base).ok().map(|p| p.to_path_buf()))
                    .collect();

                if !relative_files.is_empty() {
                    let result = FormatterManager::format_files(&relative_files, formatter);
                    std::env::set_current_dir(&original_dir).map_err(|e| {
                        FileSystemError::ReadFileFailed {
                            path: original_dir.display().to_string(),
                            source: e,
                        }
                    })?;
                    result?;
                } else {
                    std::env::set_current_dir(&original_dir).map_err(|e| {
                        FileSystemError::ReadFileFailed {
                            path: original_dir.display().to_string(),
                            source: e,
                        }
                    })?;
                }
            } else {
                FormatterManager::format_files(&all_generated_files, formatter)?;
            }
            println!("{}", "✅ Files formatted".green());
        }
    }

    Ok(())
}

async fn update_single_spec(
    spec_entry: &crate::config::model::SpecEntry,
    config: &crate::config::model::Config,
) -> Result<()> {
    use crate::generator::swagger_parser::fetch_and_parse_spec_with_cache;
    use crate::generator::ts_typings::generate_typings_with_registry_and_engine_and_spec;
    use crate::generator::zod_schema::generate_zod_schemas_with_registry_and_engine_and_spec;
    use crate::generator::api_client::generate_api_client_with_registry_and_engine_and_spec;
    use crate::templates::engine::TemplateEngine;

    // Check if spec path exists
    if !spec_entry.path.starts_with("http://")
        && !spec_entry.path.starts_with("https://")
        && !std::path::Path::new(&spec_entry.path).exists()
    {
        return Err(FileSystemError::FileNotFound {
            path: format!(
                "{}\n  The spec file no longer exists. Please run 'vika-cli generate --spec {}' again with a valid spec path.",
                spec_entry.path, spec_entry.name
            ),
        }
        .into());
    }

    // Get selected modules from config
    let selected_modules = if config.modules.selected.is_empty() {
        return Err(crate::error::GenerationError::NoModulesSelected.into());
    } else {
        config.modules.selected.clone()
    };

    println!(
        "{}",
        format!("📥 Fetching spec from: {}", spec_entry.path).bright_blue()
    );
    let use_cache = config.generation.enable_cache;
    let parsed = fetch_and_parse_spec_with_cache(&spec_entry.path, use_cache).await?;
    println!(
        "{}",
        format!("✅ Parsed spec with {} modules", parsed.modules.len()).green()
    );
    println!();
    println!(
        "{}",
        format!(
            "📦 Updating {} module(s): {}",
            selected_modules.len(),
            selected_modules.join(", ")
        )
        .bright_green()
    );
    println!();

    // Filter common schemas based on selected modules only
    let (filtered_module_schemas, common_schemas) =
        filter_common_schemas(&parsed.module_schemas, &selected_modules);

    // Initialize template engine once for all modules
    let project_root = std::env::current_dir().ok();
    let template_engine = TemplateEngine::new(project_root.as_deref())?;

    // Generate code for each module
    let schemas_dir = PathBuf::from(&config.schemas.output);
    let apis_dir = PathBuf::from(&config.apis.output);

    let mut total_files = 0;
    let mut module_summary: Vec<(String, usize)> = Vec::new();

    // Get force and backup settings from config
    let use_force = config.generation.conflict_strategy == "force";
    let use_backup = config.generation.enable_backup;

    // Generate common module first if there are shared schemas
    if !common_schemas.is_empty() {
        println!("{}", "🔨 Regenerating common schemas...".bright_cyan());

        // Shared enum registry to ensure consistent naming between TypeScript and Zod
        let mut shared_enum_registry = std::collections::HashMap::new();

        // Generate TypeScript typings for common schemas
        let common_types = generate_typings_with_registry_and_engine_and_spec(
            &parsed.openapi,
            &parsed.schemas,
            &common_schemas,
            &mut shared_enum_registry,
            &common_schemas,
            Some(&template_engine),
            Some(&spec_entry.name),
        )?;

        // Generate Zod schemas for common schemas (using same registry)
        let common_zod_schemas =
            generate_zod_schemas_with_registry_and_engine_and_spec(
                &parsed.openapi,
                &parsed.schemas,
                &common_schemas,
                &mut shared_enum_registry,
                &common_schemas,
                Some(&template_engine),
                Some(&spec_entry.name),
            )?;

        // Write common schemas (use force if config says so)
        let common_files = write_schemas_with_options(
            &schemas_dir,
            "common",
            &common_types,
            &common_zod_schemas,
            Some(&spec_entry.name),
            use_backup,
            use_force,
        )?;
        total_files += common_files.len();
        module_summary.push(("common".to_string(), common_files.len()));
    }

    for module in &selected_modules {
        println!(
            "{}",
            format!("🔨 Regenerating code for module: {}", module).bright_cyan()
        );

        // Get operations for this module
        let operations = parsed
            .operations_by_tag
            .get(module)
            .cloned()
            .unwrap_or_default();

        if operations.is_empty() {
            println!(
                "{}",
                format!("⚠️  No operations found for module: {}", module).yellow()
            );
            continue;
        }

        // Get schema names used by this module (from filtered schemas)
        let module_schema_names = filtered_module_schemas
            .get(module)
            .cloned()
            .unwrap_or_default();

        // Shared enum registry to ensure consistent naming between TypeScript and Zod
        let mut shared_enum_registry = std::collections::HashMap::new();

        // Generate TypeScript typings
        let types = if !module_schema_names.is_empty() {
            generate_typings_with_registry_and_engine_and_spec(
                &parsed.openapi,
                &parsed.schemas,
                &module_schema_names,
                &mut shared_enum_registry,
                &common_schemas,
                Some(&template_engine),
                Some(&spec_entry.name),
            )?
        } else {
            Vec::new()
        };

        // Generate Zod schemas (using same registry)
        let zod_schemas = if !module_schema_names.is_empty() {
            generate_zod_schemas_with_registry_and_engine_and_spec(
                &parsed.openapi,
                &parsed.schemas,
                &module_schema_names,
                &mut shared_enum_registry,
                &common_schemas,
                Some(&template_engine),
                Some(&spec_entry.name),
            )?
        } else {
            Vec::new()
        };

        // Generate API client (using same enum registry as schemas)
        let api_result = generate_api_client_with_registry_and_engine_and_spec(
            &parsed.openapi,
            &operations,
            module,
            &common_schemas,
            &mut shared_enum_registry,
            Some(&template_engine),
            Some(&spec_entry.name),
        )?;

        // Combine response types with schema types
        let mut all_types = types;
        all_types.extend(api_result.response_types);

        // Write schemas (use force if config says so)
        let schema_files = write_schemas_with_options(
            &schemas_dir,
            module,
            &all_types,
            &zod_schemas,
            Some(&spec_entry.name),
            use_backup,
            use_force,
        )?;
        total_files += schema_files.len();

        // Write API client (use force if config says so)
        let api_files = write_api_client_with_options(
            &apis_dir,
            module,
            &api_result.functions,
            Some(&spec_entry.name),
            use_backup,
            use_force,
        )?;
        total_files += api_files.len();

        let module_file_count = schema_files.len() + api_files.len();
        module_summary.push((module.clone(), module_file_count));
        println!(
            "{}",
            format!(
                "✅ Regenerated {} files for module: {}",
                module_file_count, module
            )
            .green()
        );
    }

    println!();
    println!(
        "{}",
        format!("✨ Successfully updated {} files!", total_files).bright_green()
    );
    println!();
    println!("{}", "Updated files:".bright_cyan());
    println!("  📁 Schemas: {}", config.schemas.output);
    println!("  📁 APIs: {}", config.apis.output);
    println!();
    if !module_summary.is_empty() {
        println!("{}", "Module breakdown:".bright_cyan());
        for (module, count) in &module_summary {
            println!("  • {}: {} files", module, count);
        }
    }

    // Format all generated files with prettier/biome if available
    let mut all_generated_files = Vec::new();

    // Collect schema files recursively
    if schemas_dir.exists() {
        collect_ts_files(&schemas_dir, &mut all_generated_files)?;
    }

    // Collect API files recursively
    if apis_dir.exists() {
        collect_ts_files(&apis_dir, &mut all_generated_files)?;
    }

    // Format files if formatter is available
    if !all_generated_files.is_empty() {
        // Find the common parent directory (where config files are likely located)
        let output_base = schemas_dir
            .parent()
            .and_then(|p| p.parent())
            .or_else(|| apis_dir.parent().and_then(|p| p.parent()));

        let formatter = if let Some(base_dir) = output_base {
            FormatterManager::detect_formatter_from_dir(base_dir)
                .or_else(FormatterManager::detect_formatter)
        } else {
            FormatterManager::detect_formatter()
        };

        if let Some(formatter) = formatter {
            println!("{}", "Formatting generated files...".bright_cyan());
            let original_dir =
                std::env::current_dir().map_err(|e| FileSystemError::ReadFileFailed {
                    path: ".".to_string(),
                    source: e,
                })?;

            if let Some(output_base) = output_base {
                std::env::set_current_dir(output_base).map_err(|e| {
                    FileSystemError::ReadFileFailed {
                        path: output_base.display().to_string(),
                        source: e,
                    }
                })?;

                let relative_files: Vec<PathBuf> = all_generated_files
                    .iter()
                    .filter_map(|p| p.strip_prefix(output_base).ok().map(|p| p.to_path_buf()))
                    .collect();

                if !relative_files.is_empty() {
                    let result = FormatterManager::format_files(&relative_files, formatter);
                    std::env::set_current_dir(&original_dir).map_err(|e| {
                        FileSystemError::ReadFileFailed {
                            path: original_dir.display().to_string(),
                            source: e,
                        }
                    })?;
                    result?;
                } else {
                    std::env::set_current_dir(&original_dir).map_err(|e| {
                        FileSystemError::ReadFileFailed {
                            path: original_dir.display().to_string(),
                            source: e,
                        }
                    })?;
                }
            } else {
                FormatterManager::format_files(&all_generated_files, formatter)?;
            }
            println!("{}", "✅ Files formatted".green());
        }
    }

    Ok(())
}

fn collect_ts_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir).map_err(|e| FileSystemError::ReadFileFailed {
            path: dir.display().to_string(),
            source: e,
        })? {
            let entry = entry.map_err(|e| FileSystemError::ReadFileFailed {
                path: dir.display().to_string(),
                source: e,
            })?;
            let path = entry.path();
            if path.is_dir() {
                collect_ts_files(&path, files)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("ts") {
                files.push(path);
            }
        }
    }
    Ok(())
}
