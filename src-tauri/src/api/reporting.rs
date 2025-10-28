use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

use crate::storage::app_usage;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AppUsageReport {
    pub employee_id: String,
    pub device_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub total_idle_time: i64,
    pub total_active_time: i64,
    pub app_usage: Vec<AppUsageEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AppUsageEntry {
    pub app_name: String,
    pub app_id: String,
    pub window_title: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration: i64,
    pub is_idle: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyReport {
    pub date: String, // YYYY-MM-DD format
    pub total_work_time: i64,
    pub idle_time: i64,
    pub top_apps: Vec<TopApp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopApp {
    pub app_name: String,
    pub app_id: String,
    pub total_time: i64,
    pub percentage: f64,
}


pub struct ReportGenerator {
    #[allow(dead_code)]
    employee_id: String,
    #[allow(dead_code)]
    device_id: String,
}

impl ReportGenerator {
    pub fn new(employee_id: String, device_id: String) -> Self {
        Self {
            employee_id,
            device_id,
        }
    }

    pub async fn generate_daily_report(&self, date: DateTime<Utc>) -> Result<DailyReport> {
        // Get app usage summary for the day
        let app_summary = app_usage::get_app_usage_summary().await;
        
        // Calculate totals
        let mut total_work_time = 0i64;
        let mut total_idle_time = 0i64;
        
        let mut top_apps = Vec::new();
        
        for (app_name, summary) in &app_summary {
            total_work_time += summary.total_time;
            total_idle_time += summary.idle_time;
            
            // Add to top apps
            top_apps.push(TopApp {
                app_name: app_name.clone(),
                app_id: summary.app_id.clone(),
                total_time: summary.total_time,
                percentage: 0.0, // Will be calculated below
            });
        }
        
        // Sort top apps by total time
        top_apps.sort_by(|a, b| b.total_time.cmp(&a.total_time));
        top_apps.truncate(10); // Keep top 10
        
        // Calculate percentages
        for app in &mut top_apps {
            if total_work_time > 0 {
                app.percentage = (app.total_time as f64 / total_work_time as f64) * 100.0;
            }
        }
        
        Ok(DailyReport {
            date: date.format("%Y-%m-%d").to_string(),
            total_work_time,
            idle_time: total_idle_time,
            top_apps,
        })
    }

    #[allow(dead_code)]
    pub async fn generate_app_usage_report(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<AppUsageReport> {
        // Get app usage summary
        let app_summary = app_usage::get_app_usage_summary().await;
        
        // Calculate totals
        let mut total_work_time = 0i64;
        let mut total_idle_time = 0i64;
        
        let mut app_usage_entries = Vec::new();
        
        for (app_name, summary) in &app_summary {
            total_work_time += summary.total_time;
            total_idle_time += summary.idle_time;
            
            app_usage_entries.push(AppUsageEntry {
                app_name: app_name.clone(),
                app_id: summary.app_id.clone(),
                window_title: None, // Could be enhanced to track window titles
                start_time,
                end_time,
                duration: summary.total_time,
                is_idle: summary.idle_time > 0,
            });
        }
        
        // Sort by duration
        app_usage_entries.sort_by(|a, b| b.duration.cmp(&a.duration));
        
        Ok(AppUsageReport {
            employee_id: self.employee_id.clone(),
            device_id: self.device_id.clone(),
            start_time,
            end_time,
            total_idle_time,
            total_active_time: total_work_time,
            app_usage: app_usage_entries,
        })
    }

    #[allow(dead_code)]
    pub async fn send_report_to_server(&self, _report: &AppUsageReport) -> Result<()> {
        // This would send the report to your backend API
        // For now, we'll just log it
        
        // TODO: Implement actual API call to send report
        // let client = crate::api::client::ApiClient::new().await?;
        // client.post_with_auth("/api/employees/app-usage", &serde_json::to_value(report)?).await?;
        
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn send_daily_report_to_server(&self, _report: &DailyReport) -> Result<()> {
        // This would send the daily report to your backend API
        
        // TODO: Implement actual API call to send daily report
        // let client = crate::api::client::ApiClient::new().await?;
        // client.post_with_auth("/api/employees/daily-reports", &serde_json::to_value(report)?).await?;
        
        Ok(())
    }
}

// Helper functions for generating reports
pub async fn generate_today_report(employee_id: String, device_id: String) -> Result<DailyReport> {
    let generator = ReportGenerator::new(employee_id, device_id);
    generator.generate_daily_report(Utc::now()).await
}

pub async fn generate_weekly_report(employee_id: String, device_id: String) -> Result<Vec<DailyReport>> {
    let mut reports = Vec::new();
    let generator = ReportGenerator::new(employee_id, device_id);
    
    // Generate reports for the last 7 days
    for i in 0..7 {
        let date = Utc::now() - Duration::days(i);
        let report = generator.generate_daily_report(date).await?;
        reports.push(report);
    }
    
    Ok(reports)
}

pub async fn generate_monthly_summary(employee_id: String, device_id: String) -> Result<MonthlySummary> {
    let generator = ReportGenerator::new(employee_id, device_id);
    let mut total_work_time = 0i64;
    let mut total_idle = 0i64;
    
    // Generate reports for the last 30 days
    for i in 0..30 {
        let date = Utc::now() - Duration::days(i);
        let report = generator.generate_daily_report(date).await?;
        
        total_work_time += report.total_work_time;
        total_idle += report.idle_time;
    }
    
    Ok(MonthlySummary {
        month: Utc::now().format("%Y-%m").to_string(),
        total_work_time,
        total_idle_time: total_idle,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlySummary {
    pub month: String,
    pub total_work_time: i64,
    pub total_idle_time: i64,
}
