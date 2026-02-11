mod common;

use chrono::{Datelike, Timelike};
use common::*;
use my_todos_lib::error::Result;
use serde::Serialize;

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

fn get_active_timer_duration(conn: &rusqlite::Connection) -> Result<Option<ActiveTimerStatsRow>> {
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
            at.is_running
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

        let duration = if is_running == 1 {
            let now = chrono::Local::now().timestamp();
            elapsed_seconds + (now - started_at)
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

fn get_time_stats_impl(db: &DbConnection, include_active_timer: bool) -> Result<TimeStats> {
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
        get_active_timer_duration(&conn)?
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

// Helper to create time entry with specific timestamp
fn create_time_entry_at(db: &DbConnection, task_id: i64, duration: i64, timestamp: i64) {
    let conn = db.lock();
    conn.execute(
        "INSERT INTO time_entries (task_id, entry_type, duration_seconds, created_at)
         VALUES (?, 'manual', ?, ?)",
        (task_id, duration, timestamp),
    )
    .unwrap();

    // Update task and project times
    conn.execute(
        "UPDATE tasks SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
        (duration, task_id),
    )
    .unwrap();

    let project_id: i64 = conn
        .query_row(
            "SELECT project_id FROM tasks WHERE id = ?",
            [task_id],
            |row| row.get(0),
        )
        .unwrap();

    conn.execute(
        "UPDATE projects SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
        (duration, project_id),
    )
    .unwrap();
}

// DATE BOUNDARY TESTS
#[test]
fn test_get_start_of_today_returns_midnight() {
    let start = get_start_of_today();
    let dt = chrono::DateTime::from_timestamp(start, 0)
        .unwrap()
        .with_timezone(&chrono::Local);

    assert_eq!(dt.hour(), 0);
    assert_eq!(dt.minute(), 0);
    assert_eq!(dt.second(), 0);
}

#[test]
fn test_get_start_of_week_returns_monday() {
    let start = get_start_of_week();
    let dt = chrono::DateTime::from_timestamp(start, 0)
        .unwrap()
        .with_timezone(&chrono::Local);

    assert_eq!(dt.weekday(), chrono::Weekday::Mon);
    assert_eq!(dt.hour(), 0);
    assert_eq!(dt.minute(), 0);
    assert_eq!(dt.second(), 0);
}

#[test]
fn test_start_of_week_is_before_or_equal_today() {
    let week_start = get_start_of_week();
    let today_start = get_start_of_today();

    assert!(week_start <= today_start);
}

// EMPTY RESULT TESTS
#[test]
fn test_get_time_stats_empty_database() {
    let db = setup_test_db();

    let stats = get_time_stats_impl(&db, false).unwrap();

    assert_eq!(stats.today_tasks.len(), 0);
    assert_eq!(stats.week_daily.len(), 0);
    assert_eq!(stats.projects.len(), 0);
}

#[test]
fn test_get_time_stats_no_entries_today() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    // Create entry from yesterday
    let yesterday = get_start_of_today() - 86400;
    create_time_entry_at(&db, task_id, 3600, yesterday);

    let stats = get_time_stats_impl(&db, false).unwrap();

    // Today should be empty
    assert_eq!(stats.today_tasks.len(), 0);

    // But project should have time
    assert_eq!(stats.projects.len(), 1);
    assert_eq!(stats.projects[0].total_seconds, 3600);
}

// TODAY'S TASKS TESTS
#[test]
fn test_get_time_stats_today_single_task() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    let now = chrono::Local::now().timestamp();
    create_time_entry_at(&db, task_id, 3600, now);

    let stats = get_time_stats_impl(&db, false).unwrap();

    assert_eq!(stats.today_tasks.len(), 1);
    assert_eq!(stats.today_tasks[0].task_id, task_id);
    assert_eq!(stats.today_tasks[0].total_seconds, 3600);
    assert_eq!(stats.today_tasks[0].task_title, "Test Task");
}

#[test]
fn test_get_time_stats_today_multiple_tasks() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task1_id = create_test_task(&db, Some(project_id), None, "Task 1");
    let task2_id = create_test_task(&db, Some(project_id), None, "Task 2");

    let now = chrono::Local::now().timestamp();
    create_time_entry_at(&db, task1_id, 3600, now);
    create_time_entry_at(&db, task2_id, 1800, now);

    let stats = get_time_stats_impl(&db, false).unwrap();

    assert_eq!(stats.today_tasks.len(), 2);
    // Should be sorted by total_seconds DESC
    assert_eq!(stats.today_tasks[0].total_seconds, 3600); // Task 1 first
    assert_eq!(stats.today_tasks[1].total_seconds, 1800); // Task 2 second
}

#[test]
fn test_get_time_stats_today_multiple_entries_same_task() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    let now = chrono::Local::now().timestamp();
    create_time_entry_at(&db, task_id, 1800, now);
    create_time_entry_at(&db, task_id, 1200, now);
    create_time_entry_at(&db, task_id, 600, now);

    let stats = get_time_stats_impl(&db, false).unwrap();

    assert_eq!(stats.today_tasks.len(), 1);
    assert_eq!(stats.today_tasks[0].total_seconds, 3600); // Sum of all entries
}

// WEEKLY AGGREGATES TESTS
#[test]
fn test_get_time_stats_week_daily_aggregation() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    let today = get_start_of_today();
    let yesterday = today - 86400;
    let two_days_ago = today - 2 * 86400;

    create_time_entry_at(&db, task_id, 3600, today + 3600); // Today
    create_time_entry_at(&db, task_id, 1800, yesterday + 3600); // Yesterday
    create_time_entry_at(&db, task_id, 900, two_days_ago + 3600); // 2 days ago

    let stats = get_time_stats_impl(&db, false).unwrap();

    // Should have 3 daily entries (if all within the week)
    assert!(!stats.week_daily.is_empty());
    assert!(stats.week_daily.len() <= 3);

    // Verify total time is preserved
    let total: i64 = stats.week_daily.iter().map(|d| d.total_seconds).sum();
    assert!(total > 0);
}

#[test]
fn test_get_time_stats_week_excludes_old_entries() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    let now = chrono::Local::now().timestamp();
    let last_week = now - 8 * 86400; // 8 days ago (definitely last week)

    create_time_entry_at(&db, task_id, 3600, now);
    create_time_entry_at(&db, task_id, 1800, last_week);

    let stats = get_time_stats_impl(&db, false).unwrap();

    // Week daily should only include this week's entries
    let week_total: i64 = stats.week_daily.iter().map(|d| d.total_seconds).sum();
    assert_eq!(week_total, 3600); // Only today's entry
}

// ACTIVE TIMER INTEGRATION TESTS
#[test]
fn test_get_active_timer_duration_no_timer() {
    let db = setup_test_db();
    let conn = db.lock();

    let result = get_active_timer_duration(&conn).unwrap();

    assert!(result.is_none());
}

#[test]
fn test_get_active_timer_duration_running() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    // Create active timer
    let now = chrono::Local::now().timestamp();
    let conn = db.lock();
    conn.execute(
        "INSERT INTO active_timer (id, task_id, started_at, elapsed_seconds, is_running, project_id)
         VALUES (1, ?, ?, 1200, 1, ?)",
        (task_id, now - 600, project_id),
    )
    .unwrap();
    drop(conn);

    let conn = db.lock();
    let result = get_active_timer_duration(&conn).unwrap();

    assert!(result.is_some());
    let (tid, _pid, title, _pname, _pcolor, duration) = result.unwrap();
    assert_eq!(tid, task_id);
    assert_eq!(title, "Test Task");
    // Duration should be elapsed_seconds + (now - started_at)
    // = 1200 + 600 = 1800 (approximately, may vary by test execution time)
    assert!(duration >= 1800);
}

#[test]
fn test_get_active_timer_duration_paused() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    // Create paused timer
    let now = chrono::Local::now().timestamp();
    let conn = db.lock();
    conn.execute(
        "INSERT INTO active_timer (id, task_id, started_at, elapsed_seconds, is_running, project_id)
         VALUES (1, ?, ?, 1800, 0, ?)",
        (task_id, now, project_id),
    )
    .unwrap();
    drop(conn);

    let conn = db.lock();
    let result = get_active_timer_duration(&conn).unwrap();

    let (_tid, _pid, _title, _pname, _pcolor, duration) = result.unwrap();
    // When paused, duration = elapsed_seconds
    assert_eq!(duration, 1800);
}

#[test]
fn test_get_time_stats_with_active_timer_new_task() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    // Create active timer but no time entries
    let now = chrono::Local::now().timestamp();
    let conn = db.lock();
    conn.execute(
        "INSERT INTO active_timer (id, task_id, started_at, elapsed_seconds, is_running, project_id)
         VALUES (1, ?, ?, 0, 1, ?)",
        (task_id, now - 600, project_id),
    )
    .unwrap();
    drop(conn);

    let stats = get_time_stats_impl(&db, true).unwrap();

    // Should have the task in today_tasks with active timer duration
    assert_eq!(stats.today_tasks.len(), 1);
    assert_eq!(stats.today_tasks[0].task_id, task_id);
    assert!(stats.today_tasks[0].total_seconds >= 600);
}

#[test]
fn test_get_time_stats_with_active_timer_existing_entries() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    // Create time entry
    let now = chrono::Local::now().timestamp();
    create_time_entry_at(&db, task_id, 3600, now);

    // Create active timer for same task
    let conn = db.lock();
    conn.execute(
        "INSERT INTO active_timer (id, task_id, started_at, elapsed_seconds, is_running, project_id)
         VALUES (1, ?, ?, 0, 1, ?)",
        (task_id, now - 600, project_id),
    )
    .unwrap();
    drop(conn);

    let stats = get_time_stats_impl(&db, true).unwrap();

    // Should merge active timer with existing entry
    assert_eq!(stats.today_tasks.len(), 1);
    assert!(stats.today_tasks[0].total_seconds >= 4200); // 3600 + 600
}

#[test]
fn test_get_time_stats_without_active_timer_flag() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    let now = chrono::Local::now().timestamp();
    let conn = db.lock();
    conn.execute(
        "INSERT INTO active_timer (id, task_id, started_at, elapsed_seconds, is_running, project_id)
         VALUES (1, ?, ?, 0, 1, ?)",
        (task_id, now - 600, project_id),
    )
    .unwrap();
    drop(conn);

    let stats = get_time_stats_impl(&db, false).unwrap();

    // Should not include active timer
    assert_eq!(stats.today_tasks.len(), 0);
}

// PROJECT TOTALS TESTS
#[test]
fn test_get_time_stats_project_totals() {
    let db = setup_test_db();
    let project1_id = create_test_project(&db, "Project 1");
    let project2_id = create_test_project(&db, "Project 2");
    let task1_id = create_test_task(&db, Some(project1_id), None, "Task 1");
    let task2_id = create_test_task(&db, Some(project2_id), None, "Task 2");

    let now = chrono::Local::now().timestamp();
    create_time_entry_at(&db, task1_id, 7200, now);
    create_time_entry_at(&db, task2_id, 3600, now);

    let stats = get_time_stats_impl(&db, false).unwrap();

    assert_eq!(stats.projects.len(), 2);
    // Should be sorted by total_time_seconds DESC
    assert_eq!(stats.projects[0].name, "Project 1");
    assert_eq!(stats.projects[0].total_seconds, 7200);
    assert_eq!(stats.projects[1].name, "Project 2");
    assert_eq!(stats.projects[1].total_seconds, 3600);
}

#[test]
fn test_get_time_stats_project_totals_with_active_timer() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    // No time entries, only active timer
    let now = chrono::Local::now().timestamp();
    let conn = db.lock();
    conn.execute(
        "INSERT INTO active_timer (id, task_id, started_at, elapsed_seconds, is_running, project_id)
         VALUES (1, ?, ?, 1200, 1, ?)",
        (task_id, now - 600, project_id),
    )
    .unwrap();
    drop(conn);

    let stats = get_time_stats_impl(&db, true).unwrap();

    // Project should appear with active timer duration
    assert_eq!(stats.projects.len(), 1);
    assert_eq!(stats.projects[0].name, "Test Project");
    assert!(stats.projects[0].total_seconds >= 1800); // 1200 + 600
}

#[test]
fn test_get_time_stats_excludes_zero_time_projects() {
    let db = setup_test_db();
    let _project1_id = create_test_project(&db, "Empty Project");
    let project2_id = create_test_project(&db, "Active Project");
    let task_id = create_test_task(&db, Some(project2_id), None, "Task");

    let now = chrono::Local::now().timestamp();
    create_time_entry_at(&db, task_id, 3600, now);

    let stats = get_time_stats_impl(&db, false).unwrap();

    // Should only include projects with time > 0
    assert_eq!(stats.projects.len(), 1);
    assert_eq!(stats.projects[0].name, "Active Project");
}

// COMPLEX SCENARIOS
#[test]
fn test_get_time_stats_complete_scenario() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task1_id = create_test_task(&db, Some(project_id), None, "Task 1");
    let task2_id = create_test_task(&db, Some(project_id), None, "Task 2");

    let now = chrono::Local::now().timestamp();
    let yesterday = get_start_of_today() - 86400;

    // Today's entries
    create_time_entry_at(&db, task1_id, 3600, now);
    create_time_entry_at(&db, task2_id, 1800, now);

    // Yesterday's entry
    create_time_entry_at(&db, task1_id, 7200, yesterday + 3600);

    // Active timer
    let conn = db.lock();
    conn.execute(
        "INSERT INTO active_timer (id, task_id, started_at, elapsed_seconds, is_running, project_id)
         VALUES (1, ?, ?, 0, 1, ?)",
        (task1_id, now - 300, project_id),
    )
    .unwrap();
    drop(conn);

    let stats = get_time_stats_impl(&db, true).unwrap();

    // Today tasks: Task 1 (3600 + ~300) and Task 2 (1800)
    assert_eq!(stats.today_tasks.len(), 2);

    // Week daily should have at least one entry (today)
    // May have 2 if yesterday is in the same week
    assert!(!stats.week_daily.is_empty());

    // Project total: 3600 + 1800 + 7200 + ~300 = ~12900
    assert_eq!(stats.projects.len(), 1);
    assert!(stats.projects[0].total_seconds >= 12900);
}
