use anyhow::{Context, Result};
use reqwest::multipart::{Form, Part};
use serde::Deserialize;
use std::fs;
use std::time::Duration;
use tokio::time::sleep;

use super::{filename_for_upload, mime_type_for_image, Client, BASE_RETRY_DELAY_MS, MAX_RETRIES};

#[derive(Debug, Clone)]
pub struct CreateDevProductRequest {
    pub name: String,
    pub price: u64,
    pub description: Option<String>,
    pub icon_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateDevProductRequest {
    pub name: Option<String>,
    pub price: Option<u64>,
    pub description: Option<String>,
    pub icon_path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevProductResponse {
    pub product_id: u64,
}

fn build_create_form(request: &CreateDevProductRequest) -> Result<Form> {
    let mut form = Form::new()
        .text("name", request.name.clone())
        .text("price", request.price.to_string());

    if let Some(ref desc) = request.description {
        form = form.text("description", desc.clone());
    }

    if let Some(ref icon_path) = request.icon_path {
        let icon_bytes = fs::read(icon_path)
            .with_context(|| format!("Failed to read icon file: {}", icon_path))?;
        let mime_type = mime_type_for_image(icon_path);
        let filename = filename_for_upload(icon_path);
        let icon_part = Part::bytes(icon_bytes)
            .file_name(filename)
            .mime_str(mime_type)?;
        form = form.part("imageFile", icon_part);
    }

    Ok(form)
}

fn build_update_form(request: &UpdateDevProductRequest) -> Result<Form> {
    let mut form = Form::new();

    if let Some(ref name) = request.name {
        form = form.text("name", name.clone());
    }
    if let Some(price) = request.price {
        form = form.text("price", price.to_string());
    }
    if let Some(ref desc) = request.description {
        form = form.text("description", desc.clone());
    }

    if let Some(ref icon_path) = request.icon_path {
        let icon_bytes = fs::read(icon_path)
            .with_context(|| format!("Failed to read icon file: {}", icon_path))?;
        let mime_type = mime_type_for_image(icon_path);
        let filename = filename_for_upload(icon_path);
        let icon_part = Part::bytes(icon_bytes)
            .file_name(filename)
            .mime_str(mime_type)?;
        form = form.part("imageFile", icon_part);
    }

    Ok(form)
}

impl Client {
    pub async fn create_dev_product(
        &self,
        universe_id: u64,
        request: CreateDevProductRequest,
    ) -> Result<DevProductResponse> {
        let url = format!(
            "https://apis.roblox.com/developer-products/v2/universes/{}/developer-products",
            universe_id
        );

        let mut retries = 0;
        loop {
            let form = build_create_form(&request)?;

            let response = self
                .http()
                .post(&url)
                .header("x-api-key", self.api_key())
                .multipart(form)
                .send()
                .await?;

            if response.status() == 429 && retries < MAX_RETRIES {
                retries += 1;
                let delay = Duration::from_millis(BASE_RETRY_DELAY_MS * (1 << retries));
                eprintln!("  Rate limited, retrying in {:?}...", delay);
                sleep(delay).await;
                continue;
            }

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to create dev product: {} - {}", status, text);
            }

            return response
                .json()
                .await
                .context("Failed to parse dev product response");
        }
    }

    pub async fn update_dev_product(
        &self,
        universe_id: u64,
        product_id: u64,
        request: UpdateDevProductRequest,
    ) -> Result<()> {
        let url = format!(
            "https://apis.roblox.com/developer-products/v2/universes/{}/developer-products/{}",
            universe_id, product_id
        );

        let mut retries = 0;
        loop {
            let form = build_update_form(&request)?;

            let response = self
                .http()
                .patch(&url)
                .header("x-api-key", self.api_key())
                .multipart(form)
                .send()
                .await?;

            if response.status() == 429 && retries < MAX_RETRIES {
                retries += 1;
                let delay = Duration::from_millis(BASE_RETRY_DELAY_MS * (1 << retries));
                eprintln!("  Rate limited, retrying in {:?}...", delay);
                sleep(delay).await;
                continue;
            }

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to update dev product: {} - {}", status, text);
            }

            return Ok(());
        }
    }
}
