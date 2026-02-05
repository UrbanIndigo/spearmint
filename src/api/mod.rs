mod dev_products;
mod gamepasses;

pub use dev_products::*;
pub use gamepasses::*;

use anyhow::{Context, Result};

pub struct Client {
    http: reqwest::Client,
    api_key: String,
}

impl Client {
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("ROBLOX_PRODUCTS_API_KEY")
            .context("ROBLOX_PRODUCTS_API_KEY environment variable not set")?;

        let http = reqwest::Client::builder().build()?;

        Ok(Self { http, api_key })
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }
}
