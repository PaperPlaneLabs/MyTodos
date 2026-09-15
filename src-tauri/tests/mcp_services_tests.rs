mod common;

use common::*;
use my_todos_lib::services::{projects_service, tasks_service, time_service, timer_service};

#[test]
fn test_list_projects_and_counts() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Work Project");

    // Add an active task and a completed task
    {
        let conn = db.lock();
        let t1 = tasks_service::create_task(
            &conn,
            Some(project_id),
            None,
            "Active task".to_string(),
            None,
            None,
        )
        .unwrap();
        let t2 = tasks_service::create_task(
            &conn,
            Some(project_id),
            None,
            "Completed task".to_string(),
            None,
            None,
        )
        .unwrap();
        tasks_service::set_task_completed(&conn, t2.id, true).unwrap();
        assert_eq!(t1.completed, false);
    }

    let conn = db.lock();
    let projects = projects_service::list_projects(&conn, false).unwrap();

    let work = projects.iter().find(|p| p.name == "Work Project").unwrap();
    assert_eq!(work.active_task_count, 1);
    assert_eq!(work.completed_task_count, 1);
    assert_eq!(work.formatted_total_time, "0m");
}

#[test]
fn test_find_project_and_section_by_name() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Client Alpha");
    let section_id = create_test_section(&db, project_id, "Sprint 1");

    let conn = db.lock();

    // Case-insensitive project lookup
    let found_proj = projects_service::find_project_by_name(&conn, "client alpha")
        .unwrap()
        .expect("Project should be found");
    assert_eq!(found_proj.id, project_id);

    // Section lookup
    let found_sec = projects_service::find_section_by_name(&conn, project_id, "sprint 1")
        .unwrap()
        .expect("Section should be found");
    assert_eq!(found_sec.id, section_id);

    // Non-existent lookup
    assert!(projects_service::find_project_by_name(&conn, "Nonexistent").unwrap().is_none());
}

#[test]
fn test_create_task_with_deadline() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Project Beta");

    let conn = db.lock();
    let task = tasks_service::create_task(
        &conn,
        Some(project_id),
        None,
        "Finish proposal".to_string(),
        Some("Detailed proposal draft".to_string()),
        Some("2026-10-15T14:30".to_string()),
    )
    .unwrap();

    assert_eq!(task.title, "Finish proposal");
    assert_eq!(task.deadline.as_deref(), Some("2026-10-15T14:30"));
    assert_eq!(task.project_id, Some(project_id));
}

#[test]
fn test_log_time_and_get_stats() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Tracking Project");

    let task_id = {
        let conn = db.lock();
        let task = tasks_service::create_task(
            &conn,
            Some(project_id),
            None,
            "Timed Task".to_string(),
            None,
            None,
        )
        .unwrap();
        task.id
    };

    // Log manual time: 3600 seconds (1 hour)
    {
        let conn = db.lock();
        let entry = time_service::create_manual_entry(
            &conn,
            task_id,
            3600,
            Some("Research & development".to_string()),
        )
        .unwrap();

        assert_eq!(entry.duration_seconds, 3600);
        assert_eq!(entry.note.as_deref(), Some("Research & development"));

        // Task total time should be updated
        let task = tasks_service::get_task(&conn, task_id).unwrap();
        assert_eq!(task.total_time_seconds, 3600);

        // Project total time should be updated
        let project = projects_service::get_project(&conn, project_id).unwrap();
        assert_eq!(project.total_time_seconds, 3600);

        // Get time stats
        let stats = time_service::get_time_stats(&conn, false).unwrap();
        assert_eq!(stats.today_tasks.len(), 1);
        assert_eq!(stats.today_tasks[0].task_id, task_id);
        assert_eq!(stats.today_tasks[0].total_seconds, 3600);
        assert_eq!(stats.today_tasks[0].project_name.as_deref(), Some("Tracking Project"));
    }
}

#[test]
fn test_timer_lifecycle() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Timer Project");

    let task_id = {
        let conn = db.lock();
        let task = tasks_service::create_task(
            &conn,
            Some(project_id),
            None,
            "Live Timer Task".to_string(),
            None,
            None,
        )
        .unwrap();
        task.id
    };

    // Start timer
    let active = timer_service::start_timer(&db, task_id).unwrap();
    assert_eq!(active.task_id, task_id);
    assert!(active.is_running);

    // Get active timer
    let current = timer_service::get_active_timer(&db).unwrap().expect("Timer should be active");
    assert_eq!(current.task_id, task_id);
    assert!(current.is_running);

    // Stop timer
    let entry = timer_service::stop_timer(&db).unwrap();
    assert_eq!(entry.task_id, task_id);

    // Active timer is cleared
    let after = timer_service::get_active_timer(&db).unwrap();
    assert!(after.is_none());
}
