use anyhow::Result;
use serde_json::json;
use uuid::Uuid;

use crate::api::client::ApiClient;

pub async fn register_device(server_url: &str, email: &str, password: &str) -> Result<(String, String)> {
    // Create a temporary client for registration
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent(format!("TrackEx-Agent/{}", env!("CARGO_PKG_VERSION")))
        .build()?;

    // First, authenticate to get user token
    let auth_response = client
        .post(&format!("{}/api/auth/login", server_url))
        .json(&json!({
            "email": email,
            "password": password
        }))
        .send()
        .await?;

    if !auth_response.status().is_success() {
        return Err(anyhow::anyhow!("Authentication failed: {}", auth_response.status()));
    }

    let auth_data: serde_json::Value = auth_response.json().await?;
    let user_token = auth_data["token"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No token in auth response"))?;

    // Generate device info
    let device_id = Uuid::new_v4().to_string();
    let device_name = format!("macOS-{}", 
        std::env::var("USER").unwrap_or_else(|_| "unknown".to_string())
    );

    // Register device
    let device_data = json!({
        "deviceId": device_id,
        "name": device_name,
        "platform": std::env::consts::OS,
        "version": std::env::consts::ARCH,
        "agent_version": env!("CARGO_PKG_VERSION")
    });

    let device_response = client
        .post(&format!("{}/api/devices/register", server_url))
        .header("Authorization", format!("Bearer {}", user_token))
        .json(&device_data)
        .send()
        .await?;

    if !device_response.status().is_success() {
        return Err(anyhow::anyhow!("Device registration failed: {}", device_response.status()));
    }

    let device_result: serde_json::Value = device_response.json().await?;
    let device_token = device_result["deviceToken"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No device token in response"))?;

    
    Ok((device_token.to_string(), device_id))
}

