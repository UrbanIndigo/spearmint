use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub const DEFAULT_CONFIG_PATH: &str = "spearmint.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub universe_id: u64,
    pub output: Option<OutputConfig>,
    pub products: HashMap<String, Product>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub path: String,
    #[serde(default)]
    pub typescript: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    #[serde(rename = "type")]
    pub product_type: ProductType,
    pub name: String,
    pub price: u64,
    pub description: Option<String>,
    pub image: Option<String>,
    pub product_id: Option<u64>,
    /// If true, the gamepass will be off sale. Defaults to false (on sale).
    /// Only applies to gamepasses, ignored for dev products.
    #[serde(default)]
    pub offsale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProductType {
    DevProduct,
    Gamepass,
}

impl std::fmt::Display for ProductType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProductType::DevProduct => write!(f, "DevProduct"),
            ProductType::Gamepass => write!(f, "Gamepass"),
        }
    }
}

pub fn load(config_path: &str) -> Result<Config> {
    let path = Path::new(config_path);

    if !path.exists() {
        anyhow::bail!("Config file not found: {}", path.display());
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    let config: Config = toml::from_str(&content).with_context(|| "Failed to parse config file")?;

    validate_no_duplicate_names(&config)?;

    Ok(config)
}

fn validate_no_duplicate_names(config: &Config) -> Result<()> {
    let mut dev_product_names: HashMap<&str, &str> = HashMap::new();
    let mut gamepass_names: HashMap<&str, &str> = HashMap::new();

    for (key, product) in &config.products {
        let name_map = match product.product_type {
            ProductType::DevProduct => &mut dev_product_names,
            ProductType::Gamepass => &mut gamepass_names,
        };

        if let Some(existing) = name_map.get(product.name.as_str()) {
            anyhow::bail!(
                "Duplicate {} name \"{}\" found in keys \"{}\" and \"{}\"",
                product.product_type,
                product.name,
                existing,
                key
            );
        }

        name_map.insert(&product.name, key);
    }

    Ok(())
}

pub fn create_default() -> Config {
    let mut products = HashMap::new();

    products.insert(
        "example_product".to_string(),
        Product {
            product_type: ProductType::DevProduct,
            name: "Example Product".to_string(),
            price: 100,
            description: Some("An example developer product".to_string()),
            image: None,
            product_id: None,
            offsale: false,
        },
    );

    products.insert(
        "example_gamepass".to_string(),
        Product {
            product_type: ProductType::Gamepass,
            name: "Example Gamepass".to_string(),
            price: 500,
            description: Some("An example gamepass".to_string()),
            image: None,
            product_id: None,
            offsale: false,
        },
    );

    Config {
        universe_id: 123456789,
        output: Some(OutputConfig {
            path: "src/shared/modules/Products.luau".to_string(),
            typescript: true,
        }),
        products,
    }
}

pub fn save(config: &Config, config_path: &str) -> Result<()> {
    let path = Path::new(config_path);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = toml::to_string_pretty(config)?;
    fs::write(path, content)?;

    Ok(())
}
