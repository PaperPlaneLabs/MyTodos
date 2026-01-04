use parking_lot::Mutex;
use rusqlite::Connection;
use std::sync::Arc;
use crate::error::Result;

pub type DbConnection = Arc<Mutex<Connection>>;

pub fn initialize_connection() -> Result<DbConnection> {
    let app_data_dir = std::env::var("APPDATA")
        .or_else(|_| std::env::var("HOME"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("."));

    let db_dir = app_data_dir.join("my-todos");
    std::fs::create_dir_all(&db_dir)
        .map_err(|e| crate::error::AppError::InvalidInput(format!("Could not create database directory: {}", e)))?;

    let db_path = db_dir.join("todos.db");
    let conn = Connection::open(db_path)?;

    conn.execute("PRAGMA foreign_keys = ON", [])?;

    Ok(Arc::new(Mutex::new(conn)))
}
