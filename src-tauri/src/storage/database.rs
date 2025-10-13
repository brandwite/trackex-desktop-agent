use anyhow::Result;
use rusqlite::Connection;
use std::path::PathBuf;

fn get_db_path() -> Result<PathBuf> {
    let mut path = dirs::data_dir().ok_or_else(|| anyhow::anyhow!("Failed to get data directory"))?;
    path.push("TrackEx");
    std::fs::create_dir_all(&path)?;
    path.push("agent.db");
    Ok(path)
}

pub async fn init() -> Result<()> {
    let db_path = get_db_path()?;
    let conn = Connection::open(&db_path)?;
    
    // Create tables
    conn.execute(
        "CREATE TABLE IF NOT EXISTS consent (
            id INTEGER PRIMARY KEY,
            accepted BOOLEAN NOT NULL DEFAULT 0,
            version TEXT NOT NULL,
            accepted_at DATETIME,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS event_queue (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            event_type TEXT NOT NULL,
            event_data TEXT NOT NULL,
            timestamp DATETIME NOT NULL,
            processed BOOLEAN NOT NULL DEFAULT 0,
            retry_count INTEGER NOT NULL DEFAULT 0,
            max_retries INTEGER NOT NULL DEFAULT 3,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

            conn.execute(
                "CREATE TABLE IF NOT EXISTS heartbeat_queue (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    heartbeat_data TEXT NOT NULL,
                    timestamp DATETIME NOT NULL,
                    processed BOOLEAN NOT NULL DEFAULT 0,
                    retry_count INTEGER NOT NULL DEFAULT 0,
                    max_retries INTEGER NOT NULL DEFAULT 3,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
                )",
                [],
            )?;

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
                    synced BOOLEAN NOT NULL DEFAULT 0,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
                )",
                [],
            )?;

            // Migration: Recreate app_usage_sessions table with correct schema
            // This ensures the table has the right structure for the app usage tracker
            let table_exists = conn.query_row(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='app_usage_sessions'",
                [],
                |row| Ok(row.get::<_, String>(0)?)
            ).is_ok();

            if table_exists {
                
                // Drop existing table (data will be lost, but this is for development)
                conn.execute("DROP TABLE app_usage_sessions", [])?;
                
                // Recreate with correct schema including synced column
                conn.execute(
                    "CREATE TABLE app_usage_sessions (
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
                        synced BOOLEAN NOT NULL DEFAULT 0,
                        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
                    )",
                    [],
                )?;
                
            }

            conn.execute(
                "CREATE TABLE IF NOT EXISTS work_sessions (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    started_at DATETIME NOT NULL,
                    ended_at DATETIME,
                    is_active BOOLEAN NOT NULL DEFAULT 1,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
                )",
                [],
            )?;

    Ok(())
}

pub fn get_connection() -> Result<Connection> {
    let db_path = get_db_path()?;
    let conn = Connection::open(&db_path)?;
    Ok(conn)
}