use crate::config::loader::load_config;
use crate::config::validator::validate_config;
use crate::error::Result;
use crate::generator::api_client::generate_api_client;
use crate::generator::swagger_parser::{fetch_and_parse_spec, filter_common_schemas};
use crate::generator::ts_typings::generate_typings_with_registry;
use crate::generator::writer::{write_api_client, write_schemas};
use crate::generator::zod_schema::generate_zod_schemas_with_registry;
use colored::*;
use std::path::PathBuf;

pub async fn run() -> Result<()> {
    println!("{}", "🔄 Updating generated code...".bright_cyan());
    println!();

    // Load config
    let config = load_config()?;
    validate_config(&config)?;

    use crate::error::GenerationError;

    // Get spec path from config
    let spec_path = config
        .spec_path
        .ok_or(GenerationError::SpecPathRequired)?;

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
    let parsed = fetch_and_parse_spec(&spec_path).await?;
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

    // Generate code for each module
    let schemas_dir = PathBuf::from(&config.schemas.output);
    let apis_dir = PathBuf::from(&config.apis.output);

    let mut total_files = 0;
    let mut module_summary: Vec<(String, usize)> = Vec::new();

    // Generate common module first if there are shared schemas
    if !common_schemas.is_empty() {
        println!("{}", "🔨 Regenerating common schemas...".bright_cyan());

        // Shared enum registry to ensure consistent naming between TypeScript and Zod
        let mut shared_enum_registry = std::collections::HashMap::new();

        // Generate TypeScript typings for common schemas
        let common_types = generate_typings_with_registry(
            &parsed.openapi,
            &parsed.schemas,
            &common_schemas,
            &mut shared_enum_registry,
        )?;

        // Generate Zod schemas for common schemas (using same registry)
        let common_zod_schemas = generate_zod_schemas_with_registry(
            &parsed.openapi,
            &parsed.schemas,
            &common_schemas,
            &mut shared_enum_registry,
        )?;

        // Write common schemas
        let common_files =
            write_schemas(&schemas_dir, "common", &common_types, &common_zod_schemas)?;
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
            generate_typings_with_registry(
                &parsed.openapi,
                &parsed.schemas,
                &module_schema_names,
                &mut shared_enum_registry,
            )?
        } else {
            Vec::new()
        };

        // Generate Zod schemas (using same registry)
        let zod_schemas = if !module_schema_names.is_empty() {
            generate_zod_schemas_with_registry(
                &parsed.openapi,
                &parsed.schemas,
                &module_schema_names,
                &mut shared_enum_registry,
            )?
        } else {
            Vec::new()
        };

        // Generate API client
        let api_functions =
            generate_api_client(&parsed.openapi, &operations, module, &common_schemas)?;

        // Write schemas
        let schema_files = write_schemas(&schemas_dir, module, &types, &zod_schemas)?;
        total_files += schema_files.len();

        // Write API client
        let api_files = write_api_client(&apis_dir, module, &api_functions)?;
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

    Ok(())
}
