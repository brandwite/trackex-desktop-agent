#![allow(dead_code)]

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::params;
use serde_json::Value;

use super::database;

#[derive(Debug)]
#[allow(dead_code)]
pub struct QueuedEvent {
    pub id: i64,
    pub event_type: String,
    pub event_data: Value,
    pub timestamp: DateTime<Utc>,
    pub retry_count: i32,
    pub max_retries: i32,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct QueuedHeartbeat {
    pub id: i64,
    pub heartbeat_data: Value,
    pub timestamp: DateTime<Utc>,
    pub retry_count: i32,
    pub max_retries: i32,
}

// Heartbeat queue operations
pub async fn queue_heartbeat(heartbeat_data: &Value) -> Result<()> {
    let conn = database::get_connection()?;
    
    let now = Utc::now();
    let data_str = serde_json::to_string(heartbeat_data)?;
    
    conn.execute(
        "INSERT INTO heartbeat_queue (heartbeat_data, timestamp) 
         VALUES (?1, ?2)",
        params![data_str, now],
    )?;
    
    Ok(())
}

pub async fn get_pending_heartbeats() -> Result<Vec<QueuedHeartbeat>> {
    let conn = database::get_connection()?;
    
    let mut stmt = conn.prepare(
        "SELECT id, heartbeat_data, timestamp, retry_count, max_retries 
         FROM heartbeat_queue 
         WHERE processed = 0 AND retry_count < max_retries
         ORDER BY timestamp ASC
         LIMIT 10"
    )?;
    
    let heartbeat_iter = stmt.query_map([], |row| {
        let heartbeat_data: String = row.get(1)?;
        let heartbeat_data: Value = serde_json::from_str(&heartbeat_data)
            .map_err(|_| rusqlite::Error::InvalidColumnType(1, "heartbeat_data".to_string(), rusqlite::types::Type::Text))?;
        
        Ok(QueuedHeartbeat {
            id: row.get(0)?,
            heartbeat_data,
            timestamp: row.get(2)?,
            retry_count: row.get(3)?,
            max_retries: row.get(4)?,
        })
    })?;
    
    let mut heartbeats = Vec::new();
    for heartbeat in heartbeat_iter {
        heartbeats.push(heartbeat?);
    }
    
    Ok(heartbeats)
}

pub async fn mark_heartbeat_processed(id: i64) -> Result<()> {
    let conn = database::get_connection()?;
    
    conn.execute(
        "UPDATE heartbeat_queue SET processed = 1 WHERE id = ?1",
        params![id],
    )?;
    
    Ok(())
}

pub async fn mark_heartbeat_failed(id: i64) -> Result<()> {
    let conn = database::get_connection()?;
    
    conn.execute(
        "UPDATE heartbeat_queue 
         SET retry_count = retry_count + 1 
         WHERE id = ?1",
        params![id],
    )?;
    
    Ok(())
}

// Event queue operations
pub async fn queue_event(event_type: &str, event_data: &Value) -> Result<()> {
    let conn = database::get_connection()?;
    
    let now = Utc::now();
    let data_str = serde_json::to_string(event_data)?;
    
    conn.execute(
        "INSERT INTO event_queue (event_type, event_data, timestamp) 
         VALUES (?1, ?2, ?3)",
        params![event_type, data_str, now],
    )?;
    
    Ok(())
}

pub async fn get_pending_events() -> Result<Vec<QueuedEvent>> {
    let conn = database::get_connection()?;
    
    let mut stmt = conn.prepare(
        "SELECT id, event_type, event_data, timestamp, retry_count, max_retries 
         FROM event_queue 
         WHERE processed = 0 AND retry_count < max_retries
         ORDER BY timestamp ASC
         LIMIT 10"
    )?;
    
    let event_iter = stmt.query_map([], |row| {
        let event_data: String = row.get(2)?;
        let event_data: Value = serde_json::from_str(&event_data)
            .map_err(|_| rusqlite::Error::InvalidColumnType(2, "event_data".to_string(), rusqlite::types::Type::Text))?;
        
        Ok(QueuedEvent {
            id: row.get(0)?,
            event_type: row.get(1)?,
            event_data,
            timestamp: row.get(3)?,
            retry_count: row.get(4)?,
            max_retries: row.get(5)?,
        })
    })?;
    
    let mut events = Vec::new();
    for event in event_iter {
        events.push(event?);
    }
    
    Ok(events)
}

pub async fn mark_event_processed(event_id: i64) -> Result<()> {
    let conn = database::get_connection()?;
    
    conn.execute(
        "UPDATE event_queue SET processed = 1 WHERE id = ?1",
        params![event_id],
    )?;
    
    Ok(())
}

pub async fn mark_event_failed(event_id: i64) -> Result<()> {
    let conn = database::get_connection()?;
    
    conn.execute(
        "UPDATE event_queue 
         SET retry_count = retry_count + 1 
         WHERE id = ?1",
        params![event_id],
    )?;
    
    Ok(())
}