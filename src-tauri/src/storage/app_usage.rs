use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::database;
use crate::utils::productivity::ProductivityCategory;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsageSession {
    pub id: Option<i64>,
    pub app_name: String,
    pub app_id: String,
    pub window_title: Option<String>,
    pub category: ProductivityCategory,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_seconds: i64,
    pub is_idle: bool,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct AppUsageTracker {
    current_session: Option<AppUsageSession>,
    session_history: Vec<AppUsageSession>,
    total_productive_time: i64,
    total_neutral_time: i64,
    total_unproductive_time: i64,
    total_idle_time: i64,
}

impl AppUsageTracker {
    pub fn new() -> Self {
        Self {
            current_session: None,
            session_history: Vec::new(),
            total_productive_time: 0,
            total_neutral_time: 0,
            total_unproductive_time: 0,
            total_idle_time: 0,
        }
    }

    pub async fn start_app_session(
        &mut self,
        app_name: String,
        app_id: String,
        window_title: Option<String>,
        category: ProductivityCategory,
        is_idle: bool,
    ) -> Result<()> {
        let now = Utc::now();

        // End current session if it exists
        if let Some(mut current) = self.current_session.take() {
            current.end_time = Some(now);
            current.duration_seconds = (now - current.start_time).num_seconds();
            current.is_active = false;
            
            // Update totals
            self.update_totals(&current);
            
            // Save to database
            self.save_session_to_db(&current).await?;
            
            self.session_history.push(current);
        }

        // Start new session
        let new_session = AppUsageSession {
            id: None,
            app_name,
            app_id,
            window_title,
            category,
            start_time: now,
            end_time: None,
            duration_seconds: 0,
            is_idle,
            is_active: true,
        };

        self.current_session = Some(new_session);
        
        Ok(())
    }

    pub async fn update_current_session(&mut self, is_idle: bool) -> Result<()> {
        if let Some(ref mut session) = self.current_session {
            session.is_idle = is_idle;
        }
        Ok(())
    }

    pub async fn end_current_session(&mut self) -> Result<()> {
        if let Some(mut current) = self.current_session.take() {
            let now = Utc::now();
            current.end_time = Some(now);
            current.duration_seconds = (now - current.start_time).num_seconds();
            current.is_active = false;
            
            // Update totals
            self.update_totals(&current);
            
            // Save to database
            self.save_session_to_db(&current).await?;
            
            // Don't send to backend - app_focus events already handle this
            // self.send_session_to_backend(&current).await?;
            
            self.session_history.push(current);
            
        }
        Ok(())
    }

    pub fn get_current_session(&self) -> Option<&AppUsageSession> {
        self.current_session.as_ref()
    }

    #[allow(dead_code)]
    pub fn get_session_history(&self) -> &Vec<AppUsageSession> {
        &self.session_history
    }

    pub fn get_totals(&self) -> (i64, i64, i64, i64) {
        (
            self.total_productive_time,
            self.total_neutral_time,
            self.total_unproductive_time,
            self.total_idle_time,
        )
    }

    pub fn get_app_usage_summary(&self) -> HashMap<String, AppUsageSummary> {
        let mut summary: HashMap<String, AppUsageSummary> = HashMap::new();

        // Process current session
        if let Some(session) = &self.current_session {
            let current_duration = (Utc::now() - session.start_time).num_seconds();
            let entry = summary.entry(session.app_name.clone()).or_insert_with(|| {
                AppUsageSummary::new(session.app_name.clone(), session.app_id.clone())
            });
            entry.add_time(session.category.clone(), current_duration, session.is_idle);
        }

        // Process history
        for session in &self.session_history {
            let entry = summary.entry(session.app_name.clone()).or_insert_with(|| {
                AppUsageSummary::new(session.app_name.clone(), session.app_id.clone())
            });
            entry.add_time(session.category.clone(), session.duration_seconds, session.is_idle);
        }

        summary
    }

    fn update_totals(&mut self, session: &AppUsageSession) {
        let duration = session.duration_seconds;
        
        if session.is_idle {
            self.total_idle_time += duration;
        } else {
            match session.category {
                ProductivityCategory::PRODUCTIVE => self.total_productive_time += duration,
                ProductivityCategory::NEUTRAL => self.total_neutral_time += duration,
                ProductivityCategory::UNPRODUCTIVE => self.total_unproductive_time += duration,
            }
        }
    }

    async fn save_session_to_db(&self, session: &AppUsageSession) -> Result<()> {
        let conn = database::get_connection()?;
        
        conn.execute(
            "INSERT INTO app_usage_sessions (
                app_name, app_id, window_title, category, 
                start_time, end_time, duration_seconds, is_idle, is_active, synced
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                session.app_name,
                session.app_id,
                session.window_title,
                session.category.to_string(),
                session.start_time,
                session.end_time,
                session.duration_seconds,
                session.is_idle,
                session.is_active,
                true, // Set synced = true since app_focus handles backend sync
            ],
        )?;
        
        Ok(())
    }

    // Removed send_session_to_backend - app_focus events handle all backend syncing

    pub async fn load_recent_sessions(&mut self, hours: i64) -> Result<()> {
        let conn = database::get_connection()?;
        let cutoff_time = Utc::now() - Duration::hours(hours);
        
        let mut stmt = conn.prepare(
            "SELECT id, app_name, app_id, window_title, category, 
                    start_time, end_time, duration_seconds, is_idle, is_active
             FROM app_usage_sessions 
             WHERE start_time >= ?1 
             ORDER BY start_time DESC"
        )?;
        
        let rows = stmt.query_map(params![cutoff_time], |row| {
            let category_str: String = row.get(4)?;
            let category = match category_str.as_str() {
                "PRODUCTIVE" => ProductivityCategory::PRODUCTIVE,
                "UNPRODUCTIVE" => ProductivityCategory::UNPRODUCTIVE,
                _ => ProductivityCategory::NEUTRAL,
            };
            
            Ok(AppUsageSession {
                id: Some(row.get(0)?),
                app_name: row.get(1)?,
                app_id: row.get(2)?,
                window_title: row.get(3)?,
                category,
                start_time: row.get(5)?,
                end_time: row.get(6)?,
                duration_seconds: row.get(7)?,
                is_idle: row.get(8)?,
                is_active: row.get(9)?,
            })
        })?;
        
        for row in rows {
            let session = row?;
            if session.is_active {
                self.current_session = Some(session);
            } else {
                self.session_history.push(session);
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsageSummary {
    pub app_name: String,
    pub app_id: String,
    pub total_time: i64,
    pub productive_time: i64,
    pub neutral_time: i64,
    pub unproductive_time: i64,
    pub idle_time: i64,
    pub session_count: i32,
}

impl AppUsageSummary {
    fn new(app_name: String, app_id: String) -> Self {
        Self {
            app_name,
            app_id,
            total_time: 0,
            productive_time: 0,
            neutral_time: 0,
            unproductive_time: 0,
            idle_time: 0,
            session_count: 0,
        }
    }

    fn add_time(&mut self, category: ProductivityCategory, duration: i64, is_idle: bool) {
        self.total_time += duration;
        self.session_count += 1;
        
        if is_idle {
            self.idle_time += duration;
        } else {
            match category {
                ProductivityCategory::PRODUCTIVE => self.productive_time += duration,
                ProductivityCategory::NEUTRAL => self.neutral_time += duration,
                ProductivityCategory::UNPRODUCTIVE => self.unproductive_time += duration,
            }
        }
    }
}

// Removed send_app_usage_to_backend function - no longer needed
// App usage is now tracked solely via app_focus events

// Global app usage tracker instance
use tokio::sync::Mutex as TokioMutex;

lazy_static::lazy_static! {
    static ref APP_USAGE_TRACKER: TokioMutex<AppUsageTracker> = 
        TokioMutex::new(AppUsageTracker::new());
}

pub async fn start_app_session(
    app_name: String,
    app_id: String,
    window_title: Option<String>,
    category: ProductivityCategory,
    is_idle: bool,
) -> Result<()> {
    let mut tracker = APP_USAGE_TRACKER.lock().await;
    tracker.start_app_session(app_name, app_id, window_title, category, is_idle).await
}

pub async fn update_current_session(is_idle: bool) -> Result<()> {
    let mut tracker = APP_USAGE_TRACKER.lock().await;
    tracker.update_current_session(is_idle).await
}

pub async fn end_current_session() -> Result<()> {
    let mut tracker = APP_USAGE_TRACKER.lock().await;
    tracker.end_current_session().await
}

pub async fn get_current_session() -> Option<AppUsageSession> {
    let tracker = APP_USAGE_TRACKER.lock().await;
    tracker.get_current_session().cloned()
}


pub async fn get_app_usage_summary() -> HashMap<String, AppUsageSummary> {
    let tracker = APP_USAGE_TRACKER.lock().await;
    tracker.get_app_usage_summary()
}

pub async fn get_usage_totals() -> (i64, i64, i64, i64) {
    let tracker = APP_USAGE_TRACKER.lock().await;
    tracker.get_totals()
}

pub async fn load_recent_sessions(hours: i64) -> Result<()> {
    let mut tracker = APP_USAGE_TRACKER.lock().await;
    tracker.load_recent_sessions(hours).await
}

/// Reset the app usage tracker to clear any stale sessions
pub async fn reset_tracker() -> Result<()> {
    let mut tracker = APP_USAGE_TRACKER.lock().await;
    // End any current session to prevent large duration calculations
    if let Some(mut current) = tracker.current_session.take() {
        let now = Utc::now();
        current.end_time = Some(now);
        current.duration_seconds = (now - current.start_time).num_seconds();
        current.is_active = false;
        
        // Update totals
        tracker.update_totals(&current);
        
        // Save to database
        tracker.save_session_to_db(&current).await?;
        
        tracker.session_history.push(current);
    }
    
    // Reset tracker to clean state
    *tracker = AppUsageTracker::new();
    
    log::info!("App usage tracker reset successfully");
    Ok(())
}

/// Handle system wake from sleep - mark idle time during sleep
pub async fn handle_system_wake(_sleep_duration_seconds: u64) -> Result<()> {
    let mut tracker = APP_USAGE_TRACKER.lock().await;
    
    // If there's a current session, end it and mark as idle
    if let Some(mut session) = tracker.current_session.take() {
        session.end_time = Some(chrono::Utc::now());
        session.duration_seconds = (session.end_time.unwrap() - session.start_time).num_seconds() as i64;
        session.is_idle = true; // Mark as idle since system was sleeping
        
        // Save the session
        tracker.save_session_to_db(&session).await?;
        log::info!("Marked previous session as idle due to system sleep: {} ({}s)", 
                  session.app_name, session.duration_seconds);
    }
    
    // Don't start a new session - wait for actual app focus
    Ok(())
}

// Initialize database table for app usage sessions
pub async fn init_database() -> Result<()> {
    let conn = database::get_connection()?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_usage_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            app_name TEXT NOT NULL,
            app_id TEXT NOT NULL,
            window_title TEXT,
            category TEXT NOT NULL,
            start_time DATETIME NOT NULL,
            end_time DATETIME,
            duration_seconds INTEGER NOT NULL DEFAULT 0,
            is_idle BOOLEAN NOT NULL DEFAULT 0,
            is_active BOOLEAN NOT NULL DEFAULT 1,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    
    // Create indexes for better performance
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_app_usage_app_name ON app_usage_sessions(app_name)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_app_usage_start_time ON app_usage_sessions(start_time)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_app_usage_category ON app_usage_sessions(category)",
        [],
    )?;
    
    Ok(())
}
