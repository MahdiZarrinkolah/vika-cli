use crate::config::model::{Config, SpecEntry};
use crate::error::Result;
use crate::formatter::FormatterManager;
use crate::generator::api_client::generate_api_client_with_registry_and_engine_and_spec;
use crate::generator::module_selector::select_modules;
use crate::generator::swagger_parser::filter_common_schemas;
use crate::generator::ts_typings::generate_typings_with_registry_and_engine_and_spec;
use crate::generator::writer::write_api_client_with_options;
use crate::generator::zod_schema::generate_zod_schemas_with_registry_and_engine_and_spec;
use crate::progress::ProgressReporter;
use std::path::PathBuf;

/// Statistics for a single spec generation run
#[derive(Debug, Clone)]
pub struct GenerationStats {
    pub spec_name: String,
    pub modules_generated: usize,
    pub files_generated: usize,
    pub modules: Vec<String>,
}

/// Hook generator type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookType {
    ReactQuery,
    Swr,
}

/// Options for generation
pub struct GenerateOptions {
    pub use_cache: bool,
    pub use_backup: bool,
    pub use_force: bool,
    pub verbose: bool,
    pub hook_type: Option<HookType>,
}

/// Generate code for a single spec
pub async fn run_single_spec(
    spec: &SpecEntry,
    _config: &Config,
    options: &GenerateOptions,
) -> Result<GenerationStats> {
    let mut progress = ProgressReporter::new(options.verbose);
    // Always use spec name (even for single spec)
    let spec_name = Some(spec.name.as_str());

    progress.start_spinner(&format!("Fetching spec from: {}", spec.path));
    let parsed = crate::generator::swagger_parser::fetch_and_parse_spec_with_cache_and_name(
        &spec.path,
        options.use_cache,
        Some(&spec.name),
    )
    .await?;
    progress.finish_spinner(&format!(
        "Parsed spec with {} modules",
        parsed.modules.len()
    ));

    // Use spec-specific configs (required per spec)
    let schemas_config = &spec.schemas;
    let apis_config = &spec.apis;
    let modules_config = &spec.modules;

    // Filter out ignored modules (using spec-specific or global)
    let available_modules: Vec<String> = parsed
        .modules
        .iter()
        .filter(|m| !modules_config.ignore.contains(m))
        .cloned()
        .collect();

    if available_modules.is_empty() {
        return Err(crate::error::GenerationError::NoModulesAvailable.into());
    }

    // Use pre-selected modules from config if available, otherwise prompt interactively
    let selected_modules = if !modules_config.selected.is_empty() {
        // Validate that all selected modules are available
        let valid_selected: Vec<String> = modules_config
            .selected
            .iter()
            .filter(|m| available_modules.contains(m))
            .cloned()
            .collect();

        if valid_selected.is_empty() {
            return Err(crate::error::GenerationError::NoModulesSelected.into());
        }

        valid_selected
    } else {
        // Select modules interactively (using spec-specific or global ignore list)
        select_modules(&available_modules, &modules_config.ignore)?
    };

    // Filter common schemas based on selected modules only
    let (filtered_module_schemas, common_schemas) =
        filter_common_schemas(&parsed.module_schemas, &selected_modules);

    // Generate code for each module (using spec-specific or global output directories)
    let schemas_dir = PathBuf::from(&schemas_config.output);
    let apis_dir = PathBuf::from(&apis_config.output);

    // Ensure runtime client exists (only once per output directory)
    use crate::generator::writer::write_runtime_client;
    let runtime_files = write_runtime_client(&apis_dir, spec_name)?;
    if options.verbose && !runtime_files.is_empty() {
        progress.success("Created runtime client files");
    }

    let mut total_files = 0;

    // Generate common module first if there are shared schemas
    if !common_schemas.is_empty() {
        progress.start_spinner("Generating common schemas...");

        // Shared enum registry to ensure consistent naming between TypeScript and Zod
        let mut shared_enum_registry = std::collections::HashMap::new();

        // Initialize template engine
        let project_root = std::env::current_dir().ok();
        let template_engine =
            crate::templates::engine::TemplateEngine::new(project_root.as_deref())?;

        // Generate TypeScript typings for common schemas
        // Pass empty common_schemas list so common schemas don't prefix themselves with "Common."
        let common_types = generate_typings_with_registry_and_engine_and_spec(
            &parsed.openapi,
            &parsed.schemas,
            &common_schemas,
            &mut shared_enum_registry,
            &[], // Empty list - common schemas shouldn't prefix themselves
            Some(&template_engine),
            spec_name,
        )?;

        // Generate Zod schemas for common schemas (using same registry)
        // Pass empty common_schemas list so common schemas don't prefix themselves with "Common."
        let common_zod_schemas = generate_zod_schemas_with_registry_and_engine_and_spec(
            &parsed.openapi,
            &parsed.schemas,
            &common_schemas,
            &mut shared_enum_registry,
            &[], // Empty list - common schemas shouldn't prefix themselves
            Some(&template_engine),
            spec_name,
        )?;

        // Write common schemas
        use crate::generator::writer::write_schemas_with_module_mapping;
        let common_files = write_schemas_with_module_mapping(
            &schemas_dir,
            "common",
            &common_types,
            &common_zod_schemas,
            spec_name,
            options.use_backup,
            options.use_force,
            Some(&filtered_module_schemas),
            &common_schemas,
        )?;
        total_files += common_files.len();
        progress.finish_spinner(&format!(
            "Generated {} common schema files",
            common_files.len()
        ));
    }

    for module in &selected_modules {
        progress.start_spinner(&format!("Generating code for module: {}", module));

        // Get operations for this module
        let operations = parsed
            .operations_by_tag
            .get(module)
            .cloned()
            .unwrap_or_default();

        if operations.is_empty() {
            progress.warning(&format!("No operations found for module: {}", module));
            continue;
        }

        // Get schema names used by this module (from filtered schemas)
        let module_schema_names = filtered_module_schemas
            .get(module)
            .cloned()
            .unwrap_or_default();

        // Initialize template engine
        let project_root = std::env::current_dir().ok();
        let template_engine =
            crate::templates::engine::TemplateEngine::new(project_root.as_deref())?;

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
                spec_name,
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
                spec_name,
            )?
        } else {
            Vec::new()
        };

        // Generate query params types and Zod schemas
        // Pass existing types and zod schemas to avoid duplicates
        use crate::generator::query_params::{
            generate_query_params_for_module, QueryParamsContext,
        };
        let query_params_result = generate_query_params_for_module(QueryParamsContext {
            openapi: &parsed.openapi,
            operations: &operations,
            enum_registry: &mut shared_enum_registry,
            template_engine: Some(&template_engine),
            spec_name,
            existing_types: &types,
            existing_zod_schemas: &zod_schemas,
        })?;

        // Generate API client (using same enum registry as schemas)
        let api_result = generate_api_client_with_registry_and_engine_and_spec(
            &parsed.openapi,
            &operations,
            module,
            &common_schemas,
            &mut shared_enum_registry,
            Some(&template_engine),
            spec_name,
        )?;

        // Response types are written to API files, not schema files
        // Combine schema types with query params types
        let mut all_types = types;
        all_types.extend(query_params_result.types);

        // Combine Zod schemas with query params Zod schemas
        let mut all_zod_schemas = zod_schemas;
        all_zod_schemas.extend(query_params_result.zod_schemas);

        // Write schemas (with backup and conflict detection)
        // Pass module_schemas mapping to enable cross-module enum imports
        use crate::generator::writer::write_schemas_with_module_mapping;
        let schema_files = write_schemas_with_module_mapping(
            &schemas_dir,
            module,
            &all_types,
            &all_zod_schemas,
            spec_name,
            options.use_backup,
            options.use_force,
            Some(&filtered_module_schemas),
            &common_schemas,
        )?;
        total_files += schema_files.len();

        // Write API client (with backup and conflict detection)
        let api_files = write_api_client_with_options(
            &apis_dir,
            module,
            &api_result.functions,
            spec_name,
            options.use_backup,
            options.use_force,
        )?;
        total_files += api_files.len();

        // Generate hooks if requested
        if let Some(hook_type) = options.hook_type {
            progress.start_spinner(&format!("Generating hooks for module: {}", module));

            // Generate query keys first (hooks depend on them)
            use crate::generator::query_keys::generate_query_keys;
            let query_keys_context = generate_query_keys(&operations, module, spec_name);

            // Render query keys template
            let query_keys_content = template_engine.render(
                crate::templates::registry::TemplateId::QueryKeys,
                &query_keys_context,
            )?;

            // Write query keys file
            // Default output directory: src/query-keys/{spec}/{module}.ts
            let root_dir = std::env::current_dir().ok();
            let query_keys_output = if let Some(ref root) = root_dir {
                if let Some(spec) = spec_name {
                    root.join("src").join("query-keys").join(spec)
                } else {
                    root.join("src").join("query-keys")
                }
            } else {
                PathBuf::from("src/query-keys")
            };

            use crate::generator::writer::write_query_keys_with_options;
            write_query_keys_with_options(
                &query_keys_output,
                module,
                &query_keys_content,
                spec_name,
                options.use_backup,
                options.use_force,
            )?;
            total_files += 1;

            // Generate hooks based on type
            let hooks = match hook_type {
                HookType::ReactQuery => {
                    use crate::generator::hooks::react_query::generate_react_query_hooks;
                    generate_react_query_hooks(
                        &parsed.openapi,
                        &operations,
                        module,
                        spec_name,
                        &common_schemas,
                        &mut shared_enum_registry,
                        &template_engine,
                    )?
                }
                HookType::Swr => {
                    use crate::generator::hooks::swr::generate_swr_hooks;
                    generate_swr_hooks(
                        &parsed.openapi,
                        &operations,
                        module,
                        spec_name,
                        &common_schemas,
                        &mut shared_enum_registry,
                        &template_engine,
                    )?
                }
            };

            // Write hooks files
            // Default output directory: src/hooks/{spec}/{module}/
            let hooks_output = if let Some(ref root) = root_dir {
                if let Some(spec) = spec_name {
                    root.join("src").join("hooks").join(spec)
                } else {
                    root.join("src").join("hooks")
                }
            } else {
                PathBuf::from("src/hooks")
            };

            use crate::generator::writer::write_hooks_with_options;
            let hook_files = write_hooks_with_options(
                &hooks_output,
                module,
                &hooks,
                spec_name,
                options.use_backup,
                options.use_force,
            )?;
            total_files += hook_files.len();

            progress.finish_spinner(&format!(
                "Generated {} hook files for module: {}",
                hook_files.len(),
                module
            ));
        }

        progress.finish_spinner(&format!(
            "Generated {} files for module: {}",
            schema_files.len() + api_files.len(),
            module
        ));
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

    // Collect hook files recursively if hooks were generated
    if options.hook_type.is_some() {
        let root_dir = std::env::current_dir().ok();
        let hooks_dir = if let Some(ref root) = root_dir {
            if let Some(spec) = spec_name {
                root.join("src").join("hooks").join(spec)
            } else {
                root.join("src").join("hooks")
            }
        } else {
            PathBuf::from("src/hooks")
        };
        if hooks_dir.exists() {
            collect_ts_files(&hooks_dir, &mut all_generated_files)?;
        }

        // Collect query keys files
        let query_keys_dir = if let Some(ref root) = root_dir {
            if let Some(spec) = spec_name {
                root.join("src").join("query-keys").join(spec)
            } else {
                root.join("src").join("query-keys")
            }
        } else {
            PathBuf::from("src/query-keys")
        };
        if query_keys_dir.exists() {
            collect_ts_files(&query_keys_dir, &mut all_generated_files)?;
        }
    }

    // Format files if formatter is available
    if !all_generated_files.is_empty() {
        // Get current directory to resolve relative paths
        let current_dir =
            std::env::current_dir().map_err(|e| crate::error::FileSystemError::ReadFileFailed {
                path: ".".to_string(),
                source: e,
            })?;

        // Resolve to absolute paths
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

        let output_base = schemas_dir_abs
            .parent()
            .and_then(|p| p.parent())
            .or_else(|| apis_dir_abs.parent().and_then(|p| p.parent()));

        let formatter = if let Some(base_dir) = output_base {
            FormatterManager::detect_formatter_from_dir(base_dir)
                .or_else(FormatterManager::detect_formatter)
        } else {
            FormatterManager::detect_formatter()
        };

        if let Some(formatter) = formatter {
            progress.start_spinner("Formatting generated files...");
            let original_dir = std::env::current_dir().map_err(|e| {
                crate::error::FileSystemError::ReadFileFailed {
                    path: ".".to_string(),
                    source: e,
                }
            })?;

            if let Some(output_base) = output_base {
                // Ensure output_base is not empty
                if output_base.as_os_str().is_empty() {
                    // Fallback: use current directory
                    FormatterManager::format_files(&all_generated_files, formatter)?;
                } else {
                    std::env::set_current_dir(output_base).map_err(|e| {
                        crate::error::FileSystemError::ReadFileFailed {
                            path: output_base.display().to_string(),
                            source: e,
                        }
                    })?;

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

                        std::env::set_current_dir(&original_dir).map_err(|e| {
                            crate::error::FileSystemError::ReadFileFailed {
                                path: original_dir.display().to_string(),
                                source: e,
                            }
                        })?;

                        result?;

                        // Update metadata for formatted files to reflect formatted content hash (batch update)
                        use crate::generator::writer::batch_update_file_metadata_from_disk;
                        if let Err(e) = batch_update_file_metadata_from_disk(&all_generated_files) {
                            // Log but don't fail - metadata update is best effort
                            progress.warning(&format!("Failed to update metadata: {}", e));
                        }
                    } else {
                        std::env::set_current_dir(&original_dir).map_err(|e| {
                            crate::error::FileSystemError::ReadFileFailed {
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
                    progress.warning(&format!("Failed to update metadata: {}", e));
                }
            }
            progress.finish_spinner("Files formatted");
        }
    }

    Ok(GenerationStats {
        spec_name: spec.name.clone(),
        modules_generated: selected_modules.len(),
        files_generated: total_files,
        modules: selected_modules,
    })
}

/// Generate code for multiple specs sequentially
pub async fn run_all_specs(
    specs: &[SpecEntry],
    config: &Config,
    options: &GenerateOptions,
) -> Result<Vec<GenerationStats>> {
    let mut stats = Vec::new();
    for spec in specs {
        let result = run_single_spec(spec, config, options).await?;
        stats.push(result);
    }
    Ok(stats)
}

fn collect_ts_files(dir: &std::path::Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if dir.is_dir() {
        for entry in
            std::fs::read_dir(dir).map_err(|e| crate::error::FileSystemError::ReadFileFailed {
                path: dir.display().to_string(),
                source: e,
            })?
        {
            let entry = entry.map_err(|e| crate::error::FileSystemError::ReadFileFailed {
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
