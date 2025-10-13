use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct PolicyConfig {
    pub screenshot_enabled: bool,
    pub screenshot_interval_minutes: u32, // 0 = disabled, 15/30/60 = intervals
    pub domain_only_mode: bool,
    pub title_redaction_enabled: bool,
    pub idle_threshold_seconds: u64,
    pub allowlist_patterns: Vec<String>,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            screenshot_enabled: false,
            screenshot_interval_minutes: 0,
            domain_only_mode: true,
            title_redaction_enabled: true,
            idle_threshold_seconds: 300, // 5 minutes
            allowlist_patterns: crate::policy::privacy::get_default_allowlist_patterns(),
        }
    }
}

impl PolicyConfig {
    #[allow(dead_code)]
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        // Read configuration from environment variables
        if let Ok(val) = std::env::var("TRACKEX_SCREENSHOT_ENABLED") {
            config.screenshot_enabled = val.parse().unwrap_or(false);
        }
        
        if let Ok(val) = std::env::var("TRACKEX_SCREENSHOT_INTERVAL") {
            config.screenshot_interval_minutes = val.parse().unwrap_or(0);
        }
        
        if let Ok(val) = std::env::var("TRACKEX_DOMAIN_ONLY") {
            config.domain_only_mode = val.parse().unwrap_or(true);
        }
        
        if let Ok(val) = std::env::var("TRACKEX_TITLE_REDACTION") {
            config.title_redaction_enabled = val.parse().unwrap_or(true);
        }
        
        if let Ok(val) = std::env::var("TRACKEX_IDLE_THRESHOLD") {
            config.idle_threshold_seconds = val.parse().unwrap_or(300);
        }
        
        config
    }
    
    #[allow(dead_code)]
    pub fn should_take_screenshot(&self) -> bool {
        self.screenshot_enabled && self.screenshot_interval_minutes > 0
    }
    
    #[allow(dead_code)]
    pub fn get_screenshot_interval_seconds(&self) -> u64 {
        (self.screenshot_interval_minutes as u64) * 60
    }
    
    #[allow(dead_code)]
    pub fn should_redact_title(&self, app_id: &str) -> bool {
        if !self.title_redaction_enabled {
            return false;
        }
        
        // Always use domain-only for browsers if domain_only_mode is enabled
        if self.domain_only_mode && crate::policy::privacy::should_use_domain_only(app_id) {
            return true;
        }
        
        // Use redaction for other apps if enabled
        self.title_redaction_enabled
    }
}

#[allow(dead_code)]
static CURRENT_POLICY: Mutex<Option<PolicyConfig>> = Mutex::new(None);

#[allow(dead_code)]
pub fn get_current_policy() -> PolicyConfig {
    CURRENT_POLICY.lock().unwrap().clone().unwrap_or_else(|| PolicyConfig::default())
}

#[allow(dead_code)]
pub fn update_policy(config: PolicyConfig) {
    *CURRENT_POLICY.lock().unwrap() = Some(config);
}
    
#[allow(dead_code)]
pub fn initialize_policy() {
    let config = PolicyConfig::from_env();
    update_policy(config);
}
