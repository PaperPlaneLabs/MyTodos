use crate::db::Task;
use crate::error::{AppError, Result};
use rusqlite::{params, Connection, Row};

fn now_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

fn task_from_row(row: &Row<'_>) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        section_id: row.get("section_id")?,
        title: row.get("title")?,
        description: row.get("description")?,
        completed: row.get("completed")?,
        position: row.get("position")?,
        total_time_seconds: row.get("total_time_seconds")?,
        deadline: row.get("deadline")?,
        google_event_id: row.get("google_event_id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn create_task(
    conn: &Connection,
    project_id: Option<i64>,
    section_id: Option<i64>,
    title: String,
    description: Option<String>,
    deadline: Option<String>,
) -> Result<Task> {
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(AppError::InvalidInput("Task title cannot be empty".into()));
    }

    let now = now_timestamp();
    let max_position: i32 = if let Some(sid) = section_id {
        conn.query_row(
            "SELECT COALESCE(MAX(position), -1) FROM tasks WHERE section_id = ?",
            [sid],
            |row| row.get(0),
        )
    } else if let Some(pid) = project_id {
        conn.query_row(
            "SELECT COALESCE(MAX(position), -1) FROM tasks WHERE project_id = ? AND section_id IS NULL",
            [pid],
            |row| row.get(0),
        )
    } else {
        conn.query_row(
            "SELECT COALESCE(MAX(position), -1) FROM tasks WHERE project_id IS NULL AND section_id IS NULL",
            [],
            |row| row.get(0),
        )
    }
    .unwrap_or(0);

    conn.execute(
        "INSERT INTO tasks (project_id, section_id, title, description, position, deadline, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            project_id,
            section_id,
            title,
            description,
            max_position + 1,
            deadline,
            now,
            now
        ],
    )?;

    get_task(conn, conn.last_insert_rowid())
}

pub fn get_task(conn: &Connection, id: i64) -> Result<Task> {
    conn.query_row(
        "SELECT id, project_id, section_id, title, description, completed, position, total_time_seconds, deadline, google_event_id, created_at, updated_at
         FROM tasks
         WHERE id = ?1 AND is_system = 0",
        [id],
        task_from_row,
    )
    .map_err(|_| AppError::NotFound(format!("Task with id {} not found", id)))
}

pub fn set_task_deadline(
    conn: &Connection,
    task_id: i64,
    deadline: Option<String>,
) -> Result<Task> {
    let now = now_timestamp();
    let rows = conn.execute(
        "UPDATE tasks SET deadline = ?1, updated_at = ?2 WHERE id = ?3 AND is_system = 0",
        params![deadline, now, task_id],
    )?;

    if rows == 0 {
        return Err(AppError::NotFound(format!(
            "Task with id {} not found",
            task_id
        )));
    }

    get_task(conn, task_id)
}

pub fn set_task_completed(conn: &Connection, task_id: i64, completed: bool) -> Result<Task> {
    let now = now_timestamp();
    let rows = conn.execute(
        "UPDATE tasks
         SET completed = ?1,
             google_event_id = CASE WHEN ?1 THEN NULL ELSE google_event_id END,
             updated_at = ?2
         WHERE id = ?3 AND is_system = 0",
        params![completed, now, task_id],
    )?;

    if rows == 0 {
        return Err(AppError::NotFound(format!(
            "Task with id {} not found",
            task_id
        )));
    }

    get_task(conn, task_id)
}

pub fn list_due_tasks(conn: &Connection, start_date: &str, end_date: &str) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, section_id, title, description, completed, position, total_time_seconds, deadline, google_event_id, created_at, updated_at
         FROM tasks
         WHERE deadline BETWEEN ?1 AND ?2
           AND deadline IS NOT NULL
           AND is_system = 0
         ORDER BY deadline, position",
    )?;

    let tasks = stmt
        .query_map(params![start_date, end_date], task_from_row)?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(tasks)
}

pub fn find_tasks(
    conn: &Connection,
    query: &str,
    include_completed: bool,
    limit: i64,
) -> Result<Vec<Task>> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let limit = limit.clamp(1, 50);
    let like_query = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT id, project_id, section_id, title, description, completed, position, total_time_seconds, deadline, google_event_id, created_at, updated_at
         FROM tasks
         WHERE is_system = 0
           AND (?1 OR completed = 0)
           AND (title LIKE ?2 OR COALESCE(description, '') LIKE ?2)
         ORDER BY completed ASC, updated_at DESC
         LIMIT ?3",
    )?;

    let tasks = stmt
        .query_map(params![include_completed, like_query, limit], task_from_row)?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(tasks)
}
