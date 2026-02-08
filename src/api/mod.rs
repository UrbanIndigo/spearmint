mod dev_products;
mod gamepasses;

pub use dev_products::*;
pub use gamepasses::*;

use anyhow::{Context, Result};
use std::path::Path;

/// Maximum number of retries on rate limit
const MAX_RETRIES: u32 = 5;
/// Base delay between retries (doubles each time)
const BASE_RETRY_DELAY_MS: u64 = 500;

/// Detect MIME type from file extension
pub fn mime_type_for_image(path: &str) -> &'static str {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match ext.as_deref() {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        _ => "image/png", // Default fallback
    }
}

/// Get filename from path for upload
pub fn filename_for_upload(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("icon.png")
        .to_string()
}

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
