use anyhow::Result;
use serde_json::{json, Value};

use crate::api::client::ApiClient;

pub async fn upload_screenshot(screenshot_data: &str) -> Result<Value> {
    let client = ApiClient::new().await?;
    
    // Request presigned upload URL
    let upload_request = json!({
        "image": screenshot_data,
    });

    let response = client.post_with_auth("/api/uploads/request", &upload_request).await?;
    
    log::info!("Upload request status: {}", response.status());
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        log::error!("Upload request failed: {} - {}", status, error_text);
        return Err(anyhow::anyhow!("Failed to request upload: {} - {}", status, error_text));
    }

    let upload_data: Value = response.json().await?;
    
    log::info!("Upload request response: {}", serde_json::to_string_pretty(&upload_data)?);
    
    Ok(json!({
        "publicId": upload_data["publicId"],
        "secureUrl": upload_data["secureUrl"],
        "width": upload_data["width"],
        "height": upload_data["height"],
        "bytes": upload_data["bytes"],
        "format": upload_data["format"],
        "createdAt": upload_data["createdAt"]
    }))
}
