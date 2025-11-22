use crate::config::loader::save_config;
use crate::config::model::Config;
use crate::config::validator::validate_config;
use crate::error::Result;
use crate::generator::writer::{ensure_directory, write_http_client_template};
use colored::*;
use std::path::PathBuf;

pub fn run() -> Result<()> {
    println!("{}", "Initializing vika-cli project...".bright_cyan());

    // Check if config already exists
    let config_path = PathBuf::from(".vika.json");
    if config_path.exists() {
        println!(
            "{}",
            "⚠️  .vika.json already exists. Skipping config creation.".yellow()
        );
    } else {
        let config = Config::default();
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
