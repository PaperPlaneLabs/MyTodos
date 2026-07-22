use crate::db::DbConnection;
use crate::error::{AppError, Result};
use rusqlite::{params, Connection, OptionalExtension};

const CURRENT_TASK_RELATED_LABEL: &str = "Current task related";

fn normalize_name(name: &str) -> Result<String> {
    let normalized = name.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return Err(AppError::InvalidInput(
            "AFK category name cannot be empty".to_string(),
        ));
    }
    if normalized.eq_ignore_ascii_case(CURRENT_TASK_RELATED_LABEL) {
        return Err(AppError::InvalidInput(format!(
            "{CURRENT_TASK_RELATED_LABEL} is a built-in option"
        )));
    }
    Ok(normalized)
}

fn list_from_conn(conn: &Connection) -> Result<Vec<String>> {
    let mut statement = conn.prepare(
        "SELECT name
         FROM afk_categories
         ORDER BY position, id",
    )?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub(crate) fn ensure_category_in_conn(conn: &Connection, name: &str) -> Result<String> {
    let normalized = normalize_name(name)?;
    if let Some(existing) = conn
        .query_row(
            "SELECT name FROM afk_categories WHERE name = ? COLLATE NOCASE",
            [&normalized],
            |row| row.get::<_, String>(0),
        )
        .optional()?
    {
        return Ok(existing);
    }

    let next_position: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM afk_categories",
        [],
        |row| row.get(0),
    )?;
    let now: i64 = conn.query_row("SELECT unixepoch()", [], |row| row.get(0))?;
    conn.execute(
        "INSERT INTO afk_categories (name, position, created_at)
         VALUES (?, ?, ?)",
        params![normalized, next_position, now],
    )?;
    Ok(normalized)
}

pub fn list_categories(db: &DbConnection) -> Result<Vec<String>> {
    let conn = db.lock();
    list_from_conn(&conn)
}

pub fn add_category(db: &DbConnection, name: &str) -> Result<String> {
    let conn = db.lock();
    ensure_category_in_conn(&conn, name)
}

pub fn merge_categories(db: &DbConnection, names: Vec<String>) -> Result<Vec<String>> {
    let mut conn = db.lock();
    let transaction = conn.transaction()?;
    for name in names {
        ensure_category_in_conn(&transaction, &name)?;
    }
    transaction.commit()?;
    list_from_conn(&conn)
}

pub fn remove_category(db: &DbConnection, name: &str) -> Result<()> {
    let normalized = normalize_name(name)?;
    let conn = db.lock();
    conn.execute(
        "DELETE FROM afk_categories WHERE name = ? COLLATE NOCASE",
        [&normalized],
    )?;
    Ok(())
}
