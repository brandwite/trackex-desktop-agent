use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

use crate::storage::app_usage;
use crate::utils::productivity::ProductivityCategory;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AppUsageReport {
    pub employee_id: String,
    pub device_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub total_productive_time: i64,
    pub total_neutral_time: i64,
    pub total_unproductive_time: i64,
    pub total_idle_time: i64,
    pub total_active_time: i64,
    pub app_usage: Vec<AppUsageEntry>,
    pub productivity_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AppUsageEntry {
    pub app_name: String,
    pub app_id: String,
    pub window_title: Option<String>,
    pub category: ProductivityCategory,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration: i64,
    pub is_idle: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyReport {
    pub date: String, // YYYY-MM-DD format
    pub total_work_time: i64,
    pub productive_time: i64,
    pub neutral_time: i64,
    pub unproductive_time: i64,
    pub idle_time: i64,
    pub productivity_score: f64,
    pub top_apps: Vec<TopApp>,
    pub category_breakdown: CategoryBreakdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopApp {
    pub app_name: String,
    pub app_id: String,
    pub total_time: i64,
    pub category: ProductivityCategory,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryBreakdown {
    pub productive_apps: Vec<String>,
    pub neutral_apps: Vec<String>,
    pub unproductive_apps: Vec<String>,
    pub productive_time: i64,
    pub neutral_time: i64,
    pub unproductive_time: i64,
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
        let start_of_day = date.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let end_of_day = date.date_naive().and_hms_opt(23, 59, 59).unwrap();
        
        let _start_time: DateTime<Utc> = DateTime::from_naive_utc_and_offset(start_of_day, Utc);
        let _end_time: DateTime<Utc> = DateTime::from_naive_utc_and_offset(end_of_day, Utc);
        
        // Get app usage summary for the day
        let app_summary = app_usage::get_app_usage_summary().await;
        
        // Calculate totals
        let mut total_productive_time = 0i64;
        let mut total_neutral_time = 0i64;
        let mut total_unproductive_time = 0i64;
        let mut total_idle_time = 0i64;
        
        let mut top_apps = Vec::new();
        let mut productive_apps = Vec::new();
        let mut neutral_apps = Vec::new();
        let mut unproductive_apps = Vec::new();
        
        for (app_name, summary) in &app_summary {
            total_productive_time += summary.productive_time;
            total_neutral_time += summary.neutral_time;
            total_unproductive_time += summary.unproductive_time;
            total_idle_time += summary.idle_time;
            
            // Determine primary category
            let primary_category = if summary.productive_time > summary.neutral_time && 
                                   summary.productive_time > summary.unproductive_time {
                ProductivityCategory::PRODUCTIVE
            } else if summary.unproductive_time > summary.neutral_time {
                ProductivityCategory::UNPRODUCTIVE
            } else {
                ProductivityCategory::NEUTRAL
            };
            
            // Add to category lists
            match primary_category {
                ProductivityCategory::PRODUCTIVE => productive_apps.push(app_name.clone()),
                ProductivityCategory::NEUTRAL => neutral_apps.push(app_name.clone()),
                ProductivityCategory::UNPRODUCTIVE => unproductive_apps.push(app_name.clone()),
            }
            
            // Add to top apps
            top_apps.push(TopApp {
                app_name: app_name.clone(),
                app_id: summary.app_id.clone(),
                total_time: summary.total_time,
                category: primary_category,
                percentage: 0.0, // Will be calculated below
            });
        }
        
        // Sort top apps by total time
        top_apps.sort_by(|a, b| b.total_time.cmp(&a.total_time));
        top_apps.truncate(10); // Keep top 10
        
        // Calculate percentages
        let total_work_time = total_productive_time + total_neutral_time + total_unproductive_time;
        for app in &mut top_apps {
            if total_work_time > 0 {
                app.percentage = (app.total_time as f64 / total_work_time as f64) * 100.0;
            }
        }
        
        // Calculate productivity score (0-100)
        let productivity_score = if total_work_time > 0 {
            let productive_ratio = total_productive_time as f64 / total_work_time as f64;
            let unproductive_ratio = total_unproductive_time as f64 / total_work_time as f64;
            (productive_ratio * 100.0) - (unproductive_ratio * 50.0)
        } else {
            0.0
        };
        
        let category_breakdown = CategoryBreakdown {
            productive_apps,
            neutral_apps,
            unproductive_apps,
            productive_time: total_productive_time,
            neutral_time: total_neutral_time,
            unproductive_time: total_unproductive_time,
        };
        
        Ok(DailyReport {
            date: date.format("%Y-%m-%d").to_string(),
            total_work_time,
            productive_time: total_productive_time,
            neutral_time: total_neutral_time,
            unproductive_time: total_unproductive_time,
            idle_time: total_idle_time,
            productivity_score: productivity_score.max(0.0).min(100.0),
            top_apps,
            category_breakdown,
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
        let mut total_productive_time = 0i64;
        let mut total_neutral_time = 0i64;
        let mut total_unproductive_time = 0i64;
        let mut total_idle_time = 0i64;
        
        let mut app_usage_entries = Vec::new();
        
        for (app_name, summary) in &app_summary {
            total_productive_time += summary.productive_time;
            total_neutral_time += summary.neutral_time;
            total_unproductive_time += summary.unproductive_time;
            total_idle_time += summary.idle_time;
            
            // Determine primary category
            let primary_category = if summary.productive_time > summary.neutral_time && 
                                   summary.productive_time > summary.unproductive_time {
                ProductivityCategory::PRODUCTIVE
            } else if summary.unproductive_time > summary.neutral_time {
                ProductivityCategory::UNPRODUCTIVE
            } else {
                ProductivityCategory::NEUTRAL
            };
            
            app_usage_entries.push(AppUsageEntry {
                app_name: app_name.clone(),
                app_id: summary.app_id.clone(),
                window_title: None, // Could be enhanced to track window titles
                category: primary_category,
                start_time,
                end_time,
                duration: summary.total_time,
                is_idle: summary.idle_time > 0,
            });
        }
        
        // Sort by duration
        app_usage_entries.sort_by(|a, b| b.duration.cmp(&a.duration));
        
        let total_active_time = total_productive_time + total_neutral_time + total_unproductive_time;
        
        // Calculate productivity score
        let productivity_score = if total_active_time > 0 {
            let productive_ratio = total_productive_time as f64 / total_active_time as f64;
            let unproductive_ratio = total_unproductive_time as f64 / total_active_time as f64;
            (productive_ratio * 100.0) - (unproductive_ratio * 50.0)
        } else {
            0.0
        };
        
        Ok(AppUsageReport {
            employee_id: self.employee_id.clone(),
            device_id: self.device_id.clone(),
            start_time,
            end_time,
            total_productive_time,
            total_neutral_time,
            total_unproductive_time,
            total_idle_time,
            total_active_time,
            app_usage: app_usage_entries,
            productivity_score: productivity_score.max(0.0).min(100.0),
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
    let mut total_productive = 0i64;
    let mut total_neutral = 0i64;
    let mut total_unproductive = 0i64;
    let mut total_idle = 0i64;
    let mut daily_scores = Vec::new();
    
    // Generate reports for the last 30 days
    for i in 0..30 {
        let date = Utc::now() - Duration::days(i);
        let report = generator.generate_daily_report(date).await?;
        
        total_productive += report.productive_time;
        total_neutral += report.neutral_time;
        total_unproductive += report.unproductive_time;
        total_idle += report.idle_time;
        daily_scores.push(report.productivity_score);
    }
    
    let total_work_time = total_productive + total_neutral + total_unproductive;
    let avg_productivity = if daily_scores.len() > 0 {
        daily_scores.iter().sum::<f64>() / daily_scores.len() as f64
    } else {
        0.0
    };
    
    Ok(MonthlySummary {
        month: Utc::now().format("%Y-%m").to_string(),
        total_work_time,
        total_productive_time: total_productive,
        total_neutral_time: total_neutral,
        total_unproductive_time: total_unproductive,
        total_idle_time: total_idle,
        average_productivity_score: avg_productivity,
        daily_scores,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlySummary {
    pub month: String,
    pub total_work_time: i64,
    pub total_productive_time: i64,
    pub total_neutral_time: i64,
    pub total_unproductive_time: i64,
    pub total_idle_time: i64,
    pub average_productivity_score: f64,
    pub daily_scores: Vec<f64>,
}
