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
    /// Add a new spec to existing project
    Add,
    /// Generate TypeScript code from Swagger spec
    Generate {
        /// Path or URL to Swagger/OpenAPI spec (for single-spec mode)
        #[arg(short, long)]
        spec: Option<String>,
        /// Generate all specs (for multi-spec mode)
        #[arg(long)]
        all_specs: bool,
        /// Generate specific spec by name (for multi-spec mode)
        #[arg(long)]
        spec_name: Option<String>,
        /// Enable verbose output
        #[arg(long)]
        verbose: bool,
        /// Use cached spec if available (overrides config)
        #[arg(long, action = clap::ArgAction::SetTrue)]
        cache: bool,
        /// Create backup before writing files (overrides config)
        #[arg(long, action = clap::ArgAction::SetTrue)]
        backup: bool,
        /// Force overwrite user-modified files (overrides config)
        #[arg(long, action = clap::ArgAction::SetTrue)]
        force: bool,
        /// Generate React Query hooks
        #[arg(long, action = clap::ArgAction::SetTrue)]
        react_query: bool,
        /// Generate SWR hooks
        #[arg(long, action = clap::ArgAction::SetTrue)]
        swr: bool,
    },
    /// Update existing generated code
    Update,
    /// Inspect OpenAPI spec without generating code
    Inspect {
        /// Path or URL to Swagger/OpenAPI spec (for single-spec mode)
        #[arg(short, long)]
        spec: Option<String>,
        /// Inspect all specs (for multi-spec mode)
        #[arg(long)]
        all_specs: bool,
        /// Inspect specific spec by name (for multi-spec mode)
        #[arg(long)]
        spec_name: Option<String>,
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
    /// Manage templates
    Templates {
        #[command(subcommand)]
        command: TemplateCommands,
    },
}

#[derive(Subcommand)]
pub enum TemplateCommands {
    /// List all available templates
    List,
    /// Initialize templates directory with built-in templates
    Init,
}
