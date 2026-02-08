use anyhow::{Context, Result};
use reqwest::multipart::{Form, Part};
use serde::Deserialize;
use std::fs;
use std::time::Duration;
use tokio::time::sleep;

use super::{filename_for_upload, mime_type_for_image, Client, BASE_RETRY_DELAY_MS, MAX_RETRIES};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GamepassResponse {
    pub game_pass_id: u64,
}

#[derive(Debug, Clone)]
pub struct UpdateGamepassRequest {
    pub name: Option<String>,
    pub price: Option<u64>,
    pub description: Option<String>,
    pub icon_path: Option<String>,
}

fn build_create_form(
    name: &str,
    price: u64,
    description: &Option<String>,
    icon_path: &Option<String>,
) -> Result<Form> {
    let mut form = Form::new()
        .text("name", name.to_string())
        .text("price", price.to_string());

    if let Some(ref desc) = description {
        form = form.text("description", desc.clone());
    }

    if let Some(ref icon_path) = icon_path {
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

fn build_update_form(request: &UpdateGamepassRequest) -> Result<Form> {
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
    pub async fn create_gamepass(
        &self,
        universe_id: u64,
        name: String,
        price: u64,
        description: Option<String>,
        icon_path: Option<String>,
    ) -> Result<GamepassResponse> {
        let url = format!(
            "https://apis.roblox.com/game-passes/v1/universes/{}/game-passes",
            universe_id
        );

        let mut retries = 0;
        loop {
            let form = build_create_form(&name, price, &description, &icon_path)?;

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
                anyhow::bail!("Failed to create gamepass: {} - {}", status, text);
            }

            return response
                .json()
                .await
                .context("Failed to parse gamepass response");
        }
    }

    pub async fn update_gamepass(
        &self,
        universe_id: u64,
        gamepass_id: u64,
        request: UpdateGamepassRequest,
    ) -> Result<()> {
        let url = format!(
            "https://apis.roblox.com/game-passes/v1/universes/{}/game-passes/{}",
            universe_id, gamepass_id
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
                anyhow::bail!("Failed to update gamepass: {} - {}", status, text);
            }

            return Ok(());
        }
    }
}
