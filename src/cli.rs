use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::Path;

use crate::config::{self, DEFAULT_CONFIG_PATH};
use crate::sync::{self, DEFAULT_MAPPING_PATH};
use crate::codegen;
use crate::api::Client;

#[derive(Parser)]
#[command(name = "spearmint")]
#[command(about = "Sync developer products and gamepasses to Roblox")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new spearmint.toml config file
    Init {
        /// Overwrite existing config file
        #[arg(short, long)]
        force: bool,
    },
    /// Sync all products to Roblox (create/update)
    Sync {
        /// Config file path
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
        /// Mapping file path
        #[arg(short, long, default_value = DEFAULT_MAPPING_PATH)]
        mapping: String,
        /// Skip code generation after sync
        #[arg(long = "no-generate", action = clap::ArgAction::SetFalse)]
        generate: bool,
    },
    /// Generate Lua and TypeScript output without syncing
    Generate {
        /// Config file path
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
        /// Mapping file path
        #[arg(short, long, default_value = DEFAULT_MAPPING_PATH)]
        mapping: String,
    },
    /// List current products and their status
    List {
        /// Config file path
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
        /// Mapping file path
        #[arg(short, long, default_value = DEFAULT_MAPPING_PATH)]
        mapping: String,
    },
}

pub fn init(force: bool) -> Result<()> {
    let config_path = Path::new(DEFAULT_CONFIG_PATH);

    if config_path.exists() && !force {
        anyhow::bail!(
            "Config file already exists: {}\nUse --force to overwrite.",
            config_path.display()
        );
    }

    let config = config::create_default();
    config::save(&config, DEFAULT_CONFIG_PATH)?;

    println!("Created config file: {}", config_path.display());
    println!("\nNext steps:");
    println!("1. Edit spearmint.toml with your universe ID and products");
    println!("2. Set ROBLOX_PRODUCTS_API_KEY in your .env file");
    println!("3. Run: spearmint sync");

    Ok(())
}

pub async fn sync(
    config_path: String,
    mapping_path: String,
    generate: bool,
) -> Result<()> {
    let config = config::load(&config_path)?;
    let mut mapping = sync::load_mapping(&mapping_path)?;
    let client = Client::new()?;

    println!("Syncing products for universe {}...\n", config.universe_id);

    let results = sync::sync_all_products(&client, &config, &mut mapping).await?;

    sync::save_mapping(&mapping, &mapping_path)?;
    println!("\nMapping saved to: {}", mapping_path);

    if generate {
        codegen::write_output(&config, &mapping)?;
    }

    let created = results.iter().filter(|r| r.action == "created").count();
    let updated = results.iter().filter(|r| r.action == "updated").count();
    let skipped = results.iter().filter(|r| r.action == "skipped" && r.error.is_none()).count();
    let failed = results.iter().filter(|r| r.error.is_some()).count();

    println!("\nSummary: {} created, {} updated, {} unchanged, {} failed", created, updated, skipped, failed);

    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

pub fn generate(
    config_path: String,
    mapping_path: String,
) -> Result<()> {
    let config = config::load(&config_path)?;
    let mapping = sync::load_mapping(&mapping_path)?;

    codegen::write_output(&config, &mapping)?;

    Ok(())
}

pub fn list(config_path: String, mapping_path: String) -> Result<()> {
    let config = config::load(&config_path)?;
    let mapping = sync::load_mapping(&mapping_path)?;

    println!("Universe ID: {}", config.universe_id);
    println!("\nProducts:");
    println!("{}", "-".repeat(60));

    for (key, product) in &config.products {
        let mapped_id = mapping.get(key).map(|m| m.roblox_id);
        let config_id = product.product_id;
        let roblox_id = config_id.or(mapped_id);

        let status = match roblox_id {
            Some(id) => {
                if config_id.is_some() {
                    format!("ID: {} (from config)", id)
                } else {
                    format!("ID: {} (from mapping)", id)
                }
            }
            None => "Not synced".to_string(),
        };

        println!("  {}", key);
        println!("    Type: {}", product.product_type);
        println!("    Name: {}", product.name);
        println!("    Price: {} Robux", product.price);
        println!("    Status: {}", status);
        println!();
    }

    Ok(())
}
