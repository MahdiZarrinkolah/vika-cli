use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vika-cli")]
#[command(version = "0.1.0")]
#[command(about = "Generate TypeScript clients from Swagger/OpenAPI specs")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new vika-cli project
    Init,
    /// Generate TypeScript code from Swagger spec
    Generate {
        /// Path or URL to Swagger/OpenAPI spec
        #[arg(short, long)]
        spec: Option<String>,
    },
    /// Update existing generated code
    Update,
}

