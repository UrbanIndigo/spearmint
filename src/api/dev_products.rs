use anyhow::{Context, Result};
use reqwest::multipart::Form;
use serde::Deserialize;

use super::Client;

#[derive(Debug)]
pub struct CreateDevProductRequest {
    pub name: String,
    pub price: u64,
    pub description: Option<String>,
}

#[derive(Debug)]
pub struct UpdateDevProductRequest {
    pub name: Option<String>,
    pub price: Option<u64>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevProductResponse {
    pub product_id: u64,
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

        let mut form = Form::new()
            .text("name", request.name)
            .text("price", request.price.to_string());

        if let Some(desc) = request.description {
            form = form.text("description", desc);
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
            anyhow::bail!("Failed to create dev product: {} - {}", status, text);
        }

        response
            .json()
            .await
            .context("Failed to parse dev product response")
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
            anyhow::bail!("Failed to update dev product: {} - {}", status, text);
        }

        Ok(())
    }
}
