mod common;

use common::*;
use my_todos_lib::commands::calendar::get_today_task_summary_impl;
use rusqlite::params;

fn set_deadline(db: &DbConnection, task_id: i64, deadline: &str) {
    db.lock()
        .execute(
            "UPDATE tasks SET deadline = ?1 WHERE id = ?2",
            params![deadline, task_id],
        )
        .expect("failed to set deadline");
}

#[test]
fn groups_incomplete_tasks_using_explicit_local_date_boundaries() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Launch");
    let overdue_id = create_test_task(&db, Some(project_id), None, "Past task");
    let today_id = create_test_task(&db, Some(project_id), None, "Today task");
    let tomorrow_id = create_test_task(&db, Some(project_id), None, "Tomorrow task");

    set_deadline(&db, overdue_id, "2026-08-11T23:59:59");
    set_deadline(&db, today_id, "2026-08-12T15:30:00");
    set_deadline(&db, tomorrow_id, "2026-08-13");

    let summary = get_today_task_summary_impl(&db.lock(), "2026-08-12", "2026-08-13")
        .expect("today summary should load");

    assert_eq!(summary.overdue.len(), 1);
    assert_eq!(summary.overdue[0].id, overdue_id);
    assert_eq!(summary.today.len(), 1);
    assert_eq!(summary.today[0].id, today_id);
    assert_eq!(summary.today[0].project_name.as_deref(), Some("Launch"));
    assert_eq!(summary.today[0].project_color.as_deref(), Some("#6366f1"));
    assert_eq!(summary.completed_today, 0);
    assert_eq!(summary.total_today, 1);
}

#[test]
fn excludes_completed_system_and_future_tasks() {
    let db = setup_test_db();
    let completed_id = create_test_task(&db, None, None, "Completed");
    let future_id = create_test_task(&db, None, None, "Future");

    {
        let conn = db.lock();
        conn.execute(
            "UPDATE tasks SET deadline = '2026-08-12', completed = 1 WHERE id = ?1",
            [completed_id],
        )
        .unwrap();
        conn.execute(
            "UPDATE tasks SET deadline = '2026-08-14' WHERE id = ?1",
            [future_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tasks (title, completed, position, total_time_seconds, deadline, created_at, updated_at, is_system)
             VALUES ('System', 0, 0, 0, '2026-08-12', 0, 0, 1)",
            [],
        )
        .unwrap();
    }

    let summary = get_today_task_summary_impl(&db.lock(), "2026-08-12", "2026-08-13").unwrap();

    assert!(summary.overdue.is_empty());
    assert!(summary.today.is_empty());
    assert_eq!(summary.completed_today, 1);
    assert_eq!(summary.total_today, 1);
}

#[test]
fn orders_each_group_by_deadline_then_position_then_id() {
    let db = setup_test_db();
    let later = create_test_task(&db, None, None, "Later");
    let earlier_second = create_test_task(&db, None, None, "Earlier second");
    let earlier_first = create_test_task(&db, None, None, "Earlier first");

    {
        let conn = db.lock();
        conn.execute(
            "UPDATE tasks SET deadline = '2026-08-12T12:00:00', position = 0 WHERE id = ?1",
            [later],
        )
        .unwrap();
        conn.execute(
            "UPDATE tasks SET deadline = '2026-08-12T09:00:00', position = 2 WHERE id = ?1",
            [earlier_second],
        )
        .unwrap();
        conn.execute(
            "UPDATE tasks SET deadline = '2026-08-12T09:00:00', position = 1 WHERE id = ?1",
            [earlier_first],
        )
        .unwrap();
    }

    let summary = get_today_task_summary_impl(&db.lock(), "2026-08-12", "2026-08-13").unwrap();

    let ids: Vec<i64> = summary.today.into_iter().map(|task| task.id).collect();
    assert_eq!(ids, vec![earlier_first, earlier_second, later]);
}

#[test]
fn rejects_reversed_or_equal_boundaries() {
    let db = setup_test_db();

    assert!(get_today_task_summary_impl(&db.lock(), "2026-08-13", "2026-08-12",).is_err());
    assert!(get_today_task_summary_impl(&db.lock(), "2026-08-12", "2026-08-12",).is_err());
}
