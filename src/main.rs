use clap::Parser;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = vika_cli::cli::Cli::parse();

    match cli.command {
        vika_cli::cli::Commands::Init => {
            if let Err(e) = vika_cli::commands::init::run() {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        vika_cli::cli::Commands::Generate {
            spec,
            verbose,
            cache,
            backup,
            force,
        } => {
            if let Err(e) =
                vika_cli::commands::generate::run(spec, verbose, cache, backup, force).await
            {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        vika_cli::cli::Commands::Update => {
            if let Err(e) = vika_cli::commands::update::run().await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        vika_cli::cli::Commands::Inspect {
            spec,
            module,
            schemas,
            graph,
            json,
        } => {
            if let Err(e) =
                vika_cli::commands::inspect::run(spec, module, schemas, graph, json).await
            {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        vika_cli::cli::Commands::Templates { command } => {
            match command {
                vika_cli::cli::TemplateCommands::List => {
                    if let Err(e) = vika_cli::commands::templates::list() {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
                vika_cli::cli::TemplateCommands::Init => {
                    if let Err(e) = vika_cli::commands::templates::init() {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}
