use clap::Parser;
use tracing_subscriber;

mod cli;
mod commands;
mod config;
mod generator;

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
        cli::Commands::Generate { spec } => {
            if let Err(e) = commands::generate::run(spec).await {
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
    }
}
