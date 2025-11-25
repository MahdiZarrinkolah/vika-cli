use crate::config::loader::save_config;
use crate::config::model::Config;
use crate::config::validator::validate_config;
use crate::error::{GenerationError, Result};
use crate::generator::writer::{ensure_directory, write_http_client_template};
use colored::*;
use dialoguer::{Confirm, Input, Select};
use std::path::PathBuf;

pub async fn run() -> Result<()> {
    println!("{}", "Initializing vika-cli project...".bright_cyan());
    println!();

    // Check if config already exists
    let config_path = PathBuf::from(".vika.json");
    if config_path.exists() {
        return Err(GenerationError::InvalidOperation {
            message: ".vika.json already exists. Use 'vika-cli generate' to generate code or 'vika-cli update' to update existing code.".to_string(),
        }.into());
    } else {
        println!(
            "{}",
            "Let's configure your vika-cli preferences:".bright_cyan()
        );
        println!();

        // Global configuration
        println!("{}", "⚙️  Global Configuration".bright_yellow());
        println!();

        let root_dir: String = Input::new()
            .with_prompt("Root directory for generated code")
            .default("src".to_string())
            .interact_text()
            .map_err(|e| GenerationError::InvalidOperation {
                message: format!("Failed to get user input: {}", e),
            })?;

        println!();

        // Generation preferences
        println!("{}", "⚙️  Generation Preferences".bright_yellow());
        println!();

        let enable_cache = Confirm::new()
            .with_prompt("Enable caching for faster regeneration?")
            .default(true)
            .interact()
            .map_err(|e| GenerationError::InvalidOperation {
                message: format!("Failed to get user input: {}", e),
            })?;

        println!();

        let enable_backup = Confirm::new()
            .with_prompt("Enable automatic backups before overwriting files?")
            .default(false)
            .interact()
            .map_err(|e| GenerationError::InvalidOperation {
                message: format!("Failed to get user input: {}", e),
            })?;

        println!();

        let conflict_strategy_options = ["ask", "force", "skip"];
        let conflict_strategy_index = Select::new()
            .with_prompt("What should happen when a file was modified by you?")
            .items(&[
                "ask - Prompt before overwriting (recommended)",
                "force - Always overwrite without asking",
                "skip - Skip modified files",
            ])
            .default(0)
            .interact()
            .map_err(|e| GenerationError::InvalidOperation {
                message: format!("Failed to get user selection: {}", e),
            })?;

        let conflict_strategy = conflict_strategy_options[conflict_strategy_index].to_string();

        println!();

        // Spec configuration
        println!("{}", "📋 Spec Configuration".bright_yellow());
        println!();

        // Collect single spec configuration
        println!("{}", "Enter your OpenAPI specification:".bright_cyan());
        println!();

        let spec_name: String = Input::new()
            .with_prompt("Spec name (kebab-case, e.g., 'ecommerce', 'auth-api')")
            .interact_text()
            .map_err(|e| GenerationError::InvalidOperation {
                message: format!("Failed to get user input: {}", e),
            })?;

        let spec_path_input: String = Input::new()
            .with_prompt(&format!("Path or URL for '{}'", spec_name.trim()))
            .interact_text()
            .map_err(|e| GenerationError::InvalidOperation {
                message: format!("Failed to get user input: {}", e),
            })?;

        println!();
        println!("{}", format!("📋 Configuration for spec '{}'", spec_name.trim()).bright_cyan());
        println!();

        // Per-spec schemas config
        println!("{}", "📁 Schemas Configuration".bright_yellow());
        println!();

        let spec_schemas_output: String = Input::new()
            .with_prompt(&format!("Schemas output directory for '{}'", spec_name.trim()))
            .default(format!("src/schemas/{}", spec_name.trim()))
            .interact_text()
            .map_err(|e| GenerationError::InvalidOperation {
                message: format!("Failed to get user input: {}", e),
            })?;

        println!();

        let spec_naming_options = ["PascalCase", "camelCase", "snake_case", "kebab-case"];
        let spec_naming_index = Select::new()
            .with_prompt(&format!("Schema naming convention for '{}'", spec_name.trim()))
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
            .with_prompt(&format!("APIs output directory for '{}'", spec_name.trim()))
            .default(format!("src/apis/{}", spec_name.trim()))
            .interact_text()
            .map_err(|e| GenerationError::InvalidOperation {
                message: format!("Failed to get user input: {}", e),
            })?;

        println!();

        let spec_api_style_options = ["fetch"];
        let spec_api_style_index = Select::new()
            .with_prompt(&format!("API client style for '{}'", spec_name.trim()))
            .items(&["fetch - Native Fetch API (recommended)"])
            .default(0)
            .interact()
            .map_err(|e| GenerationError::InvalidOperation {
                message: format!("Failed to get user selection: {}", e),
            })?;
        let spec_api_style = spec_api_style_options[spec_api_style_index].to_string();

        println!();

        let spec_base_url_input: String = Input::new()
            .with_prompt(&format!("API base URL for '{}' (optional, press Enter to skip)", spec_name.trim()))
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
            .with_prompt(&format!("Header strategy for '{}'", spec_name.trim()))
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

        // Create config with user preferences
        let config = Config {
            root_dir,
            generation: crate::config::model::GenerationConfig {
                enable_cache,
                enable_backup,
                conflict_strategy,
            },
            specs: vec![crate::config::model::SpecEntry {
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
                },
                modules: crate::config::model::ModulesConfig {
                    ignore: vec![],
                    selected: vec![],
                },
            }],
            ..Config::default()
        };

        validate_config(&config)?;
        save_config(&config)?;
        println!("{}", "✅ Created .vika.json".green());
    }

    // Create directory structure
    let config = crate::config::loader::load_config()?;

    let root_dir = PathBuf::from(&config.root_dir);
    ensure_directory(&root_dir)?;

    // Create directories for each spec
    for spec in &config.specs {
        let schemas_dir = PathBuf::from(&spec.schemas.output);
        ensure_directory(&schemas_dir)?;

        let apis_dir = PathBuf::from(&spec.apis.output);
        ensure_directory(&apis_dir)?;

        // Write http client template for each spec
        let http_client_path = apis_dir.join("http.ts");
        if !http_client_path.exists() {
            write_http_client_template(&http_client_path)?;
            println!(
                "{}",
                format!("✅ Created {} for spec '{}'", http_client_path.display(), spec.name).green()
            );
        } else {
            println!(
                "{}",
                format!(
                    "⚠️  {} already exists for spec '{}'. Skipping.",
                    http_client_path.display(),
                    spec.name
                )
                .yellow()
            );
        }
    }

    println!();
    println!("{}", "✨ Project initialized successfully!".bright_green());
    println!();

    // Automatically trigger generation if specs are provided
    if !config.specs.is_empty() {
        println!("{}", "🚀 Starting code generation...".bright_cyan());
        println!();
        
        // Call generate command internally
        use crate::commands::generate;
        if let Err(e) = generate::run(
            None, // spec - will use config.specs
            false, // all_specs
            None, // spec_name
            false, // verbose
            config.generation.enable_cache, // cache
            config.generation.enable_backup, // backup
            config.generation.conflict_strategy == "force", // force
        )
        .await
        {
            println!();
            println!("{}", "⚠️  Generation failed, but initialization completed.".yellow());
            println!("{}", format!("Error: {}", e).red());
            println!();
            println!("You can run 'vika-cli generate' manually to retry.");
            return Ok(()); // Don't fail init if generation fails
        }
        
        println!();
        println!("{}", "✅ Initialization and generation completed!".bright_green());
        println!();
        println!("💡 To add more specs later, run: vika-cli add");
    } else {
        println!("Next steps:");
        println!("  1. Run: vika-cli generate");
        println!("  2. Select the modules you want to generate");
        println!();
        println!("💡 To add more specs later, run: vika-cli add");
    }

    Ok(())
}
