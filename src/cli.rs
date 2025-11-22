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
        /// Enable verbose output
        #[arg(long)]
        verbose: bool,
        /// Use cached spec if available
        #[arg(long)]
        cache: bool,
        /// Create backup before writing files
        #[arg(long)]
        backup: bool,
        /// Force overwrite user-modified files
        #[arg(long)]
        force: bool,
    },
    /// Update existing generated code
    Update,
    /// Inspect OpenAPI spec without generating code
    Inspect {
        /// Path or URL to Swagger/OpenAPI spec
        #[arg(short, long)]
        spec: Option<String>,
        /// Show details for specific module
        #[arg(short, long)]
        module: Option<String>,
        /// Show schema details
        #[arg(long)]
        schemas: bool,
        /// Show dependency graph
        #[arg(long)]
        graph: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}
