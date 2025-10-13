use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::database;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsentRecord {
    pub accepted: bool,
    pub version: String,
    pub accepted_at: Option<DateTime<Utc>>,
}

pub async fn accept_consent(version: &str) -> Result<()> {
    let conn = database::get_connection()?;
    
    let now = Utc::now().to_rfc3339();
    
    // Insert or update consent record
    conn.execute(
        "INSERT OR REPLACE INTO consent (id, accepted, version, accepted_at) 
         VALUES (1, 1, ?1, ?2)",
        params![version, now],
    )?;
    
    Ok(())
}

pub async fn get_consent_status() -> Result<ConsentRecord> {
    let conn = database::get_connection()?;
    
    let mut stmt = conn.prepare(
        "SELECT accepted, version, accepted_at FROM consent WHERE id = 1"
    )?;
    
    match stmt.query_row([], |row| {
        let accepted: bool = row.get(0)?;
        let version: String = row.get(1)?;
        let accepted_at_str: Option<String> = row.get(2)?;
        
        let accepted_at = accepted_at_str
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc));
        
        Ok(ConsentRecord {
            accepted,
            version,
            accepted_at,
        })
    }) {
        Ok(record) => Ok(record),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // No consent record exists, return default
            Ok(ConsentRecord {
                accepted: false,
                version: "1.0.0".to_string(),
                accepted_at: None,
            })
        }
        Err(e) => Err(e.into()),
    }
}