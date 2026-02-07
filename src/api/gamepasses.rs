use anyhow::{Context, Result};
use reqwest::multipart::{Form, Part};
use serde::Deserialize;
use std::fs;

use super::Client;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GamepassResponse {
    pub game_pass_id: u64,
}

#[derive(Debug)]
pub struct UpdateGamepassRequest {
    pub name: Option<String>,
    pub price: Option<u64>,
    pub description: Option<String>,
    pub icon_path: Option<String>,
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

        let mut form = Form::new()
            .text("name", name)
            .text("price", price.to_string());

        if let Some(desc) = description {
            form = form.text("description", desc);
        }

        if let Some(icon_path) = icon_path {
            let icon_bytes = fs::read(&icon_path)
                .with_context(|| format!("Failed to read icon file: {}", icon_path))?;
            let icon_part = Part::bytes(icon_bytes)
                .file_name("icon.png")
                .mime_str("image/png")?;
            form = form.part("iconImageFile", icon_part);
        }

        let response = self
            .http()
            .post(&url)
            .header("x-api-key", self.api_key())
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to create gamepass: {} - {}", status, text);
        }

        response
            .json()
            .await
            .context("Failed to parse gamepass response")
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

        let mut form = Form::new();

        if let Some(name) = request.name {
            form = form.text("name", name);
        }
        if let Some(price) = request.price {
            form = form.text("price", price.to_string());
        }
        if let Some(desc) = request.description {
            form = form.text("description", desc);
        }

        if let Some(icon_path) = request.icon_path {
            let icon_bytes = fs::read(&icon_path)
                .with_context(|| format!("Failed to read icon file: {}", icon_path))?;
            let icon_part = Part::bytes(icon_bytes)
                .file_name("icon.png")
                .mime_str("image/png")?;
            form = form.part("iconImageFile", icon_part);
        }

        let response = self
            .http()
            .patch(&url)
            .header("x-api-key", self.api_key())
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to update gamepass: {} - {}", status, text);
        }

        Ok(())
    }
}
