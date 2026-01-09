use crate::error::Result;
use parking_lot::Mutex;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Arc;

pub type DbConnection = Arc<Mutex<Connection>>;

fn get_database_path() -> PathBuf {
    #[cfg(target_os = "android")]
    {
        // On Android, use the app's files directory which is writable
        // Use /data/data/<package>/files as the base directory
        let base_path = std::env::var("HOME")
            .or_else(|_| std::env::var("TMPDIR"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/data/local/tmp"));

        base_path.join("my-todos")
    }

    #[cfg(not(target_os = "android"))]
    {
        // On desktop platforms, use the standard data directory
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("my-todos")
    }
}

pub fn initialize_connection() -> Result<DbConnection> {
    let db_dir = get_database_path();

    std::fs::create_dir_all(&db_dir).map_err(|e| {
        crate::error::AppError::InvalidInput(format!("Could not create database directory: {}", e))
    })?;

    let db_path = db_dir.join("todos.db");
    let conn = Connection::open(db_path)?;

    conn.execute("PRAGMA foreign_keys = ON", [])?;

    Ok(Arc::new(Mutex::new(conn)))
}
