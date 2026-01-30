use crate::db::{DbConnection, Task};
use crate::error::{AppError, Result};
use tauri::State;

fn get_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

#[tauri::command]
pub fn get_tasks_by_project(db: State<DbConnection>, project_id: i64) -> Result<Vec<Task>> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, project_id, section_id, title, description, completed, position, total_time_seconds, deadline, created_at, updated_at
         FROM tasks WHERE project_id = ? ORDER BY position ASC"
    )?;

    let tasks = stmt
        .query_map([project_id], |row| {
            Ok(Task {
                id: row.get(0)?,
                project_id: row.get(1)?,
                section_id: row.get(2)?,
                title: row.get(3)?,
                description: row.get(4)?,
                completed: row.get(5)?,
                position: row.get(6)?,
                total_time_seconds: row.get(7)?,
                deadline: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(tasks)
}

#[tauri::command]
pub fn get_unassigned_tasks(db: State<DbConnection>) -> Result<Vec<Task>> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, project_id, section_id, title, description, completed, position, total_time_seconds, deadline, created_at, updated_at
         FROM tasks WHERE project_id IS NULL ORDER BY position ASC"
    )?;

    let tasks = stmt
        .query_map([], |row| {
            Ok(Task {
                id: row.get(0)?,
                project_id: row.get(1)?,
                section_id: row.get(2)?,
                title: row.get(3)?,
                description: row.get(4)?,
                completed: row.get(5)?,
                position: row.get(6)?,
                total_time_seconds: row.get(7)?,
                deadline: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(tasks)
}

#[tauri::command]
pub fn get_tasks_by_section(db: State<DbConnection>, section_id: i64) -> Result<Vec<Task>> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, project_id, section_id, title, description, completed, position, total_time_seconds, deadline, created_at, updated_at
         FROM tasks WHERE section_id = ? ORDER BY position ASC"
    )?;

    let tasks = stmt
        .query_map([section_id], |row| {
            Ok(Task {
                id: row.get(0)?,
                project_id: row.get(1)?,
                section_id: row.get(2)?,
                title: row.get(3)?,
                description: row.get(4)?,
                completed: row.get(5)?,
                position: row.get(6)?,
                total_time_seconds: row.get(7)?,
                deadline: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(tasks)
}

#[tauri::command]
pub fn create_task(
    db: State<DbConnection>,
    project_id: Option<i64>,
    section_id: Option<i64>,
    title: String,
    description: Option<String>,
) -> Result<Task> {
    let conn = db.lock();
    let now = get_timestamp();

    let max_position: i32 = if let Some(sid) = section_id {
        conn.query_row(
            "SELECT COALESCE(MAX(position), -1) FROM tasks WHERE section_id = ?",
            [sid],
            |row| row.get(0)
        )
    } else if let Some(pid) = project_id {
        conn.query_row(
            "SELECT COALESCE(MAX(position), -1) FROM tasks WHERE project_id = ? AND section_id IS NULL",
            [pid],
            |row| row.get(0)
        )
    } else {
        conn.query_row(
            "SELECT COALESCE(MAX(position), -1) FROM tasks WHERE project_id IS NULL AND section_id IS NULL",
            [],
            |row| row.get(0)
        )
    }
    .unwrap_or(0);

    conn.execute(
        "INSERT INTO tasks (project_id, section_id, title, description, position, deadline, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        (
            project_id,
            section_id,
            &title,
            &description,
            max_position + 1,
            Option::<String>::None,
            now,
            now,
        ),
    )?;

    let id = conn.last_insert_rowid();

    Ok(Task {
        id,
        project_id,
        section_id,
        title,
        description,
        completed: false,
        position: max_position + 1,
        total_time_seconds: 0,
        deadline: None,
        created_at: now,
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_task(
    db: State<DbConnection>,
    id: i64,
    title: Option<String>,
    description: Option<String>,
    completed: Option<bool>,
) -> Result<()> {
    let conn = db.lock();
    let now = get_timestamp();

    let mut stmt = conn.prepare("SELECT title, description, completed FROM tasks WHERE id = ?")?;

    let (current_title, current_description, current_completed) = stmt
        .query_row([id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, bool>(2)?,
            ))
        })
        .map_err(|_| AppError::NotFound(format!("Task with id {} not found", id)))?;

    conn.execute(
        "UPDATE tasks SET title = ?, description = ?, completed = ?, updated_at = ? WHERE id = ?",
        (
            title.unwrap_or(current_title),
            description.or(current_description),
            completed.unwrap_or(current_completed),
            now,
            id,
        ),
    )?;

    Ok(())
}

#[tauri::command]
pub fn delete_task(db: State<DbConnection>, id: i64) -> Result<()> {
    let conn = db.lock();
    let rows = conn.execute("DELETE FROM tasks WHERE id = ?", [id])?;

    if rows == 0 {
        return Err(AppError::NotFound(format!("Task with id {} not found", id)));
    }

    Ok(())
}

#[tauri::command]
pub fn toggle_task_completion(db: State<DbConnection>, id: i64) -> Result<bool> {
    let conn = db.lock();
    let now = get_timestamp();

    let completed: bool = conn
        .query_row("SELECT completed FROM tasks WHERE id = ?", [id], |row| {
            row.get(0)
        })
        .map_err(|_| AppError::NotFound(format!("Task with id {} not found", id)))?;

    let new_completed = !completed;

    conn.execute(
        "UPDATE tasks SET completed = ?, updated_at = ? WHERE id = ?",
        (new_completed, now, id),
    )?;

    Ok(new_completed)
}

#[tauri::command]
pub fn reorder_tasks(db: State<DbConnection>, task_ids: Vec<i64>) -> Result<()> {
    let conn = db.lock();

    for (position, id) in task_ids.iter().enumerate() {
        conn.execute(
            "UPDATE tasks SET position = ? WHERE id = ?",
            (position as i32, *id),
        )?;
    }

    Ok(())
}

#[tauri::command]
pub fn reset_task_time(db: State<DbConnection>, id: i64) -> Result<()> {
    let conn = db.lock();

    // Get current task info
    let (project_id, section_id, current_time): (i64, Option<i64>, i64) = conn
        .query_row(
            "SELECT project_id, section_id, total_time_seconds FROM tasks WHERE id = ?",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|_| AppError::NotFound(format!("Task with id {} not found", id)))?;

    // Delete all time entries for this task
    conn.execute("DELETE FROM time_entries WHERE task_id = ?", [id])?;

    // Reset task's total time
    conn.execute("UPDATE tasks SET total_time_seconds = 0 WHERE id = ?", [id])?;

    // Update project's total time (subtract the task's time)
    conn.execute(
        "UPDATE projects SET total_time_seconds = total_time_seconds - ? WHERE id = ?",
        (current_time, project_id),
    )?;

    // If task belongs to a section, update section's total time too
    if let Some(sid) = section_id {
        conn.execute(
            "UPDATE sections SET total_time_seconds = total_time_seconds - ? WHERE id = ?",
            (current_time, sid),
        )?;
    }

    Ok(())
}
