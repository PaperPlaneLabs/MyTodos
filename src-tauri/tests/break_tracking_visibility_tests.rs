mod common;

use common::*;
use my_todos_lib::services::timer_service;

fn get_start_of_today() -> i64 {
    let now = chrono::Local::now();
    now.date_naive()
        .and_hms_opt(0, 0, 0)
        .map(|dt| dt.and_local_timezone(chrono::Local).unwrap().timestamp())
        .unwrap_or(0)
}

fn create_manual_entry_at(db: &DbConnection, task_id: i64, duration_seconds: i64, timestamp: i64) {
    let conn = db.lock();

    conn.execute(
        "INSERT INTO time_entries (task_id, entry_type, duration_seconds, created_at)
         VALUES (?, 'manual', ?, ?)",
        (task_id, duration_seconds, timestamp),
    )
    .unwrap();

    conn.execute(
        "UPDATE tasks SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
        (duration_seconds, task_id),
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
        (duration_seconds, project_id),
    )
    .unwrap();
}

fn get_visible_project_names_impl(db: &DbConnection) -> Vec<String> {
    let conn = db.lock();
    let mut stmt = conn
        .prepare(
            "SELECT name
             FROM projects
             WHERE is_system = 0
             ORDER BY position ASC",
        )
        .unwrap();

    stmt.query_map([], |row| row.get(0))
        .unwrap()
        .collect::<std::result::Result<Vec<_>, _>>()
        .unwrap()
}

fn get_visible_task_titles_impl(db: &DbConnection, project_id: i64) -> Vec<String> {
    let conn = db.lock();
    let mut stmt = conn
        .prepare(
            "SELECT title
             FROM tasks
             WHERE project_id = ? AND is_system = 0
             ORDER BY position ASC",
        )
        .unwrap();

    stmt.query_map([project_id], |row| row.get(0))
        .unwrap()
        .collect::<std::result::Result<Vec<_>, _>>()
        .unwrap()
}

fn get_daily_total_time_impl(db: &DbConnection, start_timestamp: i64) -> i64 {
    let conn = db.lock();
    let end_timestamp = start_timestamp + 86400;

    conn.query_row(
        "SELECT COALESCE(SUM(
            CASE
                WHEN te.started_at IS NOT NULL AND te.ended_at IS NOT NULL
                THEN MAX(0, MIN(te.ended_at, ?) - MAX(te.started_at, ?))
                ELSE CASE
                    WHEN te.created_at >= ? AND te.created_at < ? THEN te.duration_seconds
                    ELSE 0
                END
            END
        ), 0)
        FROM time_entries te
        JOIN tasks t ON t.id = te.task_id
        WHERE
            t.is_system = 0
            AND (
                (te.started_at IS NOT NULL AND te.ended_at IS NOT NULL AND te.started_at < ? AND te.ended_at > ?)
                OR (te.started_at IS NULL AND te.created_at >= ? AND te.created_at < ?)
            )",
        [
            end_timestamp,
            start_timestamp,
            start_timestamp,
            end_timestamp,
            end_timestamp,
            start_timestamp,
            start_timestamp,
            end_timestamp,
        ],
        |row| row.get(0),
    )
    .unwrap()
}

fn get_visible_time_entries_with_tasks_impl(db: &DbConnection, date: &str) -> Vec<(String, i64)> {
    let conn = db.lock();
    let mut stmt = conn
        .prepare(
            "SELECT
                t.title as task_title,
                te.duration_seconds
             FROM time_entries te
             JOIN tasks t ON te.task_id = t.id
             LEFT JOIN projects p ON t.project_id = p.id
             WHERE t.is_system = 0
               AND date(te.created_at, 'unixepoch') BETWEEN ? AND ?
             ORDER BY COALESCE(te.started_at, te.created_at) ASC",
        )
        .unwrap();

    stmt.query_map([date, date], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<std::result::Result<Vec<_>, _>>()
        .unwrap()
}

fn get_stats_today_task_titles_impl(db: &DbConnection, today_start: i64) -> Vec<String> {
    let conn = db.lock();
    let mut stmt = conn
        .prepare(
            "SELECT t.title
             FROM time_entries te
             JOIN tasks t ON t.id = te.task_id
             LEFT JOIN projects p ON p.id = t.project_id
             WHERE te.created_at >= ?
             GROUP BY te.task_id
             ORDER BY SUM(te.duration_seconds) DESC",
        )
        .unwrap();

    stmt.query_map([today_start], |row| row.get(0))
        .unwrap()
        .collect::<std::result::Result<Vec<_>, _>>()
        .unwrap()
}

fn get_stats_project_names_impl(db: &DbConnection) -> Vec<String> {
    let conn = db.lock();
    let mut stmt = conn
        .prepare(
            "SELECT name
             FROM projects
             WHERE total_time_seconds > 0
             ORDER BY total_time_seconds DESC",
        )
        .unwrap();

    stmt.query_map([], |row| row.get(0))
        .unwrap()
        .collect::<std::result::Result<Vec<_>, _>>()
        .unwrap()
}

#[test]
fn test_log_break_time_uses_system_project_and_task() {
    let db = setup_test_db();

    timer_service::log_break_time(&db, 300).unwrap();

    let conn = db.lock();
    let project_row: (String, i64, bool) = conn
        .query_row(
            "SELECT name, total_time_seconds, is_system
             FROM projects
             WHERE is_system = 1
             LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();

    let task_row: (i64, String, i64, bool) = conn
        .query_row(
            "SELECT id, title, total_time_seconds, is_system
             FROM tasks
             WHERE is_system = 1
             LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();
    drop(conn);

    assert_eq!(project_row.0, "Breaks");
    assert_eq!(project_row.1, 300);
    assert!(project_row.2);

    assert_eq!(task_row.1, "Break");
    assert_eq!(task_row.2, 300);
    assert!(task_row.3);
    assert_eq!(count_time_entries(&db, task_row.0), 1);
}

#[test]
fn test_break_tracking_is_hidden_from_regular_queries_and_header_totals() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Client Work");
    let task_id = create_test_task(&db, Some(project_id), None, "Ship feature");
    let today_start = get_start_of_today();
    let now = chrono::Local::now().timestamp();
    let today_date = chrono::Local::now().format("%Y-%m-%d").to_string();

    create_manual_entry_at(&db, task_id, 1800, now);
    timer_service::log_break_time(&db, 600).unwrap();

    assert_eq!(
        get_visible_project_names_impl(&db),
        vec!["Client Work".to_string()]
    );
    assert_eq!(
        get_visible_task_titles_impl(&db, project_id),
        vec!["Ship feature".to_string()]
    );
    assert_eq!(get_daily_total_time_impl(&db, today_start), 1800);
    assert_eq!(
        get_visible_time_entries_with_tasks_impl(&db, &today_date),
        vec![("Ship feature".to_string(), 1800)]
    );
}

#[test]
fn test_break_tracking_still_appears_in_stats_queries() {
    let db = setup_test_db();
    let today_start = get_start_of_today();

    timer_service::log_break_time(&db, 900).unwrap();

    assert_eq!(
        get_stats_today_task_titles_impl(&db, today_start),
        vec!["Break".to_string()]
    );
    assert_eq!(
        get_stats_project_names_impl(&db),
        vec!["Breaks".to_string()]
    );
}
