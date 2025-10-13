pub mod consent;
pub mod database;
pub mod secure_store;
pub mod work_session;
pub mod offline_queue;
pub mod app_usage;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct AppState {
    pub device_token: Option<String>,
    pub device_id: Option<String>,
    pub email: Option<String>,
    pub server_url: Option<String>,
    pub employee_id: Option<String>,
    pub is_paused: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            device_token: None,
            device_id: None,
            email: None,
            server_url: None,
            employee_id: None,
            is_paused: false,
        }
    }

    #[allow(dead_code)]
    pub async fn initialize(&mut self) -> Result<()> {
        // Initialize database
        database::init().await?;
        
        // Initialize app usage tracking
        app_usage::init_database().await?;
        
        // Load recent app usage sessions
        app_usage::load_recent_sessions(24).await?; // Load last 24 hours
        
        // Initialize app rules
        crate::api::app_rules::initialize_app_rules().await?;
        
        Ok(())
    }
}

// Global app state manager
static GLOBAL_APP_STATE: OnceLock<Arc<Mutex<AppState>>> = OnceLock::new();

pub fn set_global_app_state(state: Arc<Mutex<AppState>>) {
    GLOBAL_APP_STATE.set(state).expect("Failed to set global app state");
}

// Function to sync device token from Tauri-managed AppState to Global AppState
pub async fn sync_device_token_to_global(device_token: String, device_id: String, email: String, server_url: String, employee_id: String) -> Result<()> {
    match get_global_app_state() {
        Ok(global_state) => {
            let mut state = global_state.lock().await;
            state.device_token = Some(device_token);
            state.device_id = Some(device_id);
            state.email = Some(email);
            state.server_url = Some(server_url);
            state.employee_id = Some(employee_id);
            Ok(())
        }
        Err(e) => {
            // If global state is not initialized, log warning but don't fail
            log::warn!("Global app state not initialized yet, skipping device token sync: {}", e);
            Ok(())
        }
    }
}

pub fn get_global_app_state() -> Result<Arc<Mutex<AppState>>> {
    GLOBAL_APP_STATE.get()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Global app state not initialized"))
}

// Global storage functions
pub async fn get_server_url() -> Result<String> {
    // Try to get the server URL from the global app state, fallback to default if not available
    match get_global_app_state() {
        Ok(app_state) => {
            let state = app_state.lock().await;
            if let Some(url) = &state.server_url {
                Ok(url.clone())
            } else {
                log::warn!("No server URL found in app state, using default");
                Ok("https://www.trackex.app".to_string())
            }
        }
        Err(_) => {
            log::warn!("Global app state not available, using default server URL");
            Ok("https://www.trackex.app".to_string())
        }
    }
}

pub async fn get_device_token() -> Result<String> {
    // Try to get the device token from the global app state, fallback to empty if not available
    match get_global_app_state() {
        Ok(app_state) => {
            let state = app_state.lock().await;
            if let Some(token) = &state.device_token {
                if !token.is_empty() {
                    Ok(token.clone())
                } else {
                    Err(anyhow::anyhow!("Device token is empty - user not authenticated"))
                }
            } else {
                Err(anyhow::anyhow!("No device token found - user not authenticated"))
            }
        }
        Err(_) => {
            Err(anyhow::anyhow!("Global app state not available"))
        }
    }
}