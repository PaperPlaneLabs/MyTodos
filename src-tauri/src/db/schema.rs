use crate::error::Result;
use rusqlite::Connection;

pub fn initialize_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            color TEXT DEFAULT '#6366f1',
            position INTEGER NOT NULL DEFAULT 0,
            total_time_seconds INTEGER DEFAULT 0,
            is_system BOOLEAN NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS sections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            position INTEGER NOT NULL DEFAULT 0,
            total_time_seconds INTEGER DEFAULT 0,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER,
            section_id INTEGER,
            title TEXT NOT NULL,
            description TEXT,
            completed BOOLEAN DEFAULT 0,
            position INTEGER NOT NULL DEFAULT 0,
            total_time_seconds INTEGER DEFAULT 0,
            is_system BOOLEAN NOT NULL DEFAULT 0,
            deadline TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
            FOREIGN KEY (section_id) REFERENCES sections(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS time_entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id INTEGER NOT NULL,
            entry_type TEXT CHECK(entry_type IN ('timer', 'manual')) NOT NULL,
            duration_seconds INTEGER NOT NULL,
            started_at INTEGER,
            ended_at INTEGER,
            note TEXT,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS active_timer (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            task_id INTEGER NOT NULL,
            started_at INTEGER NOT NULL,
            elapsed_seconds INTEGER DEFAULT 0,
            is_running BOOLEAN DEFAULT 1,
            last_heartbeat_at INTEGER,
            project_id INTEGER,
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS window_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            x INTEGER,
            y INTEGER,
            width INTEGER DEFAULT 380,
            height INTEGER DEFAULT 800,
            dock_preference TEXT CHECK(dock_preference IN ('left', 'center', 'right')),
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS window_activity_entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            app_identifier TEXT NOT NULL,
            app_name TEXT NOT NULL,
            started_at INTEGER NOT NULL,
            ended_at INTEGER NOT NULL,
            duration_seconds INTEGER NOT NULL,
            created_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS active_window_tracking (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            app_identifier TEXT NOT NULL,
            app_name TEXT NOT NULL,
            app_started_at INTEGER NOT NULL,
            work_started_at INTEGER NOT NULL,
            last_seen_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS calendar_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT,
            date TEXT NOT NULL,
            is_all_day INTEGER DEFAULT 0,
            color TEXT,
            created_at INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_sections_project ON sections(project_id);
        CREATE INDEX IF NOT EXISTS idx_tasks_project ON tasks(project_id);
        CREATE INDEX IF NOT EXISTS idx_tasks_section ON tasks(section_id);
        CREATE INDEX IF NOT EXISTS idx_time_entries_task ON time_entries(task_id);
        "#,
    )?;

    // Migration: Add deadline column to tasks if it doesn't exist
    let _ = conn.execute("ALTER TABLE tasks ADD COLUMN deadline TEXT", []);

    // Migration: Add project_id to active_timer if it doesn't exist
    let _ = conn.execute("ALTER TABLE active_timer ADD COLUMN project_id INTEGER", []);

    // Migration: Add last_heartbeat_at to active_timer if it doesn't exist
    let _ = conn.execute(
        "ALTER TABLE active_timer ADD COLUMN last_heartbeat_at INTEGER",
        [],
    );

    // Populate project_id for any existing active timer by looking up the task's project
    let _ = conn.execute(
        "UPDATE active_timer
         SET project_id = (SELECT project_id FROM tasks WHERE tasks.id = active_timer.task_id)
         WHERE project_id IS NULL",
        [],
    );

    // Populate heartbeat for any existing timer rows created before the migration.
    let _ = conn.execute(
        "UPDATE active_timer
         SET last_heartbeat_at = started_at
         WHERE last_heartbeat_at IS NULL",
        [],
    );

    // Migration: Add google_event_id column to tasks if it doesn't exist
    let _ = conn.execute("ALTER TABLE tasks ADD COLUMN google_event_id TEXT", []);

    // Migration: Add dock preference persistence for relaunch layout restore
    let _ = conn.execute(
        "ALTER TABLE window_state
         ADD COLUMN dock_preference TEXT CHECK(dock_preference IN ('left', 'center', 'right'))",
        [],
    );

    // Migration: Mark system-only records so break tracking can be hidden from work surfaces
    let _ = conn.execute(
        "ALTER TABLE projects ADD COLUMN is_system BOOLEAN NOT NULL DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE tasks ADD COLUMN is_system BOOLEAN NOT NULL DEFAULT 0",
        [],
    );

    let _ = conn.execute(
        "UPDATE projects
         SET is_system = 1
         WHERE name = 'Breaks'
           AND description = 'Automatically tracked break time'",
        [],
    );
    let _ = conn.execute(
        "UPDATE tasks
         SET is_system = 1
         WHERE title = 'Break'
           AND description = 'Auto-generated task for break time'
           AND project_id IN (SELECT id FROM projects WHERE is_system = 1)",
        [],
    );

    // Create indexes after migrations are complete
    let _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tasks_deadline ON tasks(deadline)",
        [],
    );
    let _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_calendar_events_date ON calendar_events(date)",
        [],
    );
    let _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_projects_is_system ON projects(is_system)",
        [],
    );
    let _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tasks_is_system ON tasks(is_system)",
        [],
    );
    let _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_window_activity_started_at ON window_activity_entries(started_at)",
        [],
    );
    let _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_window_activity_app ON window_activity_entries(app_identifier)",
        [],
    );

    Ok(())
}
