use crate::config::loader::{load_config, save_config};
use crate::config::validator::validate_config;
use crate::error::{FileSystemError, Result};
use crate::formatter::FormatterManager;
use crate::generator::module_selector::select_modules;
use crate::generator::swagger_parser::filter_common_schemas;
use crate::generator::ts_typings::generate_typings_with_registry;
use crate::generator::writer::{write_api_client_with_options, write_schemas_with_options};
use crate::generator::zod_schema::generate_zod_schemas_with_registry;
use crate::progress::ProgressReporter;
use colored::*;
use std::path::{Path, PathBuf};
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
    all_specs: bool,
    spec_name: Option<String>,
    verbose: bool,
    cache: bool,
    backup: bool,
    force: bool,
) -> Result<()> {
    let mut progress = ProgressReporter::new(verbose);

    progress.success("Starting code generation...");
    println!();

    // Load config
    progress.start_spinner("Loading configuration...");
    let mut config = load_config()?;
    validate_config(&config)?;
    progress.finish_spinner("Configuration loaded");

    // Use config defaults, but allow CLI flags to override
    // CLI flags are false by default (not set), so we check if they were explicitly set
    // For now, we'll use a simple approach: if flag is true, use it; otherwise use config
    let use_cache = if cache {
        true
    } else {
        config.generation.enable_cache
    };
    let use_backup = if backup {
        true
    } else {
        config.generation.enable_backup
    };
    let use_force = if force {
        true
    } else {
        config.generation.conflict_strategy == "force"
    };

    use crate::error::GenerationError;
    use crate::specs::manager::resolve_spec_selection;
    use crate::specs::runner::{run_all_specs, run_single_spec, GenerateOptions};

    // Resolve which specs to generate
    let specs_to_generate = resolve_spec_selection(&config, spec_name.clone(), all_specs)?;

    // Ensure http.ts exists for ALL specs (not just the ones being generated)
    // This fixes the issue where http.ts might be missing if init/add failed to create it
    use crate::generator::writer::{ensure_directory, write_http_client_template};
    for spec in &config.specs {
        let apis_dir = PathBuf::from(&spec.apis.output);
        ensure_directory(&apis_dir)?;
        let http_file = apis_dir.join("http.ts");
        if !http_file.exists() {
            write_http_client_template(&http_file)?;
            if verbose {
                progress.success(&format!("Created {}", http_file.display()));
            }
        }
    }

    let options = GenerateOptions {
        use_cache: if cache {
            true
        } else {
            config.generation.enable_cache
        },
        use_backup: if backup {
            true
        } else {
            config.generation.enable_backup
        },
        use_force: if force {
            true
        } else {
            config.generation.conflict_strategy == "force"
        },
        verbose,
    };

    if specs_to_generate.len() > 1 {
        // Generate all selected specs
        progress.success("Starting multi-spec generation...");
        println!();

        let stats = run_all_specs(&specs_to_generate, &config, &options).await?;

        // Update config with selected modules for each spec
        for stat in &stats {
            if let Some(spec_entry) = config.specs.iter_mut().find(|s| s.name == stat.spec_name) {
                spec_entry.modules.selected = stat.modules.clone();
            }
        }
        save_config(&config)?;

        println!();
        progress.success(&format!(
            "Successfully generated code for {} spec(s)!",
            stats.len()
        ));
        println!();

        // Display summary
        use tabled::{Table, Tabled};
        #[derive(Tabled)]
        struct SpecSummary {
            #[tabled(rename = "Spec")]
            spec: String,
            #[tabled(rename = "Modules")]
            modules: usize,
            #[tabled(rename = "Files")]
            files: usize,
        }

        let table_data: Vec<SpecSummary> = stats
            .iter()
            .map(|s| SpecSummary {
                spec: s.spec_name.clone(),
                modules: s.modules_generated,
                files: s.files_generated,
            })
            .collect();

        let table = Table::new(table_data);
        println!("{}", "Generation summary:".bright_cyan());
        println!("{}", table);
        println!();
    } else {
        // Generate single spec
        let spec_entry = &specs_to_generate[0];
        let stats = run_single_spec(spec_entry, &config, &options).await?;

        // Update config with selected modules
        if let Some(spec_entry) = config.specs.iter_mut().find(|s| s.name == stats.spec_name) {
            spec_entry.modules.selected = stats.modules.clone();
        }
        save_config(&config)?;

        println!();
        progress.success(&format!(
            "Successfully generated {} files for spec '{}'!",
            stats.files_generated, stats.spec_name
        ));
        println!();

        // Use spec-specific configs for output paths
        let schemas_config = &spec_entry.schemas;
        let apis_config = &spec_entry.apis;

        println!("{}", "Generated files:".bright_cyan());
        println!("  📁 Schemas: {}", schemas_config.output);
        println!("  📁 APIs: {}", apis_config.output);
        println!();
    }

    return Ok(());

    // This should never be reached - all code generation goes through run_single_spec or run_all_specs above
    unreachable!("All specs should be handled by run_single_spec or run_all_specs")
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
