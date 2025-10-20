use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write;
use std::time::Duration;

pub fn init() {
    let mut builder = Builder::from_default_env();
    
    builder
        .target(Target::Stdout)
        .filter_level(if cfg!(debug_assertions) {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] [{}] {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.args()
            )
        })
        .init();

}

/// Send a small JSON log to remote endpoint (fire-and-forget)
/// This function never panics and will not block the main loop.
pub async fn log_remote_non_blocking(event: &str, level: &str, message: &str, context: Option<serde_json::Value>) {
    // Build payload
    let payload = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "event": event,
        "level": level,
        "message": message,
        "context": context.unwrap_or(serde_json::json!({}))
    });

    // Spawn and detach the network call
    tokio::spawn(async move {
        // Short timeout client
        let client = match reqwest::Client::builder()
            .timeout(Duration::from_secs(4))
            .build() {
                Ok(c) => c,
                Err(e) => {
                    log::debug!("remote log: failed to build client: {}", e);
                    return;
                }
            };

        // Resolve server URL from storage (falls back internally to default)
        let base_url = match crate::storage::get_server_url().await {
            Ok(u) => u,
            Err(e) => {
                log::debug!("remote log: failed to get server url: {}", e);
                "https://www.trackex.app".to_string()
            }
        };
        let base = base_url.trim_end_matches('/');
        let url = format!("{}/api/logs", base);
        match client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        log::debug!("remote log failed with status {}", resp.status());
                    }
                }
                Err(e) => {
                    log::debug!("remote log error: {}", e);
                }
            }
    });
}