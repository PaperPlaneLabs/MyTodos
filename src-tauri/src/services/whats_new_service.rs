use crate::commands::common::get_timestamp;
use crate::db::DbConnection;
use crate::error::{AppError, Result};
use rusqlite::OptionalExtension;

const LAST_SEEN_VERSION_KEY: &str = "whats_new_last_seen_version";

fn validate_version(version: &str) -> Result<&str> {
    let trimmed = version.trim();
    let parts = trimmed.split('.').collect::<Vec<_>>();
    if parts.len() != 3
        || parts
            .iter()
            .any(|part| part.is_empty() || part.parse::<u64>().is_err())
    {
        return Err(AppError::InvalidInput(
            "What's New version must use X.Y.Z format".to_string(),
        ));
    }
    Ok(trimmed)
}

pub fn get_last_seen_version(db: &DbConnection) -> Result<Option<String>> {
    let conn = db.lock();
    Ok(conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = ?",
            [LAST_SEEN_VERSION_KEY],
            |row| row.get(0),
        )
        .optional()?)
}

pub fn set_last_seen_version(db: &DbConnection, version: &str) -> Result<()> {
    let version = validate_version(version)?;
    let conn = db.lock();
    conn.execute(
        "INSERT INTO app_settings (key, value, updated_at)
         VALUES (?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET
             value = excluded.value,
             updated_at = excluded.updated_at",
        (LAST_SEEN_VERSION_KEY, version, get_timestamp()),
    )?;
    Ok(())
}
