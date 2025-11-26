use crate::config::loader::{load_config, save_config};
use crate::config::validator::validate_config;
use crate::error::Result;
use crate::progress::ProgressReporter;
use colored::*;
use std::path::PathBuf;
use tabled::Tabled;

#[allow(clippy::too_many_arguments)]
pub async fn run(
    _spec: Option<String>,
    all_specs: bool,
    spec_name: Option<String>,
    verbose: bool,
    cache: bool,
    backup: bool,
    force: bool,
    react_query: bool,
    swr: bool,
) -> Result<()> {
    // Validate hook flags - only one can be set
    let hook_flags_count = [react_query, swr].iter().filter(|&&f| f).count();
    if hook_flags_count > 1 {
        return Err(crate::error::GenerationError::InvalidHookFlags.into());
    }

    let mut progress = ProgressReporter::new(verbose);

    progress.success("Starting code generation...");
    println!();

    // Load config
    progress.start_spinner("Loading configuration...");
    let mut config = load_config()?;
    validate_config(&config)?;
    progress.finish_spinner("Configuration loaded");

    use crate::specs::manager::resolve_spec_selection;
    use crate::specs::runner::{run_all_specs, run_single_spec, GenerateOptions};

    // Resolve which specs to generate
    let specs_to_generate = resolve_spec_selection(&config, spec_name.clone(), all_specs)?;

    // Ensure runtime client exists for ALL specs (not just the ones being generated)
    // This fixes the issue where runtime files might be missing if init/add failed to create them
    use crate::generator::writer::{ensure_directory, write_runtime_client};
    for spec in &config.specs {
        let apis_dir = PathBuf::from(&spec.apis.output);
        ensure_directory(&apis_dir)?;
        let runtime_dir = apis_dir.join("runtime");
        if !runtime_dir.exists() {
            write_runtime_client(&apis_dir, Some(&spec.name))?;
            if verbose {
                progress.success(&format!("Created runtime client for {}", spec.name));
            }
        }
    }

    let hook_type = if react_query {
        Some(crate::specs::runner::HookType::ReactQuery)
    } else if swr {
        Some(crate::specs::runner::HookType::Swr)
    } else {
        None
    };

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
        hook_type,
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

    Ok(())
}
