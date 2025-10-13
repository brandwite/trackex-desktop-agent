use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::utils::productivity::{ProductivityClassifier, AppRule, ProductivityCategory};
use crate::api::client::ApiClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteAppRule {
    pub id: String,
    pub matcher_type: String, // EXACT, GLOB, REGEX, DOMAIN
    pub value: String,
    pub category: String, // PRODUCTIVE, NEUTRAL, UNPRODUCTIVE
    pub priority: i32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

pub struct AppRulesManager {
    classifier: ProductivityClassifier,
    last_sync: Option<chrono::DateTime<chrono::Utc>>,
    sync_interval: chrono::Duration,
}

impl AppRulesManager {
    pub fn new() -> Self {
        Self {
            classifier: ProductivityClassifier::with_default_rules(),
            last_sync: None,
            sync_interval: chrono::Duration::hours(1), // Sync every hour
        }
    }

    pub async fn sync_rules_from_server(&mut self) -> Result<()> {
        
        let client = ApiClient::new().await?;
        let response = client.get_with_auth("/api/app-rules").await?;
        
        if response.status().is_success() {
            let remote_rules: Vec<RemoteAppRule> = response.json().await?;
            
            // Convert remote rules to local rules
            let mut local_rules = Vec::new();
            for remote_rule in remote_rules {
                let category = match remote_rule.category.as_str() {
                    "PRODUCTIVE" => ProductivityCategory::PRODUCTIVE,
                    "UNPRODUCTIVE" => ProductivityCategory::UNPRODUCTIVE,
                    _ => ProductivityCategory::NEUTRAL,
                };
                
                let local_rule = AppRule {
                    matcher_type: remote_rule.matcher_type,
                    value: remote_rule.value,
                    category,
                    priority: remote_rule.priority,
                    is_active: remote_rule.is_active,
                };
                
                local_rules.push(local_rule);
            }
            
            // Update classifier with new rules
            self.classifier.clear_rules();
            self.classifier.add_rules(local_rules);
            
            self.last_sync = Some(chrono::Utc::now());
        } else {
            log::warn!("Failed to sync app rules from server: {}", response.status());
        }
        
        Ok(())
    }

    pub async fn should_sync(&self) -> bool {
        match self.last_sync {
            Some(last_sync) => {
                let now = chrono::Utc::now();
                now - last_sync >= self.sync_interval
            }
            None => true, // Never synced before
        }
    }

    pub async fn auto_sync_if_needed(&mut self) -> Result<()> {
        if self.should_sync().await {
            if let Err(e) = self.sync_rules_from_server().await {
                log::error!("Failed to auto-sync app rules: {}", e);
                // Don't return error, just log it and continue with existing rules
            }
        }
        Ok(())
    }

    pub fn get_rules(&self) -> &Vec<AppRule> {
        self.classifier.get_rules()
    }

    #[allow(dead_code)]
    pub fn add_rule(&mut self, rule: AppRule) {
        self.classifier.add_rule(rule);
    }

    #[allow(dead_code)]
    pub fn clear_rules(&mut self) {
        self.classifier.clear_rules();
    }

    #[allow(dead_code)]
    pub fn set_default_category(&mut self, category: ProductivityCategory) {
        self.classifier.set_default_category(category);
    }

    #[allow(dead_code)]
    pub async fn upload_custom_rule(&self, rule: &AppRule) -> Result<()> {
        let client = ApiClient::new().await?;
        
        let remote_rule = serde_json::json!({
            "matcher_type": rule.matcher_type,
            "value": rule.value,
            "category": rule.category.to_string(),
            "priority": rule.priority,
            "is_active": rule.is_active
        });
        
        let response = client.post_with_auth("/api/app-rules", &remote_rule).await?;
        
        if response.status().is_success() {
        } else {
            log::error!("Failed to upload custom app rule: {}", response.status());
        }
        
        Ok(())
    }

    pub async fn get_rule_statistics(&self) -> Result<RuleStatistics> {
        let rules = self.get_rules();
        let mut stats = RuleStatistics {
            total_rules: rules.len(),
            active_rules: 0,
            productive_rules: 0,
            neutral_rules: 0,
            unproductive_rules: 0,
            exact_matchers: 0,
            glob_matchers: 0,
            regex_matchers: 0,
            domain_matchers: 0,
        };
        
        for rule in rules {
            if rule.is_active {
                stats.active_rules += 1;
            }
            
            match rule.category {
                ProductivityCategory::PRODUCTIVE => stats.productive_rules += 1,
                ProductivityCategory::NEUTRAL => stats.neutral_rules += 1,
                ProductivityCategory::UNPRODUCTIVE => stats.unproductive_rules += 1,
            }
            
            match rule.matcher_type.as_str() {
                "EXACT" => stats.exact_matchers += 1,
                "GLOB" => stats.glob_matchers += 1,
                "REGEX" => stats.regex_matchers += 1,
                "DOMAIN" => stats.domain_matchers += 1,
                _ => {}
            }
        }
        
        Ok(stats)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleStatistics {
    pub total_rules: usize,
    pub active_rules: usize,
    pub productive_rules: usize,
    pub neutral_rules: usize,
    pub unproductive_rules: usize,
    pub exact_matchers: usize,
    pub glob_matchers: usize,
    pub regex_matchers: usize,
    pub domain_matchers: usize,
}

// Global app rules manager instance
use tokio::sync::Mutex as TokioMutex;

lazy_static::lazy_static! {
    static ref APP_RULES_MANAGER: TokioMutex<AppRulesManager> = 
        TokioMutex::new(AppRulesManager::new());
}

pub async fn sync_app_rules() -> Result<()> {
    let mut manager = APP_RULES_MANAGER.lock().await;
    manager.sync_rules_from_server().await
}

pub async fn auto_sync_app_rules() -> Result<()> {
    let mut manager = APP_RULES_MANAGER.lock().await;
    manager.auto_sync_if_needed().await
}

pub async fn get_app_rules() -> Vec<AppRule> {
    let manager = APP_RULES_MANAGER.lock().await;
    manager.get_rules().clone()
}

#[allow(dead_code)]
pub async fn add_custom_rule(rule: AppRule) -> Result<()> {
    let mut manager = APP_RULES_MANAGER.lock().await;
    manager.add_rule(rule.clone());
    manager.upload_custom_rule(&rule).await
}

pub async fn get_rule_statistics() -> Result<RuleStatistics> {
    let manager = APP_RULES_MANAGER.lock().await;
    manager.get_rule_statistics().await
}

pub async fn initialize_app_rules() -> Result<()> {
    
    // Try to sync rules from server, but don't fail if it doesn't work
    if let Err(e) = sync_app_rules().await {
        log::warn!("Failed to sync app rules from server, using defaults: {}", e);
    }
    
    // Start periodic sync
    tokio::spawn(async {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // 1 hour
        
        loop {
            interval.tick().await;
            if let Err(e) = auto_sync_app_rules().await {
                log::error!("Failed to auto-sync app rules: {}", e);
            }
        }
    });
    
    Ok(())
}
