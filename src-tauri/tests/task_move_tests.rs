mod common;

use common::{
    create_test_project, create_test_section, create_test_task, get_project_time,
    get_section_time, get_task_time, setup_test_db,
};
use my_todos_lib::services::tasks_service;

#[test]
fn test_get_task_success() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Sample Task");

    let conn = db.lock();
    let task = tasks_service::get_task(&conn, task_id).expect("Should get task");

    assert_eq!(task.id, task_id);
    assert_eq!(task.title, "Sample Task");
    assert_eq!(task.project_id, Some(project_id));
}

#[test]
fn test_get_task_not_found() {
    let db = setup_test_db();
    let conn = db.lock();
    let res = tasks_service::get_task(&conn, 9999);
    assert!(res.is_err());
}

#[test]
fn test_move_task_between_projects_transfers_denormalized_time() {
    let db = setup_test_db();
    let p1 = create_test_project(&db, "Project 1");
    let p2 = create_test_project(&db, "Project 2");
    let s1 = create_test_section(&db, p1, "Section 1");

    let task_id = create_test_task(&db, Some(p1), Some(s1), "Moving Task");

    // Set 1800s total time on task, section, and project
    {
        let conn = db.lock();
        conn.execute(
            "UPDATE tasks SET total_time_seconds = 1800 WHERE id = ?",
            [task_id],
        )
        .unwrap();
        conn.execute(
            "UPDATE sections SET total_time_seconds = 1800 WHERE id = ?",
            [s1],
        )
        .unwrap();
        conn.execute(
            "UPDATE projects SET total_time_seconds = 1800 WHERE id = ?",
            [p1],
        )
        .unwrap();
    }

    assert_eq!(get_task_time(&db, task_id), 1800);
    assert_eq!(get_project_time(&db, p1), 1800);
    assert_eq!(get_section_time(&db, s1), 1800);
    assert_eq!(get_project_time(&db, p2), 0);

    // Move task to Project 2
    {
        let conn = db.lock();
        let updated = tasks_service::move_task(&conn, task_id, Some(p2))
            .expect("Should successfully move task");

        assert_eq!(updated.project_id, Some(p2));
        assert_eq!(updated.section_id, None);
    }

    // Check that p1 and s1 decreased by 1800, and p2 increased by 1800
    assert_eq!(get_project_time(&db, p1), 0);
    assert_eq!(get_section_time(&db, s1), 0);
    assert_eq!(get_project_time(&db, p2), 1800);
    assert_eq!(get_task_time(&db, task_id), 1800);
}

#[test]
fn test_move_task_to_unassigned() {
    let db = setup_test_db();
    let p1 = create_test_project(&db, "Project 1");
    let task_id = create_test_task(&db, Some(p1), None, "To Unassigned");

    {
        let conn = db.lock();
        conn.execute(
            "UPDATE tasks SET total_time_seconds = 500 WHERE id = ?",
            [task_id],
        )
        .unwrap();
        conn.execute(
            "UPDATE projects SET total_time_seconds = 500 WHERE id = ?",
            [p1],
        )
        .unwrap();
    }

    {
        let conn = db.lock();
        let updated = tasks_service::move_task(&conn, task_id, None)
            .expect("Should move task to unassigned");
        assert_eq!(updated.project_id, None);
        assert_eq!(updated.section_id, None);
    }

    assert_eq!(get_project_time(&db, p1), 0);
    assert_eq!(get_task_time(&db, task_id), 500);
}

#[test]
fn test_move_task_from_unassigned_to_project() {
    let db = setup_test_db();
    let p1 = create_test_project(&db, "Project 1");
    let task_id = create_test_task(&db, None, None, "From Unassigned");

    {
        let conn = db.lock();
        conn.execute(
            "UPDATE tasks SET total_time_seconds = 600 WHERE id = ?",
            [task_id],
        )
        .unwrap();
    }

    assert_eq!(get_project_time(&db, p1), 0);

    {
        let conn = db.lock();
        let updated = tasks_service::move_task(&conn, task_id, Some(p1))
            .expect("Should move task to project");
        assert_eq!(updated.project_id, Some(p1));
    }

    assert_eq!(get_project_time(&db, p1), 600);
}

#[test]
fn test_move_task_updates_active_timer_project_id() {
    let db = setup_test_db();
    let p1 = create_test_project(&db, "Project 1");
    let p2 = create_test_project(&db, "Project 2");
    let task_id = create_test_task(&db, Some(p1), None, "Active Task");

    // Insert into active_timer
    {
        let conn = db.lock();
        conn.execute(
            "INSERT INTO active_timer (id, task_id, project_id, started_at, elapsed_seconds, is_running)
             VALUES (1, ?, ?, 1000, 0, 1)",
            (task_id, p1),
        )
        .unwrap();
    }

    // Move to p2
    {
        let conn = db.lock();
        tasks_service::move_task(&conn, task_id, Some(p2)).unwrap();

        let active_project_id: Option<i64> = conn
            .query_row(
                "SELECT project_id FROM active_timer WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(active_project_id, Some(p2));
    }
}

#[test]
fn test_move_task_same_project_is_noop() {
    let db = setup_test_db();
    let p1 = create_test_project(&db, "Project 1");
    let task_id = create_test_task(&db, Some(p1), None, "No-op Task");

    let conn = db.lock();
    let updated = tasks_service::move_task(&conn, task_id, Some(p1)).unwrap();
    assert_eq!(updated.project_id, Some(p1));
}
