use parking_lot::Mutex;
use rusqlite::Connection;
use std::sync::Arc;
use std::path::PathBuf;
use crate::error::Result;

pub type DbConnection = Arc<Mutex<Connection>>;

fn get_database_path() -> PathBuf {
    #[cfg(target_os = "android")]
    {
        // On Android, use the app's cache directory which is writable
        // This is set by the Android runtime and available through env var or relative path
        // We use a relative path to the app's data directory
        PathBuf::from("./my-todos")
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

    std::fs::create_dir_all(&db_dir)
        .map_err(|e| crate::error::AppError::InvalidInput(format!("Could not create database directory: {}", e)))?;

    let db_path = db_dir.join("todos.db");
    let conn = Connection::open(db_path)?;

    conn.execute("PRAGMA foreign_keys = ON", [])?;

    Ok(Arc::new(Mutex::new(conn)))
}
