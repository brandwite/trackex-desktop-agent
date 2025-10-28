use anyhow::Result;
use reqwest::{Client, Response};
use serde_json::Value;
use std::time::Duration;

use crate::storage::secure_store;

use std::env;

pub struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    pub async fn new() -> Result<Self> {
        

        let base_url = crate::storage::get_server_url().await?;

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(format!("TrackEx-Agent/{}", env!("CARGO_PKG_VERSION")))
            .build()?;

        Ok(Self { client, base_url })
    }

    pub async fn get_with_auth(&self, endpoint: &str) -> Result<Response> {
        let device_token = crate::storage::get_device_token().await
            .map_err(|_| anyhow::anyhow!("No device token available"))?;
        log::info!("Device token: {}", device_token);
        let device_id = crate::storage::get_device_id().await
            .map_err(|_| anyhow::anyhow!("No device ID available"))?;
        let url = format!("{}{}", self.base_url, endpoint);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", device_token))
            .header("X-Device-ID", device_id.clone())
            .header("Content-Type", "application/json")
            .send()
            .await?;

        Ok(response)
    }

    pub async fn post_with_auth(&self, endpoint: &str, body: &Value) -> Result<Response> {
        let device_token = crate::storage::get_device_token().await
            .map_err(|_| anyhow::anyhow!("No device token available"))?;
        let device_id = crate::storage::get_device_id().await
            .map_err(|_| anyhow::anyhow!("No device ID available"))?;
        let url = format!("{}{}", self.base_url, endpoint);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", device_token))
            .header("X-Device-ID", device_id.clone())
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        Ok(response)
    }

    #[allow(dead_code)]
    pub async fn post(&self, endpoint: &str, body: &Value) -> Result<Response> {
        let url = format!("{}{}", self.base_url, endpoint);

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        Ok(response)
    }

    #[allow(dead_code)]
    pub async fn put_with_auth(&self, endpoint: &str, body: &Value) -> Result<Response> {
        let device_token = secure_store::get_device_token().await?
            .ok_or_else(|| anyhow::anyhow!("No device token available"))?;
        let device_id = crate::storage::get_device_id().await
            .map_err(|_| anyhow::anyhow!("No device ID available"))?;
        let url = format!("{}{}", self.base_url, endpoint);

        let response = self.client
            .put(&url)
            .header("Authorization", format!("Bearer {}", device_token))
            .header("X-Device-ID", device_id.clone())
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        Ok(response)
    }
}

