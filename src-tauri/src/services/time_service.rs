use crate::commands::common::{apply_task_and_parent_time_delta, get_timestamp};
use crate::db::TimeEntry;
use crate::error::{AppError, Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTimeEntry {
    pub task_id: i64,
    pub task_title: String,
    pub project_name: Option<String>,
    pub project_color: Option<String>,
    pub total_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyAggregate {
    pub date: String,
    pub total_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTime {
    pub name: String,
    pub color: String,
    pub total_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeStats {
    pub today_tasks: Vec<TaskTimeEntry>,
    pub week_daily: Vec<DailyAggregate>,
    pub projects: Vec<ProjectTime>,
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

pub fn create_manual_entry(
    conn: &Connection,
    task_id: i64,
    duration_seconds: i64,
    note: Option<String>,
) -> Result<TimeEntry> {
    let task_exists: bool = conn
        .query_row("SELECT 1 FROM tasks WHERE id = ?", [task_id], |_| {
            Ok(true)
        })
        .unwrap_or(false);

    if !task_exists {
        return Err(AppError::NotFound(format!(
            "Task with id {} not found",
            task_id
        )));
    }

    if duration_seconds <= 0 {
        return Err(AppError::InvalidInput(
            "Duration must be positive".to_string(),
        ));
    }

    let now = get_timestamp();

    conn.execute(
        "INSERT INTO time_entries (task_id, entry_type, duration_seconds, note, created_at)
         VALUES (?, 'manual', ?, ?, ?)",
        (task_id, duration_seconds, &note, now),
    )?;

    let id = conn.last_insert_rowid();

    apply_task_and_parent_time_delta(conn, task_id, duration_seconds)?;

    Ok(TimeEntry {
        id,
        task_id,
        entry_type: "manual".to_string(),
        duration_seconds,
        started_at: None,
        ended_at: None,
        note,
        created_at: now,
    })
}

type ActiveTimerStatsRow = (i64, i64, String, Option<String>, Option<String>, i64);

fn get_start_of_today() -> i64 {
    let now = chrono::Local::now();
    now.date_naive()
        .and_hms_opt(0, 0, 0)
        .map(|dt| dt.and_local_timezone(chrono::Local).unwrap().timestamp())
        .unwrap_or(0)
}

fn get_start_of_week() -> i64 {
    use chrono::{Datelike, Local};
    let now = Local::now();
    let days_since_monday = now.weekday().num_days_from_monday();
    let monday = now.date_naive() - chrono::Duration::days(days_since_monday as i64);
    monday
        .and_hms_opt(0, 0, 0)
        .map(|dt| dt.and_local_timezone(Local).unwrap().timestamp())
        .unwrap_or(0)
}

pub fn get_active_timer_duration(conn: &rusqlite::Connection) -> Result<Option<ActiveTimerStatsRow>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            at.task_id,
            t.project_id,
            t.title,
            p.name as project_name,
            p.color as project_color,
            at.elapsed_seconds,
            at.started_at,
            at.is_running,
            at.timer_expires_at
        FROM active_timer at
        JOIN tasks t ON t.id = at.task_id
        LEFT JOIN projects p ON p.id = t.project_id
        WHERE at.id = 1
        "#,
    )?;

    let result = stmt.query_row([], |row| {
        let task_id: i64 = row.get(0)?;
        let project_id: i64 = row.get(1)?;
        let task_title: String = row.get(2)?;
        let project_name: Option<String> = row.get(3)?;
        let project_color: Option<String> = row.get(4)?;
        let elapsed_seconds: i64 = row.get(5)?;
        let started_at: i64 = row.get(6)?;
        let is_running: i64 = row.get(7)?;
        let timer_expires_at: Option<i64> = row.get(8)?;

        let duration = if is_running == 1 {
            let now = chrono::Local::now().timestamp();
            let effective_now = timer_expires_at
                .map(|expires_at| now.min(expires_at))
                .unwrap_or(now);
            elapsed_seconds + (effective_now - started_at).max(0)
        } else {
            elapsed_seconds
        };

        Ok((
            task_id,
            project_id,
            task_title,
            project_name,
            project_color,
            duration,
        ))
    });

    match result {
        Ok(data) => Ok(Some(data)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn get_time_stats(conn: &Connection, include_active_timer: bool) -> Result<TimeStats> {
    let today_start = get_start_of_today();
    let week_start = get_start_of_week();

    // Get today's time entries grouped by task
    let mut stmt = conn.prepare(
        r#"
        SELECT
            te.task_id,
            t.title,
            p.name as project_name,
            p.color as project_color,
            SUM(te.duration_seconds) as total_seconds
        FROM time_entries te
        JOIN tasks t ON t.id = te.task_id
        LEFT JOIN projects p ON p.id = t.project_id
        WHERE te.created_at >= ?
        GROUP BY te.task_id
        ORDER BY total_seconds DESC
        "#,
    )?;

    let mut today_tasks: Vec<TaskTimeEntry> = stmt
        .query_map([today_start], |row| {
            Ok(TaskTimeEntry {
                task_id: row.get(0)?,
                task_title: row.get(1)?,
                project_name: row.get(2)?,
                project_color: row.get(3)?,
                total_seconds: row.get(4)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    // Include active timer if requested
    let active_timer_info = if include_active_timer {
        get_active_timer_duration(conn)?
    } else {
        None
    };

    // Add active timer to today's tasks
    if let Some((task_id, _, task_title, project_name, project_color, duration)) =
        &active_timer_info
    {
        if let Some(task) = today_tasks.iter_mut().find(|t| t.task_id == *task_id) {
            task.total_seconds += duration;
        } else {
            today_tasks.push(TaskTimeEntry {
                task_id: *task_id,
                task_title: task_title.clone(),
                project_name: project_name.clone(),
                project_color: project_color.clone(),
                total_seconds: *duration,
            });
        }
    }

    // Get this week's entries aggregated by day
    let mut stmt = conn.prepare(
        r#"
        SELECT 
            date(created_at, 'unixepoch', 'localtime') as entry_date,
            SUM(duration_seconds) as total_seconds
        FROM time_entries
        WHERE created_at >= ?
        GROUP BY entry_date
        ORDER BY entry_date ASC
        "#,
    )?;

    let mut week_daily: Vec<DailyAggregate> = stmt
        .query_map([week_start], |row| {
            Ok(DailyAggregate {
                date: row.get(0)?,
                total_seconds: row.get(1)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    // Add active timer to today's week_daily entry
    if let Some((_, _, _, _, _, duration)) = &active_timer_info {
        let today_date = chrono::Local::now().format("%Y-%m-%d").to_string();
        if let Some(daily) = week_daily.iter_mut().find(|d| d.date == today_date) {
            daily.total_seconds += duration;
        } else {
            week_daily.push(DailyAggregate {
                date: today_date,
                total_seconds: *duration,
            });
        }
    }

    // Get overall time by project
    let mut stmt = conn.prepare(
        r#"
        SELECT 
            p.name,
            p.color,
            p.total_time_seconds
        FROM projects p
        WHERE p.total_time_seconds > 0
        ORDER BY p.total_time_seconds DESC
        "#,
    )?;

    let mut projects: Vec<ProjectTime> = stmt
        .query_map([], |row| {
            Ok(ProjectTime {
                name: row.get(0)?,
                color: row.get(1)?,
                total_seconds: row.get(2)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    // Add active timer to project totals
    if let Some((_, project_id, _, _, _, duration)) = &active_timer_info {
        let project_result: std::result::Result<(String, String), rusqlite::Error> = conn
            .query_row(
                "SELECT name, color FROM projects WHERE id = ?",
                [project_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            );

        if let Ok((project_name, project_color)) = project_result {
            if let Some(project) = projects.iter_mut().find(|p| p.name == project_name) {
                project.total_seconds += duration;
            } else {
                projects.push(ProjectTime {
                    name: project_name,
                    color: project_color,
                    total_seconds: *duration,
                });
            }
        }
    }

    Ok(TimeStats {
        today_tasks,
        week_daily,
        projects,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::initialize_schema;
    use rusqlite::Connection;

    #[test]
    fn active_timed_timer_stats_are_capped_at_expiry() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
        initialize_schema(&conn).unwrap();
        let now = chrono::Local::now().timestamp();

        conn.execute(
            "INSERT INTO projects (id, name, color, position, created_at, updated_at)
             VALUES (1, 'Focus', '#6366f1', 0, ?, ?)",
            (now, now),
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tasks (
                 id, project_id, title, completed, position, created_at, updated_at
             ) VALUES (1, 1, 'Deep work', 0, 0, ?, ?)",
            (now, now),
        )
        .unwrap();
        conn.execute(
            "INSERT INTO active_timer (
                 id, task_id, started_at, elapsed_seconds, is_running,
                 last_heartbeat_at, project_id, timer_limit_seconds,
                 timer_remaining_seconds, timer_expires_at
             ) VALUES (1, 1, ?, 0, 1, ?, 1, 60, 60, ?)",
            (now - 120, now, now - 60),
        )
        .unwrap();

        let row = get_active_timer_duration(&conn).unwrap().unwrap();
        assert_eq!(row.5, 60);
    }
}
