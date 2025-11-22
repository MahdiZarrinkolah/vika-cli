use crate::config::loader::{load_config, save_config};
use crate::config::validator::validate_config;
use crate::error::Result;
use crate::generator::api_client::generate_api_client_with_registry;
use crate::generator::module_selector::select_modules;
use crate::generator::swagger_parser::filter_common_schemas;
use crate::generator::ts_typings::generate_typings_with_registry;
use crate::generator::writer::{write_api_client_with_options, write_schemas_with_options};
use crate::generator::zod_schema::generate_zod_schemas_with_registry;
use crate::progress::ProgressReporter;
use colored::*;
use std::path::PathBuf;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct ModuleSummary {
    #[tabled(rename = "Module")]
    module: String,
    #[tabled(rename = "Files")]
    files: usize,
    #[tabled(rename = "Status")]
    status: String,
}

pub async fn run(
    spec: Option<String>,
    verbose: bool,
    _cache: bool,
    _backup: bool,
    _force: bool,
) -> Result<()> {
    let mut progress = ProgressReporter::new(verbose);

    progress.success("Starting code generation...");
    println!();

    // Load config
    progress.start_spinner("Loading configuration...");
    let mut config = load_config()?;
    validate_config(&config)?;
    progress.finish_spinner("Configuration loaded");

    use crate::error::GenerationError;

    // Get spec path
    let spec_path = spec.ok_or(GenerationError::SpecPathRequired)?;

    // Fetch and parse Swagger spec
    progress.start_spinner(&format!("Fetching spec from: {}", spec_path));
    let parsed =
        crate::generator::swagger_parser::fetch_and_parse_spec_with_cache(&spec_path, _cache)
            .await?;
    progress.finish_spinner(&format!(
        "Parsed spec with {} modules",
        parsed.modules.len()
    ));
    println!();

    // Filter out ignored modules
    let available_modules: Vec<String> = parsed
        .modules
        .iter()
        .filter(|m| !config.modules.ignore.contains(m))
        .cloned()
        .collect();

    if available_modules.is_empty() {
        return Err(GenerationError::NoModulesAvailable.into());
    }

    // Select modules interactively
    let selected_modules = select_modules(&available_modules, &config.modules.ignore)?;
    println!();

    // Display module selection summary
    if verbose {
        progress.info(&format!(
            "Selected {} module(s): {}",
            selected_modules.len(),
            selected_modules.join(", ")
        ));
    }
    println!();

    // Save spec path and selected modules to config
    config.spec_path = Some(spec_path.clone());
    config.modules.selected = selected_modules.clone();
    save_config(&config)?;

    // Filter common schemas based on selected modules only
    let (filtered_module_schemas, common_schemas) =
        filter_common_schemas(&parsed.module_schemas, &selected_modules);

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
        progress.start_spinner("Generating common schemas...");

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
        let common_files = write_schemas_with_options(
            &schemas_dir,
            "common",
            &common_types,
            &common_zod_schemas,
            _backup,
            _force,
        )?;
        total_files += common_files.len();
        module_summary.push(("common".to_string(), common_files.len()));
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

        // Generate API client (using same enum registry as schemas)
        let api_result = generate_api_client_with_registry(
            &parsed.openapi,
            &operations,
            module,
            &common_schemas,
            &mut shared_enum_registry,
        )?;

        // Combine response types with schema types
        let mut all_types = types;
        all_types.extend(api_result.response_types);

        // Write schemas (with backup and conflict detection)
        let schema_files = write_schemas_with_options(
            &schemas_dir,
            module,
            &all_types,
            &zod_schemas,
            _backup,
            _force,
        )?;
        total_files += schema_files.len();

        // Write API client (with backup and conflict detection)
        let api_files =
            write_api_client_with_options(&apis_dir, module, &api_result.functions, _backup, _force)?;
        total_files += api_files.len();

        let module_file_count = schema_files.len() + api_files.len();
        module_summary.push((module.clone(), module_file_count));
        progress.finish_spinner(&format!(
            "Generated {} files for module: {}",
            module_file_count, module
        ));
    }

    println!();
    progress.success(&format!("Successfully generated {} files!", total_files));
    println!();
    println!("{}", "Generated files:".bright_cyan());
    println!("  📁 Schemas: {}", config.schemas.output);
    println!("  📁 APIs: {}", config.apis.output);
    println!();

    // Display module summary table
    if !module_summary.is_empty() {
        let table_data: Vec<ModuleSummary> = module_summary
            .iter()
            .map(|(module, count)| ModuleSummary {
                module: module.clone(),
                files: *count,
                status: "✅".to_string(),
            })
            .collect();

        let table = Table::new(table_data);
        println!("{}", "Module breakdown:".bright_cyan());
        println!("{}", table);
        println!();
    }

    Ok(())
}
