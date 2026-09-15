use crate::db::{Project, Section};
use crate::error::{AppError, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectWithStats {
    pub id: i64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub color: String,
    pub position: i32,
    pub total_time_seconds: i64,
    pub formatted_total_time: String,
    pub active_task_count: i64,
    pub completed_task_count: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

pub fn format_duration(seconds: i64) -> String {
    let seconds = seconds.max(0);
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

pub fn list_projects(conn: &Connection, include_system: bool) -> Result<Vec<ProjectWithStats>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            p.id,
            p.name,
            p.description,
            p.color,
            p.position,
            p.total_time_seconds,
            p.created_at,
            p.updated_at,
            COALESCE(SUM(CASE WHEN t.completed = 0 AND (t.is_system = 0 OR t.is_system IS NULL) THEN 1 ELSE 0 END), 0) as active_task_count,
            COALESCE(SUM(CASE WHEN t.completed = 1 AND (t.is_system = 0 OR t.is_system IS NULL) THEN 1 ELSE 0 END), 0) as completed_task_count
        FROM projects p
        LEFT JOIN tasks t ON t.project_id = p.id
        WHERE (?1 OR p.is_system = 0)
        GROUP BY p.id
        ORDER BY p.position ASC
        "#,
    )?;

    let projects = stmt
        .query_map([include_system], |row| {
            let total_time_seconds: i64 = row.get(5)?;
            Ok(ProjectWithStats {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                color: row.get(3)?,
                position: row.get(4)?,
                total_time_seconds,
                formatted_total_time: format_duration(total_time_seconds),
                active_task_count: row.get(8)?,
                completed_task_count: row.get(9)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(projects)
}

pub fn get_all_projects(conn: &Connection) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, color, position, total_time_seconds, created_at, updated_at
         FROM projects
         WHERE is_system = 0
         ORDER BY position ASC",
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

pub fn get_project(conn: &Connection, id: i64) -> Result<Project> {
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

pub fn find_project_by_name(conn: &Connection, name: &str) -> Result<Option<Project>> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let mut stmt = conn.prepare(
        "SELECT id, name, description, color, position, total_time_seconds, created_at, updated_at
         FROM projects
         WHERE LOWER(name) = LOWER(?1) AND is_system = 0
         LIMIT 1",
    )?;

    let result = stmt.query_row([trimmed], |row| {
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
    });

    match result {
        Ok(project) => Ok(Some(project)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

pub fn find_section_by_name(
    conn: &Connection,
    project_id: i64,
    name: &str,
) -> Result<Option<Section>> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, position, total_time_seconds, created_at
         FROM sections
         WHERE project_id = ?1 AND LOWER(name) = LOWER(?2)
         LIMIT 1",
    )?;

    let result = stmt.query_row(params![project_id, trimmed], |row| {
        Ok(Section {
            id: row.get(0)?,
            project_id: row.get(1)?,
            name: row.get(2)?,
            position: row.get(3)?,
            total_time_seconds: row.get(4)?,
            created_at: row.get(5)?,
        })
    });

    match result {
        Ok(section) => Ok(Some(section)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(error.into()),
    }
}
