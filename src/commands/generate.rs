use anyhow::Result;
use colored::*;
use std::path::PathBuf;
use crate::config::loader::{load_config, save_config};
use crate::config::validator::validate_config;
use crate::generator::api_client::generate_api_client;
use crate::generator::module_selector::select_modules;
use crate::generator::swagger_parser::{fetch_and_parse_spec, filter_common_schemas};
use crate::generator::ts_typings::generate_typings_with_registry;
use crate::generator::writer::{write_api_client, write_schemas};
use crate::generator::zod_schema::generate_zod_schemas_with_registry;

pub async fn run(spec: Option<String>) -> Result<()> {
    println!("{}", "🚀 Starting code generation...".bright_cyan());
    println!();

    // Load config
    let mut config = load_config()?;
    validate_config(&config)?;

    // Get spec path
    let spec_path = spec.ok_or_else(|| anyhow::anyhow!("Spec path or URL is required. Use --spec <path-or-url>"))?;

    // Fetch and parse Swagger spec
    println!("{}", format!("📥 Fetching spec from: {}", spec_path).bright_blue());
    let parsed = fetch_and_parse_spec(&spec_path).await?;
    println!("{}", format!("✅ Parsed spec with {} modules", parsed.modules.len()).green());
    println!();

    // Filter out ignored modules
    let available_modules: Vec<String> = parsed.modules
        .iter()
        .filter(|m| !config.modules.ignore.contains(m))
        .cloned()
        .collect();

    if available_modules.is_empty() {
        return Err(anyhow::anyhow!("No modules available after filtering"));
    }

    // Select modules interactively
    let selected_modules = select_modules(&available_modules, &config.modules.ignore)?;
    println!();
    println!("{}", format!("📦 Selected {} module(s): {}", selected_modules.len(), selected_modules.join(", ")).bright_green());
    println!();

    // Save spec path and selected modules to config
    config.spec_path = Some(spec_path.clone());
    config.modules.selected = selected_modules.clone();
    save_config(&config)?;

    // Filter common schemas based on selected modules only
    let (filtered_module_schemas, common_schemas) = filter_common_schemas(&parsed.module_schemas, &selected_modules);

    // Generate code for each module
    let schemas_dir = PathBuf::from(&config.schemas.output);
    let apis_dir = PathBuf::from(&config.apis.output);

    // Ensure http.ts file exists
    let http_file = apis_dir.join("http.ts");
    if !http_file.exists() {
        use crate::generator::writer::write_http_client_template;
        write_http_client_template(&http_file)?;
        println!("{}", format!("✅ Created {}", http_file.display()).green());
    }

    let mut total_files = 0;
    let mut module_summary: Vec<(String, usize)> = Vec::new();

    // Generate common module first if there are shared schemas
    if !common_schemas.is_empty() {
        println!("{}", format!("🔨 Generating common schemas...").bright_cyan());
        
        // Shared enum registry to ensure consistent naming between TypeScript and Zod
        let mut shared_enum_registry = std::collections::HashMap::new();
        
        // Generate TypeScript typings for common schemas
        let common_types = generate_typings_with_registry(&parsed.openapi, &parsed.schemas, &parsed.common_schemas, &mut shared_enum_registry)?;
        
        // Generate Zod schemas for common schemas (using same registry)
        let common_zod_schemas = generate_zod_schemas_with_registry(&parsed.openapi, &parsed.schemas, &parsed.common_schemas, &mut shared_enum_registry)?;
        
        // Write common schemas
        let common_files = write_schemas(&schemas_dir, "common", &common_types, &common_zod_schemas)?;
        total_files += common_files.len();
        module_summary.push(("common".to_string(), common_files.len()));
    }

    for module in &selected_modules {
        println!("{}", format!("🔨 Generating code for module: {}", module).bright_cyan());

        // Get operations for this module
        let operations = parsed.operations_by_tag
            .get(module)
            .cloned()
            .unwrap_or_default();

        if operations.is_empty() {
            println!("{}", format!("⚠️  No operations found for module: {}", module).yellow());
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
            generate_typings_with_registry(&parsed.openapi, &parsed.schemas, &module_schema_names, &mut shared_enum_registry)?
        } else {
            Vec::new()
        };

        // Generate Zod schemas (using same registry)
        let zod_schemas = if !module_schema_names.is_empty() {
            generate_zod_schemas_with_registry(&parsed.openapi, &parsed.schemas, &module_schema_names, &mut shared_enum_registry)?
        } else {
            Vec::new()
        };

        // Generate API client
        let api_functions = generate_api_client(&parsed.openapi, &operations, module, &common_schemas)?;

        // Write schemas
        let schema_files = write_schemas(&schemas_dir, module, &types, &zod_schemas)?;
        total_files += schema_files.len();

        // Write API client
        let api_files = write_api_client(&apis_dir, module, &api_functions)?;
        total_files += api_files.len();

        let module_file_count = schema_files.len() + api_files.len();
        module_summary.push((module.clone(), module_file_count));
        println!("{}", format!("✅ Generated {} files for module: {}", module_file_count, module).green());
    }

    println!();
    println!("{}", format!("✨ Successfully generated {} files!", total_files).bright_green());
    println!();
    println!("{}", "Generated files:".bright_cyan());
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
