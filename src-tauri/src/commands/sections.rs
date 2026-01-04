use crate::db::{DbConnection, Section};
use crate::error::{AppError, Result};
use tauri::State;

fn get_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

#[tauri::command]
pub fn get_sections_by_project(db: State<DbConnection>, project_id: i64) -> Result<Vec<Section>> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, position, total_time_seconds, created_at
         FROM sections WHERE project_id = ? ORDER BY position ASC"
    )?;

    let sections = stmt.query_map([project_id], |row| {
        Ok(Section {
            id: row.get(0)?,
            project_id: row.get(1)?,
            name: row.get(2)?,
            position: row.get(3)?,
            total_time_seconds: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?
    .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(sections)
}

#[tauri::command]
pub fn create_section(
    db: State<DbConnection>,
    project_id: i64,
    name: String,
) -> Result<Section> {
    let conn = db.lock();
    let now = get_timestamp();

    let max_position: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(position), -1) FROM sections WHERE project_id = ?",
            [project_id],
            |row| row.get(0)
        )
        .unwrap_or(0);

    conn.execute(
        "INSERT INTO sections (project_id, name, position, created_at)
         VALUES (?, ?, ?, ?)",
        (&project_id, &name, max_position + 1, now),
    )?;

    let id = conn.last_insert_rowid();

    Ok(Section {
        id,
        project_id,
        name,
        position: max_position + 1,
        total_time_seconds: 0,
        created_at: now,
    })
}

#[tauri::command]
pub fn update_section(
    db: State<DbConnection>,
    id: i64,
    name: String,
) -> Result<()> {
    let conn = db.lock();

    let rows = conn.execute(
        "UPDATE sections SET name = ? WHERE id = ?",
        (&name, id),
    )?;

    if rows == 0 {
        return Err(AppError::NotFound(format!("Section with id {} not found", id)));
    }

    Ok(())
}

#[tauri::command]
pub fn delete_section(db: State<DbConnection>, id: i64) -> Result<()> {
    let conn = db.lock();
    let rows = conn.execute("DELETE FROM sections WHERE id = ?", [id])?;

    if rows == 0 {
        return Err(AppError::NotFound(format!("Section with id {} not found", id)));
    }

    Ok(())
}

#[tauri::command]
pub fn reorder_sections(db: State<DbConnection>, section_ids: Vec<i64>) -> Result<()> {
    let conn = db.lock();

    for (position, id) in section_ids.iter().enumerate() {
        conn.execute(
            "UPDATE sections SET position = ? WHERE id = ?",
            (position as i32, *id),
        )?;
    }

    Ok(())
}
