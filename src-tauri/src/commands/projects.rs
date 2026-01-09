use crate::db::{DbConnection, Project, ProjectStats};
use crate::error::{AppError, Result};
use tauri::State;

fn get_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

#[tauri::command]
pub fn get_all_projects(db: State<DbConnection>) -> Result<Vec<Project>> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, name, description, color, position, total_time_seconds, created_at, updated_at
         FROM projects ORDER BY position ASC",
    )?;

    let projects = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                color: row.get(3)?,
                position: row.get(4)?,
                total_time_seconds: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(projects)
}

#[tauri::command]
pub fn get_project(db: State<DbConnection>, id: i64) -> Result<Project> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, name, description, color, position, total_time_seconds, created_at, updated_at
         FROM projects WHERE id = ?",
    )?;

    stmt.query_row([id], |row| {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            color: row.get(3)?,
            position: row.get(4)?,
            total_time_seconds: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })
    .map_err(|_| AppError::NotFound(format!("Project with id {} not found", id)))
}

#[tauri::command]
pub fn create_project(
    db: State<DbConnection>,
    name: String,
    description: Option<String>,
    color: Option<String>,
) -> Result<Project> {
    let conn = db.lock();
    let now = get_timestamp();
    let color = color.unwrap_or_else(|| "#6366f1".to_string());

    let max_position: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(position), -1) FROM projects",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    conn.execute(
        "INSERT INTO projects (name, description, color, position, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)",
        (&name, &description, &color, max_position + 1, now, now),
    )?;

    let id = conn.last_insert_rowid();

    Ok(Project {
        id,
        name,
        description,
        color,
        position: max_position + 1,
        total_time_seconds: 0,
        created_at: now,
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_project(
    db: State<DbConnection>,
    id: i64,
    name: Option<String>,
    description: Option<String>,
    color: Option<String>,
) -> Result<()> {
    let conn = db.lock();
    let now = get_timestamp();

    // Fetch current project values within the same lock scope
    let mut stmt = conn.prepare("SELECT name, description, color FROM projects WHERE id = ?")?;

    let (current_name, current_description, current_color) = stmt
        .query_row([id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|_| AppError::NotFound(format!("Project with id {} not found", id)))?;

    drop(stmt);

    conn.execute(
        "UPDATE projects SET name = ?, description = ?, color = ?, updated_at = ? WHERE id = ?",
        (
            name.unwrap_or(current_name),
            description.or(current_description),
            color.unwrap_or(current_color),
            now,
            id,
        ),
    )?;

    Ok(())
}

#[tauri::command]
pub fn delete_project(db: State<DbConnection>, id: i64) -> Result<()> {
    let conn = db.lock();
    let rows = conn.execute("DELETE FROM projects WHERE id = ?", [id])?;

    if rows == 0 {
        return Err(AppError::NotFound(format!(
            "Project with id {} not found",
            id
        )));
    }

    Ok(())
}

#[tauri::command]
pub fn reorder_projects(db: State<DbConnection>, project_ids: Vec<i64>) -> Result<()> {
    let conn = db.lock();

    for (position, id) in project_ids.iter().enumerate() {
        conn.execute(
            "UPDATE projects SET position = ? WHERE id = ?",
            (position as i32, *id),
        )?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_project_stats(db: State<DbConnection>, project_id: i64) -> Result<ProjectStats> {
    let conn = db.lock();

    let mut stmt = conn.prepare(
        "SELECT COUNT(*) as task_count,
                SUM(CASE WHEN completed = 1 THEN 1 ELSE 0 END) as completed_count,
                COALESCE(SUM(total_time_seconds), 0) as total_time
         FROM tasks WHERE project_id = ?",
    )?;

    let stats = stmt.query_row([project_id], |row| {
        Ok(ProjectStats {
            task_count: row.get(0)?,
            completed_count: row.get(1)?,
            total_time_seconds: row.get(2)?,
        })
    })?;

    Ok(stats)
}
