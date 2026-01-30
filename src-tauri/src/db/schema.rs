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
            updated_at INTEGER NOT NULL
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

    // Populate project_id for any existing active timer by looking up the task's project
    let _ = conn.execute(
        "UPDATE active_timer
         SET project_id = (SELECT project_id FROM tasks WHERE tasks.id = active_timer.task_id)
         WHERE project_id IS NULL",
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

    Ok(())
}
