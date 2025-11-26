use crate::config::loader::{load_config, save_config};
use crate::config::validator::validate_config;
use crate::error::{GenerationError, Result};
use crate::generator::writer::{ensure_directory, write_runtime_client};
use colored::*;
use dialoguer::{Confirm, Input, Select};
use std::path::PathBuf;

pub async fn run() -> Result<()> {
    println!("{}", "Adding new spec to vika-cli project...".bright_cyan());
    println!();

    // Check if config exists
    let config_path = PathBuf::from(".vika.json");
    if !config_path.exists() {
        return Err(GenerationError::InvalidOperation {
            message: ".vika.json not found. Please run 'vika-cli init' first.".to_string(),
        }
        .into());
    }

    // Load existing config
    let mut config = load_config()?;
    validate_config(&config)?;

    println!("{}", "Let's configure your new spec:".bright_cyan());
    println!();

    // Collect spec details
    let spec_name: String = Input::new()
        .with_prompt("Spec name (kebab-case recommended, e.g., 'ecommerce', 'auth-api')")
        .validate_with(|input: &String| -> std::result::Result<(), String> {
            if input.trim().is_empty() {
                return Err("Spec name cannot be empty".to_string());
            }
            // Check if spec name already exists
            if config.specs.iter().any(|s| s.name == input.trim()) {
                return Err(format!("Spec '{}' already exists", input.trim()));
            }
            Ok(())
        })
        .interact_text()
        .map_err(|e| GenerationError::InvalidOperation {
            message: format!("Failed to get user input: {}", e),
        })?;

    let spec_path_input: String = Input::new()
        .with_prompt(format!("Path or URL for '{}'", spec_name.trim()))
        .interact_text()
        .map_err(|e| GenerationError::InvalidOperation {
            message: format!("Failed to get user input: {}", e),
        })?;

    println!();
    println!(
        "{}",
        format!("📋 Configuration for spec '{}'", spec_name.trim()).bright_cyan()
    );
    println!();

    // Per-spec schemas config
    println!("{}", "📁 Schemas Configuration".bright_yellow());
    println!();

    let spec_schemas_output: String = Input::new()
        .with_prompt(format!(
            "Schemas output directory for '{}'",
            spec_name.trim()
        ))
        .default(format!("src/schemas/{}", spec_name.trim()))
        .interact_text()
        .map_err(|e| GenerationError::InvalidOperation {
            message: format!("Failed to get user input: {}", e),
        })?;

    println!();

    let spec_naming_options = ["PascalCase", "camelCase", "snake_case", "kebab-case"];
    let spec_naming_index = Select::new()
        .with_prompt(format!(
            "Schema naming convention for '{}'",
            spec_name.trim()
        ))
        .items(&[
            "PascalCase - ProductDto, UserProfile (recommended)",
            "camelCase - productDto, userProfile",
            "snake_case - product_dto, user_profile",
            "kebab-case - product-dto, user-profile",
        ])
        .default(0)
        .interact()
        .map_err(|e| GenerationError::InvalidOperation {
            message: format!("Failed to get user selection: {}", e),
        })?;
    let spec_naming = spec_naming_options[spec_naming_index].to_string();

    println!();

    // Per-spec APIs config
    println!("{}", "🔌 API Configuration".bright_yellow());
    println!();

    let spec_apis_output: String = Input::new()
        .with_prompt(format!("APIs output directory for '{}'", spec_name.trim()))
        .default(format!("src/apis/{}", spec_name.trim()))
        .interact_text()
        .map_err(|e| GenerationError::InvalidOperation {
            message: format!("Failed to get user input: {}", e),
        })?;

    println!();

    let spec_api_style_options = ["fetch"];
    let spec_api_style_index = Select::new()
        .with_prompt(format!("API client style for '{}'", spec_name.trim()))
        .items(&["fetch - Native Fetch API (recommended)"])
        .default(0)
        .interact()
        .map_err(|e| GenerationError::InvalidOperation {
            message: format!("Failed to get user selection: {}", e),
        })?;
    let spec_api_style = spec_api_style_options[spec_api_style_index].to_string();

    println!();

    let spec_base_url_input: String = Input::new()
        .with_prompt(format!(
            "API base URL for '{}' (optional, press Enter to skip)",
            spec_name.trim()
        ))
        .allow_empty(true)
        .interact_text()
        .map_err(|e| GenerationError::InvalidOperation {
            message: format!("Failed to get user input: {}", e),
        })?;

    let spec_base_url = if spec_base_url_input.trim().is_empty() {
        None
    } else {
        Some(spec_base_url_input.trim().to_string())
    };

    println!();

    let spec_header_strategy_options = ["consumerInjected", "bearerToken", "fixed"];
    let spec_header_strategy_index = Select::new()
        .with_prompt(format!("Header strategy for '{}'", spec_name.trim()))
        .items(&[
            "consumerInjected - Headers provided by consumer (recommended)",
            "bearerToken - Automatic Bearer token injection",
            "fixed - Fixed headers from config",
        ])
        .default(0)
        .interact()
        .map_err(|e| GenerationError::InvalidOperation {
            message: format!("Failed to get user selection: {}", e),
        })?;
    let spec_header_strategy = spec_header_strategy_options[spec_header_strategy_index].to_string();

    println!();

    // Hooks configuration
    println!("{}", "📎 Hooks Configuration".bright_yellow());
    println!();

    let enable_hooks = Confirm::new()
        .with_prompt("Do you want to generate hooks (React Query or SWR)?")
        .default(false)
        .interact()
        .map_err(|e| GenerationError::InvalidOperation {
            message: format!("Failed to get user input: {}", e),
        })?;

    let hooks_config = if enable_hooks {
        let hook_library_options = ["react-query", "swr"];
        let hook_library_index = Select::new()
            .with_prompt("Which hook library do you want to use?")
            .items(&hook_library_options)
            .default(0)
            .interact()
            .map_err(|e| GenerationError::InvalidOperation {
                message: format!("Failed to get user selection: {}", e),
            })?;
        let hook_library = hook_library_options[hook_library_index].to_string();

        let hooks_output: String = Input::new()
            .with_prompt("Hooks output directory")
            .default("src/hooks".to_string())
            .interact_text()
            .map_err(|e| GenerationError::InvalidOperation {
                message: format!("Failed to get user input: {}", e),
            })?;

        let query_keys_output: String = Input::new()
            .with_prompt("Query keys output directory")
            .default("src/query-keys".to_string())
            .interact_text()
            .map_err(|e| GenerationError::InvalidOperation {
                message: format!("Failed to get user input: {}", e),
            })?;

        Some(crate::config::model::HooksConfig {
            output: hooks_output.trim().to_string(),
            query_keys_output: query_keys_output.trim().to_string(),
            library: Some(hook_library),
        })
    } else {
        None
    };

    println!();

    // Create the new spec entry
    let new_spec = crate::config::model::SpecEntry {
        name: spec_name.trim().to_string(),
        path: spec_path_input.trim().to_string(),
        schemas: crate::config::model::SchemasConfig {
            output: spec_schemas_output.trim().to_string(),
            naming: spec_naming,
        },
        apis: crate::config::model::ApisConfig {
            output: spec_apis_output.trim().to_string(),
            style: spec_api_style,
            base_url: spec_base_url,
            header_strategy: spec_header_strategy,
            timeout: None,
            retries: None,
            retry_delay: None,
            headers: None,
        },
        hooks: hooks_config,
        modules: crate::config::model::ModulesConfig {
            ignore: vec![],
            selected: vec![],
        },
    };

    // Add spec to config
    config.specs.push(new_spec.clone());

    // Validate updated config
    validate_config(&config)?;

    // Save config
    save_config(&config)?;
    println!("{}", "✅ Added spec to .vika.json".green());
    println!();

    // Create directory structure
    let schemas_dir = PathBuf::from(&new_spec.schemas.output);
    ensure_directory(&schemas_dir)?;

    let apis_dir = PathBuf::from(&new_spec.apis.output);
    ensure_directory(&apis_dir)?;

    // Write runtime client at root_dir (shared across all specs)
    let root_dir_path = PathBuf::from(&config.root_dir);
    ensure_directory(&root_dir_path)?;
    let runtime_dir = root_dir_path.join("runtime");
    if !runtime_dir.exists() {
        write_runtime_client(&root_dir_path, None, Some(&new_spec.apis))?;
        println!(
            "{}",
            format!("✅ Created runtime client for spec '{}'", new_spec.name).green()
        );
    } else {
        println!(
            "{}",
            format!(
                "⚠️  Runtime client already exists for spec '{}'. Skipping.",
                new_spec.name
            )
            .yellow()
        );
    }

    println!();
    println!("{}", "✨ Spec added successfully!".bright_green());
    println!();

    // Ask if user wants to generate code now
    let generate_now = Confirm::new()
        .with_prompt("Generate code for this spec now?")
        .default(true)
        .interact()
        .map_err(|e| GenerationError::InvalidOperation {
            message: format!("Failed to get user input: {}", e),
        })?;

    if generate_now {
        println!();
        println!("{}", "🚀 Starting code generation...".bright_cyan());
        println!();

        // Call generate command for this specific spec
        use crate::commands::generate;
        if let Err(e) = generate::run(
            None,                                           // spec - will use spec_name
            false,                                          // all_specs
            Some(new_spec.name.clone()),                    // spec_name
            false,                                          // verbose
            config.generation.enable_cache,                 // cache
            config.generation.enable_backup,                // backup
            config.generation.conflict_strategy == "force", // force
            false,                                          // react_query
            false,                                          // swr
        )
        .await
        {
            println!();
            println!(
                "{}",
                "⚠️  Generation failed, but spec was added successfully.".yellow()
            );
            println!("{}", format!("Error: {}", e).red());
            println!();
            println!(
                "You can run 'vika-cli generate --spec-name {}' manually to retry.",
                new_spec.name
            );
            return Ok(()); // Don't fail add if generation fails
        }

        println!();
        println!("{}", "✅ Spec added and code generated!".bright_green());
    } else {
        println!("Next steps:");
        println!("  Run: vika-cli generate --spec-name {}", new_spec.name);
    }

    Ok(())
}
