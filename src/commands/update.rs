use crate::config::loader::load_config;
use crate::config::validator::validate_config;
use crate::error::{FileSystemError, Result};
use crate::formatter::FormatterManager;
use crate::generator::swagger_parser::filter_common_schemas;
use crate::generator::writer::write_api_client_with_options;
use colored::*;
use std::path::{Path, PathBuf};

pub async fn run() -> Result<()> {
    println!("{}", "🔄 Updating generated code...".bright_cyan());
    println!();

    // Load config
    let config = load_config()?;
    validate_config(&config)?;

    use crate::error::{FileSystemError, GenerationError};
    use crate::specs::manager::list_specs;

    // Get specs from config
    let specs = list_specs(&config);
    if specs.is_empty() {
        return Err(GenerationError::SpecPathRequired.into());
    }

    // Update all specs
    type SpecSummary = (String, usize, Vec<(String, usize)>);
    let mut all_specs_summary: Vec<SpecSummary> = Vec::new();
    let mut all_generated_files = Vec::new();
    // Track which URLs we've already printed the fetch message for
    let mut printed_urls: std::collections::HashSet<String> = std::collections::HashSet::new();

    for spec in &specs {
        println!();
        println!(
            "{}",
            format!("🔄 Updating spec: {}", spec.name).bright_cyan()
        );
        println!();

        let spec_path = &spec.path;

        // Check if spec path is a temporary file that might not exist anymore
        if !spec_path.starts_with("http://")
            && !spec_path.starts_with("https://")
            && !std::path::Path::new(spec_path).exists()
        {
            println!(
                "{}",
                format!(
                    "⚠️  Skipping spec '{}': spec file no longer exists at {}",
                    spec.name, spec_path
                )
                .yellow()
            );
            continue;
        }

        // Use spec-specific configs (required per spec)
        let schemas_config = &spec.schemas;
        let apis_config = &spec.apis;
        let modules_config = &spec.modules;

        // Ensure http.ts exists for this spec
        use crate::generator::writer::{ensure_directory, write_http_client_template};
        let apis_dir = PathBuf::from(&apis_config.output);
        ensure_directory(&apis_dir)?;
        let http_file = apis_dir.join("http.ts");
        if !http_file.exists() {
            write_http_client_template(&http_file)?;
        }

        // Print fetch message only once per unique URL
        if !printed_urls.contains(spec_path) {
            println!(
                "{}",
                format!("📥 Fetching spec from: {}", spec_path).bright_blue()
            );
            printed_urls.insert(spec_path.clone());
        }

        // Use caching for update command (same as generate)
        let use_cache = config.generation.enable_cache;
        let parsed = crate::generator::swagger_parser::fetch_and_parse_spec_with_cache_and_name(
            spec_path,
            use_cache,
            Some(&spec.name),
        )
        .await?;

        // Get selected modules from config, or select interactively if empty
        let selected_modules = if modules_config.selected.is_empty() {
            // No modules selected in config, select interactively
            println!(
                "{}",
                format!(
                    "No modules selected for spec '{}'. Please select modules to update:",
                    spec.name
                )
                .bright_yellow()
            );
            println!();

            // Filter out ignored modules
            let available_modules: Vec<String> = parsed
                .modules
                .iter()
                .filter(|m| !modules_config.ignore.contains(m))
                .cloned()
                .collect();

            if available_modules.is_empty() {
                println!(
                    "{}",
                    format!("⚠️  Skipping spec '{}': No modules available", spec.name).yellow()
                );
                continue;
            }

            // Select modules interactively
            use crate::generator::module_selector::select_modules;
            let selected = select_modules(&available_modules, &modules_config.ignore)?;

            // Update config with selected modules
            use crate::config::loader::load_config;
            let mut config = load_config()?;
            if let Some(spec_entry) = config.specs.iter_mut().find(|s| s.name == spec.name) {
                spec_entry.modules.selected = selected.clone();
            }
            use crate::config::loader::save_config;
            save_config(&config)?;

            selected
        } else {
            modules_config.selected.clone()
        };
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
        let template_engine =
            crate::templates::engine::TemplateEngine::new(project_root.as_deref())?;

        // Generate code for each module (using spec-specific or global output directories)
        let schemas_dir = PathBuf::from(&schemas_config.output);
        let apis_dir = PathBuf::from(&apis_config.output);

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
            // Pass empty common_schemas list so common schemas don't prefix themselves with "Common."
            let common_types =
                crate::generator::ts_typings::generate_typings_with_registry_and_engine_and_spec(
                    &parsed.openapi,
                    &parsed.schemas,
                    &common_schemas,
                    &mut shared_enum_registry,
                    &[], // Empty list - common schemas shouldn't prefix themselves
                    Some(&template_engine),
                    Some(&spec.name),
                )?;

            // Generate Zod schemas for common schemas (using same registry)
            // Pass empty common_schemas list so common schemas don't prefix themselves with "Common."
            let common_zod_schemas =
            crate::generator::zod_schema::generate_zod_schemas_with_registry_and_engine_and_spec(
                &parsed.openapi,
                &parsed.schemas,
                &common_schemas,
                &mut shared_enum_registry,
                &[], // Empty list - common schemas shouldn't prefix themselves
                Some(&template_engine),
                Some(&spec.name),
            )?;

            // Write common schemas (use force if config says so)
            use crate::generator::writer::write_schemas_with_module_mapping;
            let common_files = write_schemas_with_module_mapping(
                &schemas_dir,
                "common",
                &common_types,
                &common_zod_schemas,
                Some(&spec.name), // spec_name for multi-spec mode
                use_backup,
                use_force,
                Some(&filtered_module_schemas),
                &common_schemas,
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
                crate::generator::ts_typings::generate_typings_with_registry_and_engine_and_spec(
                    &parsed.openapi,
                    &parsed.schemas,
                    &module_schema_names,
                    &mut shared_enum_registry,
                    &common_schemas,
                    Some(&template_engine),
                    Some(&spec.name),
                )?
            } else {
                Vec::new()
            };

            // Generate Zod schemas (using same registry)
            let zod_schemas = if !module_schema_names.is_empty() {
                crate::generator::zod_schema::generate_zod_schemas_with_registry_and_engine_and_spec(
                &parsed.openapi,
                &parsed.schemas,
                &module_schema_names,
                &mut shared_enum_registry,
                &common_schemas,
                Some(&template_engine),
                Some(&spec.name),
            )?
            } else {
                Vec::new()
            };

            // Generate API client (using same enum registry as schemas)
            let api_result =
            crate::generator::api_client::generate_api_client_with_registry_and_engine_and_spec(
                &parsed.openapi,
                &operations,
                module,
                &common_schemas,
                &mut shared_enum_registry,
                Some(&template_engine),
                Some(&spec.name),
            )?;

            // Combine response types with schema types
            let mut all_types = types;
            all_types.extend(api_result.response_types);

            // Write schemas (use force if config says so)
            use crate::generator::writer::write_schemas_with_module_mapping;
            let schema_files = write_schemas_with_module_mapping(
                &schemas_dir,
                module,
                &all_types,
                &zod_schemas,
                Some(&spec.name), // spec_name for multi-spec mode
                use_backup,
                use_force,
                Some(&filtered_module_schemas),
                &common_schemas,
            )?;
            total_files += schema_files.len();

            // Write API client (use force if config says so)
            let api_files = write_api_client_with_options(
                &apis_dir,
                module,
                &api_result.functions,
                Some(&spec.name), // spec_name for multi-spec mode
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
            format!(
                "✨ Successfully updated {} files for spec '{}'!",
                total_files, spec.name
            )
            .bright_green()
        );
        println!();
        println!(
            "{}",
            format!("Updated files for '{}':", spec.name).bright_cyan()
        );
        println!("  📁 Schemas: {}", schemas_config.output);
        println!("  📁 APIs: {}", apis_config.output);

        // Store summary for this spec
        all_specs_summary.push((spec.name.clone(), total_files, module_summary.clone()));

        // Collect files for this spec for formatting
        let current_dir = std::env::current_dir().map_err(|e| FileSystemError::ReadFileFailed {
            path: ".".to_string(),
            source: e,
        })?;

        let schemas_dir_abs = if schemas_dir.is_absolute() {
            schemas_dir.clone()
        } else {
            current_dir.join(&schemas_dir)
        };
        let apis_dir_abs = if apis_dir.is_absolute() {
            apis_dir.clone()
        } else {
            current_dir.join(&apis_dir)
        };

        // Collect schema files recursively
        if schemas_dir_abs.exists() {
            collect_ts_files(&schemas_dir_abs, &mut all_generated_files)?;
        }

        // Collect API files recursively
        if apis_dir_abs.exists() {
            collect_ts_files(&apis_dir_abs, &mut all_generated_files)?;
        }
    }

    // Print overall summary
    println!();
    println!("{}", "=".repeat(60).bright_black());
    println!();
    let total_all_files: usize = all_specs_summary.iter().map(|(_, count, _)| count).sum();
    println!(
        "{}",
        format!(
            "✨ Successfully updated {} files across {} spec(s)!",
            total_all_files,
            all_specs_summary.len()
        )
        .bright_green()
    );
    println!();
    println!("{}", "Summary by spec:".bright_cyan());
    for (spec_name, file_count, module_summary) in &all_specs_summary {
        println!("  📦 {}: {} files", spec_name, file_count);
        if !module_summary.is_empty() {
            for (module, count) in module_summary {
                println!("    • {}: {} files", module, count);
            }
        }
    }
    println!();

    // Format files if formatter is available
    if !all_generated_files.is_empty() {
        // Find the common parent directory (where config files are likely located)
        // Try to find it from the first file path, or use current directory
        let output_base = all_generated_files.first().and_then(|first_file| {
            first_file
                .parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.parent())
        });

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
                // Ensure output_base is not empty
                if output_base.as_os_str().is_empty() {
                    // Fallback: use current directory
                    FormatterManager::format_files(&all_generated_files, formatter)?;
                } else {
                    std::env::set_current_dir(output_base).map_err(|e| {
                        FileSystemError::ReadFileFailed {
                            path: output_base.display().to_string(),
                            source: e,
                        }
                    })?;

                    // Convert paths to relative paths from output base directory
                    let relative_files: Vec<PathBuf> = all_generated_files
                        .iter()
                        .filter_map(|p| {
                            p.strip_prefix(output_base)
                                .ok()
                                .map(|p| p.to_path_buf())
                                .filter(|p| !p.as_os_str().is_empty())
                        })
                        .collect();

                    if !relative_files.is_empty() {
                        let result = FormatterManager::format_files(&relative_files, formatter);

                        // Restore original directory
                        std::env::set_current_dir(&original_dir).map_err(|e| {
                            FileSystemError::ReadFileFailed {
                                path: original_dir.display().to_string(),
                                source: e,
                            }
                        })?;

                        result?;

                        // Update metadata for formatted files to reflect formatted content hash (batch update)
                        use crate::generator::writer::batch_update_file_metadata_from_disk;
                        if let Err(e) = batch_update_file_metadata_from_disk(&all_generated_files) {
                            // Log but don't fail - metadata update is best effort
                            eprintln!("Warning: Failed to update metadata: {}", e);
                        }
                    } else {
                        // Restore original directory
                        std::env::set_current_dir(&original_dir).map_err(|e| {
                            FileSystemError::ReadFileFailed {
                                path: original_dir.display().to_string(),
                                source: e,
                            }
                        })?;
                    }
                }
            } else {
                FormatterManager::format_files(&all_generated_files, formatter)?;

                // Update metadata for formatted files to reflect formatted content hash (batch update)
                use crate::generator::writer::batch_update_file_metadata_from_disk;
                if let Err(e) = batch_update_file_metadata_from_disk(&all_generated_files) {
                    // Log but don't fail - metadata update is best effort
                    eprintln!("Warning: Failed to update metadata: {}", e);
                }
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
            // Skip if path is empty or invalid
            if path.as_os_str().is_empty() {
                continue;
            }
            if path.is_dir() {
                collect_ts_files(&path, files)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("ts") {
                files.push(path);
            }
        }
    }
    Ok(())
}
