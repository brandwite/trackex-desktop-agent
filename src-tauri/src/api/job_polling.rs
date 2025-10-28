use anyhow::Result;
use tauri::AppHandle;
use tokio::time::{sleep, Duration};
use serde_json::Value;
use serde_json::json;

use crate::api::client::ApiClient;
use crate::screenshots::screen_capture;

pub async fn start_job_polling(_app_handle: AppHandle) {
    let interval_seconds = crate::sampling::get_job_polling_interval();

    let mut interval = tokio::time::interval(Duration::from_secs(interval_seconds));
    let mut last_cursor: Option<String> = None;
    
    loop {
        // Check if services should continue running (authenticated AND clocked in)
        if !crate::sampling::should_services_run().await {
            // Stop if user is not authenticated or not clocked in
            if !crate::sampling::is_services_running().await {
                break; // Service stopped completely
            }
            // Otherwise, just wait before checking again
            interval.tick().await;
            continue;
        }

        // Poll for jobs (only when authenticated and clocked in)
        if let Err(e) = poll_jobs(&mut last_cursor).await {
            log::error!("Failed to poll jobs: {}", e);
            // Wait a bit before retrying on error
            sleep(Duration::from_secs(10)).await;
        }

        interval.tick().await;
    }

}

async fn poll_jobs(last_cursor: &mut Option<String>) -> Result<()> {
    let client = ApiClient::new().await?;
    
    let endpoint = if let Some(cursor) = last_cursor {
        format!("/api/ingest/jobs?since={}", cursor)
    } else {
        "/api/ingest/jobs".to_string()
    };

    let response = client.get_with_auth(&endpoint).await?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Job polling failed: {}", response.status()));
    }

    let jobs_data: Value = response.json().await?;
    if let Some(jobs) = jobs_data["jobs"].as_array() {
        for job in jobs {
            let job_status = job["status"].as_str().unwrap();
            if job_status == "pending" {
                if let Err(e) = process_job(job).await {
                    log::error!("Failed to process job: {}", e);
                }
            }
        }
    }

    // Update cursor for next poll
    if let Some(new_cursor) = jobs_data["cursor"].as_str() {
        *last_cursor = Some(new_cursor.to_string());
    }

    Ok(())
}

async fn process_job(job: &Value) -> Result<()> {
    let job_type = job["type"].as_str()
        .ok_or_else(|| anyhow::anyhow!("Job missing type"))?;

    match job_type {
        "screenshot" => {
            process_screenshot_job(job).await?;
        }
        "diagnostics" => {
            process_diagnostics_job(job).await?;
        }
        _ => {
            log::warn!("Unknown job type: {}", job_type);
        }
    }

    Ok(())
}

async fn process_screenshot_job(job: &Value) -> Result<()> {
    let job_id = job["id"].as_str().unwrap();
    
    // Mark job as in_progress on the backend
    if let Err(e) = update_job_status(job_id, "in_progress", None).await {
        log::warn!("Failed to set job {} to in_progress: {}", job_id, e);
    }

    // Take screenshot
    let screenshot_data = screen_capture::capture_screen().await?;
    
    // Upload screenshot
    let upload_result = crate::api::uploads::upload_screenshot(&screenshot_data).await?;
    
    // Send completion event
    let completion_event = serde_json::json!({
        "jobId": job_id,
        "storageKey": upload_result["publicId"],
        "imageUrl": upload_result["secureUrl"],
        "width": upload_result["width"],
        "height": upload_result["height"],
        "bytes": upload_result["bytes"],
        "format": upload_result["format"],
        "createdAt": upload_result["createdAt"],
    });

    crate::storage::offline_queue::queue_event("screenshot_taken", &completion_event).await?;
    
    Ok(())
}

async fn update_job_status(job_id: &str, status: &str, result: Option<&Value>) -> Result<()> {
    let client = ApiClient::new().await?;
    let body = json!({
        "jobId": job_id,
        "status": status,
        "result": result
    });
    let resp = client.post_with_auth("/api/ingest/jobs", &body).await?;
    if !resp.status().is_success() {
        let status_code = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("Job status update failed: {} - {}", status_code, text));
    }
    Ok(())
}

async fn process_diagnostics_job(_job: &Value) -> Result<()> {
    // TODO: Implement diagnostics collection
    Ok(())
}

