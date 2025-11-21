use anyhow::Result;
use colored::*;
use std::path::PathBuf;
use crate::config::loader::load_config;
use crate::config::validator::validate_config;
use crate::generator::api_client::generate_api_client;
use crate::generator::swagger_parser::fetch_and_parse_spec;
use crate::generator::ts_typings::generate_typings;
use crate::generator::writer::{write_api_client, write_schemas};
use crate::generator::zod_schema::generate_zod_schemas;

pub async fn run() -> Result<()> {
    println!("{}", "🔄 Updating generated code...".bright_cyan());
    println!();

    // Load config
    let config = load_config()?;
    validate_config(&config)?;

    // Get spec path from config
    let spec_path = config.spec_path
        .ok_or_else(|| anyhow::anyhow!(
            "No spec path found in config. Please run 'vika-cli generate --spec <path-or-url>' first."
        ))?;

    // Get selected modules from config
    let selected_modules = if config.modules.selected.is_empty() {
        return Err(anyhow::anyhow!(
            "No modules selected in config. Please run 'vika-cli generate --spec <path-or-url>' first."
        ));
    } else {
        config.modules.selected.clone()
    };

    println!("{}", format!("📥 Fetching spec from: {}", spec_path).bright_blue());
    let parsed = fetch_and_parse_spec(&spec_path).await?;
    println!("{}", format!("✅ Parsed spec with {} modules", parsed.modules.len()).green());
    println!();
    println!("{}", format!("📦 Updating {} module(s): {}", selected_modules.len(), selected_modules.join(", ")).bright_green());
    println!();

    // Generate code for each module
    let schemas_dir = PathBuf::from(&config.schemas.output);
    let apis_dir = PathBuf::from(&config.apis.output);

    let mut total_files = 0;

    for module in &selected_modules {
        println!("{}", format!("🔨 Regenerating code for module: {}", module).bright_cyan());

        // Get operations for this module
        let operations = parsed.operations_by_tag
            .get(module)
            .cloned()
            .unwrap_or_default();

        if operations.is_empty() {
            println!("{}", format!("⚠️  No operations found for module: {}", module).yellow());
            continue;
        }

        // Use all schemas (same as generate command)
        let all_schema_names: Vec<String> = parsed.schemas.keys().cloned().collect();

        // Generate TypeScript typings
        let types = generate_typings(&parsed.openapi, &parsed.schemas, &all_schema_names)?;

        // Generate Zod schemas
        let zod_schemas = generate_zod_schemas(&parsed.openapi, &parsed.schemas, &all_schema_names)?;

        // Generate API client
        let api_functions = generate_api_client(&parsed.openapi, &operations)?;

        // Write schemas
        let schema_files = write_schemas(&schemas_dir, module, &types, &zod_schemas)?;
        total_files += schema_files.len();

        // Write API client
        let api_files = write_api_client(&apis_dir, module, &api_functions)?;
        total_files += api_files.len();

        println!("{}", format!("✅ Regenerated {} files for module: {}", schema_files.len() + api_files.len(), module).green());
    }

    println!();
    println!("{}", format!("✨ Successfully updated {} files!", total_files).bright_green());
    println!();
    println!("Updated files:");
    println!("  📁 Schemas: {}", config.schemas.output);
    println!("  📁 APIs: {}", config.apis.output);

    Ok(())
}
