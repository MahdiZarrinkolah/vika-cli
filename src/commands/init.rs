use crate::config::loader::save_config;
use crate::config::model::Config;
use crate::config::validator::validate_config;
use crate::error::{GenerationError, Result};
use crate::generator::writer::{ensure_directory, write_http_client_template};
use colored::*;
use dialoguer::{Confirm, Input, Select};
use std::path::PathBuf;

pub fn run() -> Result<()> {
    println!("{}", "Initializing vika-cli project...".bright_cyan());
    println!();

    // Check if config already exists
    let config_path = PathBuf::from(".vika.json");
    if config_path.exists() {
        println!(
            "{}",
            "⚠️  .vika.json already exists. Skipping config creation.".yellow()
        );
    } else {
        println!("{}", "Let's configure your vika-cli preferences:".bright_cyan());
        println!();

        // Paths configuration
        println!("{}", "📁 Paths Configuration".bright_yellow());
        println!();

        let root_dir: String = Input::new()
            .with_prompt("Root directory for generated code")
            .default("src".to_string())
            .interact_text()
            .map_err(|e| GenerationError::InvalidOperation {
                message: format!("Failed to get user input: {}", e),
            })?;

        println!();

        let schemas_output: String = Input::new()
            .with_prompt("Schemas output directory")
            .default("src/schemas".to_string())
            .interact_text()
            .map_err(|e| GenerationError::InvalidOperation {
                message: format!("Failed to get user input: {}", e),
            })?;

        println!();

        let apis_output: String = Input::new()
            .with_prompt("APIs output directory")
            .default("src/apis".to_string())
            .interact_text()
            .map_err(|e| GenerationError::InvalidOperation {
                message: format!("Failed to get user input: {}", e),
            })?;

        println!();

        // Schema naming convention
        let naming_options = vec!["PascalCase", "camelCase", "snake_case", "kebab-case"];
        let naming_index = Select::new()
            .with_prompt("Schema naming convention")
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

        let naming = naming_options[naming_index].to_string();

        println!();

        // API configuration
        println!("{}", "🔌 API Configuration".bright_yellow());
        println!();

        let api_style_options = vec!["fetch"];
        let api_style_index = Select::new()
            .with_prompt("API client style")
            .items(&["fetch - Native Fetch API (recommended)"])
            .default(0)
            .interact()
            .map_err(|e| GenerationError::InvalidOperation {
                message: format!("Failed to get user selection: {}", e),
            })?;

        let api_style = api_style_options[api_style_index].to_string();

        println!();

        let base_url_input: String = Input::new()
            .with_prompt("API base URL (optional, press Enter to skip)")
            .allow_empty(true)
            .interact_text()
            .map_err(|e| GenerationError::InvalidOperation {
                message: format!("Failed to get user input: {}", e),
            })?;

        let base_url = if base_url_input.trim().is_empty() {
            None
        } else {
            Some(base_url_input.trim().to_string())
        };

        println!();

        let header_strategy_options = vec!["consumerInjected", "bearerToken", "fixed"];
        let header_strategy_index = Select::new()
            .with_prompt("Header strategy for API requests")
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

        let header_strategy = header_strategy_options[header_strategy_index].to_string();

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

        let conflict_strategy_options = vec!["ask", "force", "skip"];
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

        // Create config with user preferences
        let mut config = Config::default();
        config.root_dir = root_dir;
        config.schemas.output = schemas_output;
        config.schemas.naming = naming;
        config.apis.output = apis_output;
        config.apis.style = api_style;
        config.apis.base_url = base_url;
        config.apis.header_strategy = header_strategy;
        config.generation.enable_cache = enable_cache;
        config.generation.enable_backup = enable_backup;
        config.generation.conflict_strategy = conflict_strategy;

        validate_config(&config)?;
        save_config(&config)?;
        println!("{}", "✅ Created .vika.json".green());
    }

    // Create directory structure
    let config = crate::config::loader::load_config()?;

    let root_dir = PathBuf::from(&config.root_dir);
    ensure_directory(&root_dir)?;

    let schemas_dir = PathBuf::from(&config.schemas.output);
    ensure_directory(&schemas_dir)?;

    let apis_dir = PathBuf::from(&config.apis.output);
    ensure_directory(&apis_dir)?;

    // Write http client template
    let http_client_path = apis_dir.join("http.ts");
    if !http_client_path.exists() {
        write_http_client_template(&http_client_path)?;
        println!(
            "{}",
            format!("✅ Created {}", http_client_path.display()).green()
        );
    } else {
        println!(
            "{}",
            format!(
                "⚠️  {} already exists. Skipping.",
                http_client_path.display()
            )
            .yellow()
        );
    }

    println!();
    println!("{}", "✨ Project initialized successfully!".bright_green());
    println!();
    println!("Next steps:");
    println!("  1. Run: vika-cli generate --spec <path-or-url-to-swagger>");
    println!("  2. Select the modules you want to generate");

    Ok(())
}
