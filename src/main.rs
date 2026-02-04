use anyhow::Result;
use clap::Parser;
use tracing::{Level, info};
use tracing_subscriber;

mod cli;
mod config;
mod modules;

use cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Bug Bounty Engine");

    // Parse CLI arguments
    let cli = Cli::parse();

    // Execute the appropriate command
    match cli.execute().await {
        Ok(_) => {
            info!("Execution completed successfully");
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
