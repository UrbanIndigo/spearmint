use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::api::{Client, CreateDevProductRequest, UpdateDevProductRequest, UpdateGamepassRequest};
use crate::config::{Config, Product, ProductType};

pub const DEFAULT_MAPPING_PATH: &str = "spearmint.lock.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingEntry {
    pub roblox_id: u64,
    pub name: Option<String>,
    pub price: Option<u64>,
    pub description: Option<String>,
    pub image_hash: Option<String>,
}

pub type Mapping = HashMap<String, MappingEntry>;

#[derive(Debug)]
pub struct SyncResult {
    pub action: String,
    pub error: Option<String>,
}

pub fn load_mapping(mapping_path: &str) -> Result<Mapping> {
    let path = Path::new(mapping_path);

    if !path.exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read mapping file: {}", path.display()))?;

    toml::from_str(&content).with_context(|| "Failed to parse mapping file")
}

pub fn save_mapping(mapping: &Mapping, mapping_path: &str) -> Result<()> {
    let path = Path::new(mapping_path);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = toml::to_string_pretty(mapping)?;
    fs::write(path, content)?;

    Ok(())
}

pub async fn sync_all_products(
    client: &Client,
    config: &Config,
    mapping: &mut Mapping,
) -> Result<Vec<SyncResult>> {
    let mut results = Vec::new();

    for (key, product) in &config.products {
        let result = sync_product(client, config.universe_id, key, product, mapping).await;

        match result {
            Ok(action) => {
                println!("[{}] {} - {}", action, product.product_type, key);
                results.push(SyncResult {
                    action,
                    error: None,
                });
            }
            Err(e) => {
                println!("[ERROR] {} - {}: {}", product.product_type, key, e);
                results.push(SyncResult {
                    action: "error".to_string(),
                    error: Some(e.to_string()),
                });
            }
        }
    }

    Ok(results)
}

async fn sync_product(
    client: &Client,
    universe_id: u64,
    key: &str,
    product: &Product,
    mapping: &mut Mapping,
) -> Result<String> {
    let existing_id = product
        .product_id
        .or_else(|| mapping.get(key).map(|m| m.roblox_id));

    match product.product_type {
        ProductType::DevProduct => {
            sync_dev_product(client, universe_id, key, product, existing_id, mapping).await
        }
        ProductType::Gamepass => {
            sync_gamepass(client, universe_id, key, product, existing_id, mapping).await
        }
    }
}

fn hash_file(path: &str) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let hash = Sha256::digest(&bytes);
    Some(hex::encode(hash))
}

fn image_hash(product: &Product) -> Option<String> {
    product.image.as_deref().and_then(hash_file)
}

fn config_changed(product: &Product, entry: &MappingEntry) -> bool {
    entry.name.as_deref() != Some(&product.name)
        || entry.price != Some(product.price)
        || entry.description != product.description
        || image_hash(product) != entry.image_hash
}

fn update_mapping_entry(entry: &mut MappingEntry, product: &Product) {
    entry.name = Some(product.name.clone());
    entry.price = Some(product.price);
    entry.description = product.description.clone();
    entry.image_hash = image_hash(product);
}

async fn sync_dev_product(
    client: &Client,
    universe_id: u64,
    key: &str,
    product: &Product,
    existing_id: Option<u64>,
    mapping: &mut Mapping,
) -> Result<String> {
    match existing_id {
        Some(id) => {
            // Check locally if the config has changed since last sync
            if let Some(entry) = mapping.get(key) {
                if !config_changed(product, entry) {
                    return Ok("skipped".to_string());
                }
            }

            // Only include icon if it has changed
            let icon_path = if let Some(ref image) = product.image {
                let new_hash = hash_file(image);
                let old_hash = mapping.get(key).and_then(|e| e.image_hash.clone());
                if new_hash != old_hash {
                    Some(image.clone())
                } else {
                    None
                }
            } else {
                None
            };

            client
                .update_dev_product(
                    universe_id,
                    id,
                    UpdateDevProductRequest {
                        name: Some(product.name.clone()),
                        price: Some(product.price),
                        description: product.description.clone(),
                        icon_path,
                    },
                )
                .await?;

            let entry = mapping.entry(key.to_string()).or_insert(MappingEntry {
                roblox_id: id,
                name: None,
                price: None,
                description: None,
                image_hash: None,
            });
            update_mapping_entry(entry, product);

            Ok("updated".to_string())
        }
        None => {
            let response = client
                .create_dev_product(
                    universe_id,
                    CreateDevProductRequest {
                        name: product.name.clone(),
                        price: product.price,
                        description: product.description.clone(),
                        icon_path: product.image.clone(),
                    },
                )
                .await?;

            mapping.insert(
                key.to_string(),
                MappingEntry {
                    roblox_id: response.product_id,
                    name: Some(product.name.clone()),
                    price: Some(product.price),
                    description: product.description.clone(),
                    image_hash: image_hash(product),
                },
            );

            Ok("created".to_string())
        }
    }
}

async fn sync_gamepass(
    client: &Client,
    universe_id: u64,
    key: &str,
    product: &Product,
    existing_id: Option<u64>,
    mapping: &mut Mapping,
) -> Result<String> {
    match existing_id {
        Some(id) => {
            // Check locally if the config has changed since last sync
            if let Some(entry) = mapping.get(key) {
                if !config_changed(product, entry) {
                    return Ok("skipped".to_string());
                }
            }

            // Only include icon if it has changed
            let icon_path = if let Some(ref image) = product.image {
                let new_hash = hash_file(image);
                let old_hash = mapping.get(key).and_then(|e| e.image_hash.clone());
                if new_hash != old_hash {
                    Some(image.clone())
                } else {
                    None
                }
            } else {
                None
            };

            client
                .update_gamepass(
                    universe_id,
                    id,
                    UpdateGamepassRequest {
                        name: Some(product.name.clone()),
                        price: Some(product.price),
                        description: product.description.clone(),
                        icon_path,
                    },
                )
                .await?;

            let entry = mapping.entry(key.to_string()).or_insert(MappingEntry {
                roblox_id: id,
                name: None,
                price: None,
                description: None,
                image_hash: None,
            });
            update_mapping_entry(entry, product);

            Ok("updated".to_string())
        }
        None => {
            let response = client
                .create_gamepass(
                    universe_id,
                    product.name.clone(),
                    product.price,
                    product.description.clone(),
                    product.image.clone(),
                )
                .await?;

            mapping.insert(
                key.to_string(),
                MappingEntry {
                    roblox_id: response.game_pass_id,
                    name: Some(product.name.clone()),
                    price: Some(product.price),
                    description: product.description.clone(),
                    image_hash: image_hash(product),
                },
            );

            Ok("created".to_string())
        }
    }
}
