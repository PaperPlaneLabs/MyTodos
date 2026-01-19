use crate::db::DbConnection;
use crate::error::Result;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct TaskTimeEntry {
    pub task_id: i64,
    pub task_title: String,
    pub project_name: Option<String>,
    pub project_color: Option<String>,
    pub total_seconds: i64,
}

#[derive(Debug, Serialize)]
pub struct DailyAggregate {
    pub date: String,
    pub total_seconds: i64,
}

#[derive(Debug, Serialize)]
pub struct ProjectTime {
    pub name: String,
    pub color: String,
    pub total_seconds: i64,
}

#[derive(Debug, Serialize)]
pub struct TimeStats {
    pub today_tasks: Vec<TaskTimeEntry>,
    pub week_daily: Vec<DailyAggregate>,
    pub projects: Vec<ProjectTime>,
}

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

#[tauri::command]
pub fn get_time_stats(db: State<DbConnection>) -> Result<TimeStats> {
    let conn = db.lock();

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

    let today_tasks: Vec<TaskTimeEntry> = stmt
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

    let week_daily: Vec<DailyAggregate> = stmt
        .query_map([week_start], |row| {
            Ok(DailyAggregate {
                date: row.get(0)?,
                total_seconds: row.get(1)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

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

    let projects: Vec<ProjectTime> = stmt
        .query_map([], |row| {
            Ok(ProjectTime {
                name: row.get(0)?,
                color: row.get(1)?,
                total_seconds: row.get(2)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(TimeStats {
        today_tasks,
        week_daily,
        projects,
    })
}
