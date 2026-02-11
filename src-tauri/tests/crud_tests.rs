mod common;

use common::*;
use my_todos_lib::db::{Project, ProjectStats, Section};
use my_todos_lib::error::{AppError, Result};

fn get_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

// Project CRUD implementations for testing
fn get_all_projects_impl(db: &DbConnection) -> Result<Vec<Project>> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, name, description, color, position, total_time_seconds, created_at, updated_at
         FROM projects ORDER BY position ASC",
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

fn get_project_impl(db: &DbConnection, id: i64) -> Result<Project> {
    let conn = db.lock();
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

fn delete_project_impl(db: &DbConnection, id: i64) -> Result<()> {
    let conn = db.lock();
    let rows = conn.execute("DELETE FROM projects WHERE id = ?", [id])?;

    if rows == 0 {
        return Err(AppError::NotFound(format!(
            "Project with id {} not found",
            id
        )));
    }

    Ok(())
}

fn reorder_projects_impl(db: &DbConnection, project_ids: Vec<i64>) -> Result<()> {
    let conn = db.lock();

    for (position, id) in project_ids.iter().enumerate() {
        conn.execute(
            "UPDATE projects SET position = ? WHERE id = ?",
            (position as i32, *id),
        )?;
    }

    Ok(())
}

fn get_project_stats_impl(db: &DbConnection, project_id: i64) -> Result<ProjectStats> {
    let conn = db.lock();

    let mut stmt = conn.prepare(
        "SELECT COUNT(*) as task_count,
                COALESCE(SUM(CASE WHEN completed = 1 THEN 1 ELSE 0 END), 0) as completed_count,
                COALESCE(SUM(total_time_seconds), 0) as total_time
         FROM tasks WHERE project_id = ?",
    )?;

    let stats = stmt.query_row([project_id], |row| {
        Ok(ProjectStats {
            task_count: row.get(0)?,
            completed_count: row.get(1)?,
            total_time_seconds: row.get(2)?,
        })
    })?;

    Ok(stats)
}

// Section implementations
fn get_sections_by_project_impl(db: &DbConnection, project_id: i64) -> Result<Vec<Section>> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, position, total_time_seconds, created_at
         FROM sections WHERE project_id = ? ORDER BY position ASC",
    )?;

    let sections = stmt
        .query_map([project_id], |row| {
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

fn delete_section_impl(db: &DbConnection, id: i64) -> Result<()> {
    let conn = db.lock();
    let rows = conn.execute("DELETE FROM sections WHERE id = ?", [id])?;

    if rows == 0 {
        return Err(AppError::NotFound(format!(
            "Section with id {} not found",
            id
        )));
    }

    Ok(())
}

fn reorder_sections_impl(db: &DbConnection, section_ids: Vec<i64>) -> Result<()> {
    let conn = db.lock();

    for (position, id) in section_ids.iter().enumerate() {
        conn.execute(
            "UPDATE sections SET position = ? WHERE id = ?",
            (position as i32, *id),
        )?;
    }

    Ok(())
}

// Task implementations
fn toggle_task_completion_impl(db: &DbConnection, id: i64) -> Result<bool> {
    let conn = db.lock();
    let now = get_timestamp();

    let completed: bool = conn
        .query_row("SELECT completed FROM tasks WHERE id = ?", [id], |row| {
            row.get(0)
        })
        .map_err(|_| AppError::NotFound(format!("Task with id {} not found", id)))?;

    let new_completed = !completed;

    conn.execute(
        "UPDATE tasks SET completed = ?, updated_at = ? WHERE id = ?",
        (new_completed, now, id),
    )?;

    Ok(new_completed)
}

fn delete_task_impl(db: &DbConnection, id: i64) -> Result<()> {
    let conn = db.lock();
    let rows = conn.execute("DELETE FROM tasks WHERE id = ?", [id])?;

    if rows == 0 {
        return Err(AppError::NotFound(format!("Task with id {} not found", id)));
    }

    Ok(())
}

// PROJECT TESTS
#[test]
fn test_create_project_with_defaults() {
    let db = setup_test_db();

    let id = create_test_project(&db, "Test Project");
    let project = get_project_impl(&db, id).unwrap();

    assert_eq!(project.name, "Test Project");
    assert_eq!(project.color, "#6366f1");
    assert_eq!(project.position, 0);
    assert_eq!(project.total_time_seconds, 0);
}

#[test]
fn test_create_multiple_projects_position() {
    let db = setup_test_db();

    let id1 = create_test_project(&db, "Project 1");
    let id2 = create_test_project(&db, "Project 2");
    let id3 = create_test_project(&db, "Project 3");

    let proj1 = get_project_impl(&db, id1).unwrap();
    let proj2 = get_project_impl(&db, id2).unwrap();
    let proj3 = get_project_impl(&db, id3).unwrap();

    assert_eq!(proj1.position, 0);
    assert_eq!(proj2.position, 1);
    assert_eq!(proj3.position, 2);
}

#[test]
fn test_get_all_projects_ordered() {
    let db = setup_test_db();

    create_test_project(&db, "Project A");
    create_test_project(&db, "Project B");
    create_test_project(&db, "Project C");

    let projects = get_all_projects_impl(&db).unwrap();

    assert_eq!(projects.len(), 3);
    assert_eq!(projects[0].name, "Project A");
    assert_eq!(projects[1].name, "Project B");
    assert_eq!(projects[2].name, "Project C");
}

#[test]
fn test_get_project_not_found() {
    let db = setup_test_db();

    let result = get_project_impl(&db, 999);

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::NotFound(msg) => assert!(msg.contains("999")),
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_delete_project_success() {
    let db = setup_test_db();

    let id = create_test_project(&db, "Test Project");
    let result = delete_project_impl(&db, id);

    assert!(result.is_ok());

    let get_result = get_project_impl(&db, id);
    assert!(get_result.is_err());
}

#[test]
fn test_delete_project_not_found() {
    let db = setup_test_db();

    let result = delete_project_impl(&db, 999);

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::NotFound(msg) => assert!(msg.contains("999")),
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_delete_project_cascades_to_tasks() {
    let db = setup_test_db();

    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    delete_project_impl(&db, project_id).unwrap();

    // Task should also be deleted due to CASCADE
    let conn = db.lock();
    let task_exists: bool = conn
        .query_row("SELECT 1 FROM tasks WHERE id = ?", [task_id], |_| Ok(true))
        .unwrap_or(false);

    assert!(!task_exists);
}

#[test]
fn test_delete_project_cascades_to_sections() {
    let db = setup_test_db();

    let project_id = create_test_project(&db, "Test Project");
    let section_id = create_test_section(&db, project_id, "Test Section");

    delete_project_impl(&db, project_id).unwrap();

    // Section should also be deleted
    let conn = db.lock();
    let section_exists: bool = conn
        .query_row("SELECT 1 FROM sections WHERE id = ?", [section_id], |_| {
            Ok(true)
        })
        .unwrap_or(false);

    assert!(!section_exists);
}

#[test]
fn test_reorder_projects() {
    let db = setup_test_db();

    let id1 = create_test_project(&db, "Project 1");
    let id2 = create_test_project(&db, "Project 2");
    let id3 = create_test_project(&db, "Project 3");

    // Reverse order
    reorder_projects_impl(&db, vec![id3, id2, id1]).unwrap();

    let projects = get_all_projects_impl(&db).unwrap();
    assert_eq!(projects[0].id, id3);
    assert_eq!(projects[1].id, id2);
    assert_eq!(projects[2].id, id1);
}

#[test]
fn test_get_project_stats_empty() {
    let db = setup_test_db();

    let project_id = create_test_project(&db, "Test Project");
    let stats = get_project_stats_impl(&db, project_id).unwrap();

    assert_eq!(stats.task_count, 0);
    assert_eq!(stats.completed_count, 0);
    assert_eq!(stats.total_time_seconds, 0);
}

#[test]
fn test_get_project_stats_with_tasks() {
    let db = setup_test_db();

    let project_id = create_test_project(&db, "Test Project");
    let task1_id = create_test_task(&db, Some(project_id), None, "Task 1");
    let task2_id = create_test_task(&db, Some(project_id), None, "Task 2");
    let _task3_id = create_test_task(&db, Some(project_id), None, "Task 3");

    // Complete two tasks
    toggle_task_completion_impl(&db, task1_id).unwrap();
    toggle_task_completion_impl(&db, task2_id).unwrap();

    // Add time to tasks
    let conn = db.lock();
    conn.execute(
        "UPDATE tasks SET total_time_seconds = 3600 WHERE id = ?",
        [task1_id],
    )
    .unwrap();
    conn.execute(
        "UPDATE tasks SET total_time_seconds = 1800 WHERE id = ?",
        [task2_id],
    )
    .unwrap();
    drop(conn);

    let stats = get_project_stats_impl(&db, project_id).unwrap();

    assert_eq!(stats.task_count, 3);
    assert_eq!(stats.completed_count, 2);
    assert_eq!(stats.total_time_seconds, 5400); // 3600 + 1800
}

// SECTION TESTS
#[test]
fn test_create_section_with_position() {
    let db = setup_test_db();

    let project_id = create_test_project(&db, "Test Project");
    let section1_id = create_test_section(&db, project_id, "Section 1");
    let section2_id = create_test_section(&db, project_id, "Section 2");

    let sections = get_sections_by_project_impl(&db, project_id).unwrap();

    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0].id, section1_id);
    assert_eq!(sections[0].position, 0);
    assert_eq!(sections[1].id, section2_id);
    assert_eq!(sections[1].position, 1);
}

#[test]
fn test_get_sections_empty_project() {
    let db = setup_test_db();

    let project_id = create_test_project(&db, "Test Project");
    let sections = get_sections_by_project_impl(&db, project_id).unwrap();

    assert_eq!(sections.len(), 0);
}

#[test]
fn test_delete_section_success() {
    let db = setup_test_db();

    let project_id = create_test_project(&db, "Test Project");
    let section_id = create_test_section(&db, project_id, "Test Section");

    let result = delete_section_impl(&db, section_id);

    assert!(result.is_ok());

    let sections = get_sections_by_project_impl(&db, project_id).unwrap();
    assert_eq!(sections.len(), 0);
}

#[test]
fn test_delete_section_not_found() {
    let db = setup_test_db();

    let result = delete_section_impl(&db, 999);

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::NotFound(msg) => assert!(msg.contains("999")),
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_delete_section_cascades_to_tasks() {
    let db = setup_test_db();

    let project_id = create_test_project(&db, "Test Project");
    let section_id = create_test_section(&db, project_id, "Test Section");
    let task_id = create_test_task(&db, Some(project_id), Some(section_id), "Test Task");

    delete_section_impl(&db, section_id).unwrap();

    // Task should also be deleted due to CASCADE
    let conn = db.lock();
    let task_exists: bool = conn
        .query_row("SELECT 1 FROM tasks WHERE id = ?", [task_id], |_| Ok(true))
        .unwrap_or(false);

    assert!(!task_exists);
}

#[test]
fn test_reorder_sections() {
    let db = setup_test_db();

    let project_id = create_test_project(&db, "Test Project");
    let section1_id = create_test_section(&db, project_id, "Section 1");
    let section2_id = create_test_section(&db, project_id, "Section 2");
    let section3_id = create_test_section(&db, project_id, "Section 3");

    // Reverse order
    reorder_sections_impl(&db, vec![section3_id, section2_id, section1_id]).unwrap();

    let sections = get_sections_by_project_impl(&db, project_id).unwrap();
    assert_eq!(sections[0].id, section3_id);
    assert_eq!(sections[1].id, section2_id);
    assert_eq!(sections[2].id, section1_id);
}

// TASK TESTS
#[test]
fn test_toggle_task_completion() {
    let db = setup_test_db();

    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    // Initially not completed
    let conn = db.lock();
    let initial_completed: bool = conn
        .query_row(
            "SELECT completed FROM tasks WHERE id = ?",
            [task_id],
            |row| row.get(0),
        )
        .unwrap();
    drop(conn);

    assert!(!initial_completed);

    // Toggle to completed
    let new_state = toggle_task_completion_impl(&db, task_id).unwrap();
    assert!(new_state);

    // Toggle back to not completed
    let new_state = toggle_task_completion_impl(&db, task_id).unwrap();
    assert!(!new_state);
}

#[test]
fn test_toggle_task_not_found() {
    let db = setup_test_db();

    let result = toggle_task_completion_impl(&db, 999);

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::NotFound(msg) => assert!(msg.contains("999")),
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_delete_task_success() {
    let db = setup_test_db();

    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    let result = delete_task_impl(&db, task_id);

    assert!(result.is_ok());

    let conn = db.lock();
    let task_exists: bool = conn
        .query_row("SELECT 1 FROM tasks WHERE id = ?", [task_id], |_| Ok(true))
        .unwrap_or(false);
    drop(conn);

    assert!(!task_exists);
}

#[test]
fn test_delete_task_not_found() {
    let db = setup_test_db();

    let result = delete_task_impl(&db, 999);

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::NotFound(msg) => assert!(msg.contains("999")),
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_delete_task_with_time_entries() {
    let db = setup_test_db();

    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    // Add time entry
    let conn = db.lock();
    let now = get_timestamp();
    conn.execute(
        "INSERT INTO time_entries (task_id, entry_type, duration_seconds, created_at)
         VALUES (?, 'manual', 3600, ?)",
        (task_id, now),
    )
    .unwrap();
    drop(conn);

    delete_task_impl(&db, task_id).unwrap();

    // Time entries should also be deleted due to CASCADE
    let conn = db.lock();
    let entry_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM time_entries WHERE task_id = ?",
            [task_id],
            |row| row.get(0),
        )
        .unwrap();
    drop(conn);

    assert_eq!(entry_count, 0);
}

// FOREIGN KEY AND CASCADE TESTS
#[test]
fn test_foreign_keys_enabled() {
    let db = setup_test_db();

    let conn = db.lock();
    let fk_enabled: i32 = conn
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .unwrap();

    assert_eq!(fk_enabled, 1);
}

#[test]
fn test_cascade_delete_project_with_complex_hierarchy() {
    let db = setup_test_db();

    let project_id = create_test_project(&db, "Test Project");
    let section_id = create_test_section(&db, project_id, "Test Section");
    let task1_id = create_test_task(&db, Some(project_id), Some(section_id), "Task in Section");
    let task2_id = create_test_task(&db, Some(project_id), None, "Task in Project");

    // Add time entries
    let conn = db.lock();
    let now = get_timestamp();
    conn.execute(
        "INSERT INTO time_entries (task_id, entry_type, duration_seconds, created_at)
         VALUES (?, 'manual', 3600, ?)",
        (task1_id, now),
    )
    .unwrap();
    conn.execute(
        "INSERT INTO time_entries (task_id, entry_type, duration_seconds, created_at)
         VALUES (?, 'manual', 1800, ?)",
        (task2_id, now),
    )
    .unwrap();
    drop(conn);

    // Delete the project
    delete_project_impl(&db, project_id).unwrap();

    // Everything should be deleted
    let conn = db.lock();
    let section_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM sections", [], |row| row.get(0))
        .unwrap();
    let task_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))
        .unwrap();
    let entry_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM time_entries", [], |row| row.get(0))
        .unwrap();

    assert_eq!(section_count, 0);
    assert_eq!(task_count, 0);
    assert_eq!(entry_count, 0);
}

#[test]
fn test_position_calculation_after_delete() {
    let db = setup_test_db();

    let _id1 = create_test_project(&db, "Project 1");
    let id2 = create_test_project(&db, "Project 2");
    let _id3 = create_test_project(&db, "Project 3");

    // Delete middle project
    delete_project_impl(&db, id2).unwrap();

    // Create new project - gets MAX(position) + 1 = 3
    // (positions don't auto-adjust after delete, which is correct)
    let id4 = create_test_project(&db, "Project 4");

    let proj4 = get_project_impl(&db, id4).unwrap();
    assert_eq!(proj4.position, 3); // MAX(0, 1, 2) + 1 = 3
}
