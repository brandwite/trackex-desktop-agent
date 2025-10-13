use anyhow::Result;
use serde_json::{json, Value};
use base64::{self, Engine};

use crate::api::client::ApiClient;

pub async fn upload_screenshot(screenshot_data: &str) -> Result<Value> {
    let client = ApiClient::new().await?;
    
    // Request presigned upload URL
    let upload_request = json!({
        "contentType": "image/jpeg",
        "purpose": "screenshot"
    });

    let response = client.post_with_auth("/api/uploads/request", &upload_request).await?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to request upload URL: {}", response.status()));
    }

    let upload_data: Value = response.json().await?;
    
    let presigned_url = upload_data["presignedUrl"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No presigned URL in response"))?;

    let storage_key = upload_data["storageKey"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No storage key in response"))?;

    // Decode base64 screenshot data
    let image_data = base64::engine::general_purpose::STANDARD.decode(screenshot_data)?;

    // Upload to presigned URL
    let upload_response = client.upload_file(presigned_url, &image_data, "image/jpeg").await?;
    
    if !upload_response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to upload screenshot: {}", upload_response.status()));
    }

    
    Ok(json!({
        "storageKey": storage_key,
        "size": image_data.len()
    }))
}
