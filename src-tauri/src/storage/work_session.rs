use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::database;

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct WorkSession {
    pub id: i64,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[allow(dead_code)]
pub async fn start_session() -> Result<i64> {
    let conn = database::get_connection()?;
    
    // End any existing active sessions first
    conn.execute(
        "UPDATE work_sessions SET is_active = 0, ended_at = CURRENT_TIMESTAMP 
         WHERE is_active = 1",
        [],
    )?;
    
    let now = Utc::now();
    
    // Start new session
    conn.execute(
        "INSERT INTO work_sessions (started_at, is_active) VALUES (?1, 1)",
        params![now],
    )?;
    
    let session_id = conn.last_insert_rowid();
    
    Ok(session_id)
}

#[allow(dead_code)]
pub async fn end_session() -> Result<()> {
    let conn = database::get_connection()?;
    
    let rows_affected = conn.execute(
        "UPDATE work_sessions SET is_active = 0, ended_at = CURRENT_TIMESTAMP 
         WHERE is_active = 1",
        [],
    )?;
    
    if rows_affected > 0 {
    } else {
        log::warn!("No active work session to end");
    }
    
    Ok(())
}

#[allow(dead_code)]
pub async fn get_current_session() -> Result<Option<WorkSession>> {
    let conn = database::get_connection()?;
    
    let mut stmt = conn.prepare(
        "SELECT id, started_at, ended_at, is_active 
         FROM work_sessions 
         WHERE is_active = 1 
         ORDER BY started_at DESC 
         LIMIT 1"
    )?;
    
    match stmt.query_row([], |row| {
        Ok(WorkSession {
            id: row.get(0)?,
            started_at: row.get(1)?,
            ended_at: row.get(2)?,
            is_active: row.get(3)?,
        })
    }) {
        Ok(session) => Ok(Some(session)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

#[allow(dead_code)]
pub async fn is_session_active() -> Result<bool> {
    let session = get_current_session().await?;
    Ok(session.is_some())
}

#[allow(dead_code)]
pub async fn clear_all_active_sessions() -> Result<()> {
    let conn = database::get_connection()?;
    
    let rows_affected = conn.execute(
        "UPDATE work_sessions SET is_active = 0, ended_at = CURRENT_TIMESTAMP 
         WHERE is_active = 1",
        [],
    )?;
    
    if rows_affected > 0 {
        log::info!("Cleared {} active sessions from database", rows_affected);
    } else {
        log::info!("No active sessions to clear");
    }
    
    Ok(())
}

#[allow(dead_code)]
pub async fn get_current_session_id() -> Result<Option<i64>> {
    let session = get_current_session().await?;
    Ok(session.map(|s| s.id))
}

pub async fn get_session_start_time() -> Result<DateTime<Utc>> {
    let conn = database::get_connection()?;
    
    let mut stmt = conn.prepare(
        "SELECT started_at FROM work_sessions 
         WHERE is_active = 1 
         ORDER BY started_at DESC 
         LIMIT 1"
    )?;
    
    match stmt.query_row([], |row| {
        Ok(row.get::<_, DateTime<Utc>>(0)?)
    }) {
        Ok(start_time) => Ok(start_time),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // No active session, return current time
            Ok(Utc::now())
        },
        Err(e) => Err(e.into()),
    }
}

pub async fn get_today_time_totals() -> Result<(i64, i64)> {
    let conn = database::get_connection()?;
    
    // Phase 2 Spec: Total Work = Σ(session clock_in→clock_out) in range
    let mut work_stmt = conn.prepare(
        "SELECT COALESCE(SUM(
            CASE 
                WHEN ended_at IS NOT NULL THEN 
                    (strftime('%s', ended_at) - strftime('%s', started_at))
                ELSE 
                    (strftime('%s', 'now') - strftime('%s', started_at))
            END
        ), 0) as total_work_time
         FROM work_sessions 
         WHERE DATE(started_at) = DATE('now')"
    )?;
    
    let total_work_time: i64 = work_stmt.query_row([], |row| {
        Ok(row.get::<_, i64>(0)?)
    })?;
    
    // Phase 2 Spec: Idle = minutes with no input ≥ threshold while clocked in
    let mut idle_stmt = conn.prepare(
        "SELECT COALESCE(SUM(
            CASE 
                WHEN end_time IS NOT NULL THEN 
                    (strftime('%s', end_time) - strftime('%s', start_time))
                ELSE 
                    (strftime('%s', 'now') - strftime('%s', start_time))
            END
        ), 0) as total_idle_time
         FROM app_usage_sessions 
         WHERE DATE(start_time) = DATE('now') AND is_idle = 1"
    )?;
    
    let idle_time: i64 = idle_stmt.query_row([], |row| {
        Ok(row.get::<_, i64>(0)?)
    })?;
    
    // Phase 2 Spec: Active = Work − Idle
    let active_time = total_work_time - idle_time;
    
    // Ensure active time is not negative (in case of calculation errors)
    let active_time = active_time.max(0);
    
    
    Ok((active_time, idle_time))
}

