use crate::error::Result;
use rusqlite::Connection;

pub fn get_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

pub fn apply_parent_time_delta(conn: &Connection, task_id: i64, delta: i64) -> Result<()> {
    let (project_id, section_id): (Option<i64>, Option<i64>) = conn.query_row(
        "SELECT project_id, section_id FROM tasks WHERE id = ?",
        [task_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    if let Some(project_id) = project_id {
        conn.execute(
            "UPDATE projects SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
            (delta, project_id),
        )?;
    }

    if let Some(section_id) = section_id {
        conn.execute(
            "UPDATE sections SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
            (delta, section_id),
        )?;
    }

    Ok(())
}

pub fn apply_task_and_parent_time_delta(conn: &Connection, task_id: i64, delta: i64) -> Result<()> {
    conn.execute(
        "UPDATE tasks SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
        (delta, task_id),
    )?;

    apply_parent_time_delta(conn, task_id, delta)
}
