#![allow(dead_code)]

use parking_lot::Mutex;
use rusqlite::Connection;
use std::sync::Arc;

pub type DbConnection = Arc<Mutex<Connection>>;

/// Creates an in-memory SQLite database for testing
pub fn setup_test_db() -> DbConnection {
    let conn = Connection::open_in_memory().expect("Failed to open in-memory database");

    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON", [])
        .expect("Failed to enable foreign keys");

    // Initialize schema
    my_todos_lib::db::schema::initialize_schema(&conn).expect("Failed to initialize schema");

    Arc::new(Mutex::new(conn))
}

/// Get current timestamp
pub fn get_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

/// Create a test project and return its ID
pub fn create_test_project(db: &DbConnection, name: &str) -> i64 {
    let conn = db.lock();
    let now = get_timestamp();

    let max_position: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(position), -1) FROM projects",
            [],
            |row| row.get(0),
        )
        .unwrap_or(-1);

    conn.execute(
        "INSERT INTO projects (name, color, position, created_at, updated_at, total_time_seconds)
         VALUES (?, '#6366f1', ?, ?, ?, 0)",
        (name, max_position + 1, now, now),
    )
    .expect("Failed to create test project");

    conn.last_insert_rowid()
}

/// Create a test section and return its ID
pub fn create_test_section(db: &DbConnection, project_id: i64, name: &str) -> i64 {
    let conn = db.lock();
    let now = get_timestamp();

    let max_position: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(position), -1) FROM sections WHERE project_id = ?",
            [project_id],
            |row| row.get(0),
        )
        .unwrap_or(-1);

    conn.execute(
        "INSERT INTO sections (project_id, name, position, created_at, total_time_seconds)
         VALUES (?, ?, ?, ?, 0)",
        (project_id, name, max_position + 1, now),
    )
    .expect("Failed to create test section");

    conn.last_insert_rowid()
}

/// Create a test task and return its ID
pub fn create_test_task(
    db: &DbConnection,
    project_id: Option<i64>,
    section_id: Option<i64>,
    title: &str,
) -> i64 {
    let conn = db.lock();
    let now = get_timestamp();

    conn.execute(
        "INSERT INTO tasks (project_id, section_id, title, completed, position, created_at, updated_at, total_time_seconds)
         VALUES (?, ?, ?, 0, 0, ?, ?, 0)",
        (project_id, section_id, title, now, now),
    )
    .expect("Failed to create test task");

    conn.last_insert_rowid()
}

/// Get task total time
pub fn get_task_time(db: &DbConnection, task_id: i64) -> i64 {
    let conn = db.lock();
    conn.query_row(
        "SELECT total_time_seconds FROM tasks WHERE id = ?",
        [task_id],
        |row| row.get(0),
    )
    .expect("Failed to get task time")
}

/// Get project total time
pub fn get_project_time(db: &DbConnection, project_id: i64) -> i64 {
    let conn = db.lock();
    conn.query_row(
        "SELECT total_time_seconds FROM projects WHERE id = ?",
        [project_id],
        |row| row.get(0),
    )
    .expect("Failed to get project time")
}

/// Get section total time
pub fn get_section_time(db: &DbConnection, section_id: i64) -> i64 {
    let conn = db.lock();
    conn.query_row(
        "SELECT total_time_seconds FROM sections WHERE id = ?",
        [section_id],
        |row| row.get(0),
    )
    .expect("Failed to get section time")
}

/// Check if active timer exists
pub fn has_active_timer(db: &DbConnection) -> bool {
    let conn = db.lock();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM active_timer WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    count > 0
}

/// Get active timer details
pub fn get_active_timer_raw(db: &DbConnection) -> Option<(i64, i64, i64, bool)> {
    let conn = db.lock();
    conn.query_row(
        "SELECT task_id, started_at, elapsed_seconds, is_running FROM active_timer WHERE id = 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    )
    .ok()
}

/// Count time entries for a task
pub fn count_time_entries(db: &DbConnection, task_id: i64) -> i64 {
    let conn = db.lock();
    conn.query_row(
        "SELECT COUNT(*) FROM time_entries WHERE task_id = ?",
        [task_id],
        |row| row.get(0),
    )
    .unwrap_or(0)
}
