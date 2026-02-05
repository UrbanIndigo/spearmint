mod cli;
mod config;
mod api;
mod sync;
mod codegen;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { force } => cli::init(force)?,
        Commands::Sync { config, mapping, generate } => {
            cli::sync(config, mapping, generate).await?
        }
        Commands::Generate { config, mapping } => {
            cli::generate(config, mapping)?
        }
        Commands::List { config, mapping } => cli::list(config, mapping)?,
    }

    Ok(())
}
