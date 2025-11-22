use clap::Parser;
use tracing_subscriber;

mod cache;
mod cli;
mod commands;
mod config;
mod error;
mod formatter;
mod generator;
mod progress;

use cli::Cli;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        cli::Commands::Init => {
            if let Err(e) = commands::init::run() {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        cli::Commands::Generate { spec, verbose, cache, backup, force } => {
            if let Err(e) = commands::generate::run(spec, verbose, cache, backup, force).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        cli::Commands::Update => {
            if let Err(e) = commands::update::run().await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        cli::Commands::Inspect { spec, module, schemas, graph, json } => {
            if let Err(e) = commands::inspect::run(spec, module, schemas, graph, json).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
