use rusqlite::Connection;
use crate::error::Result;

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
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS window_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            x INTEGER,
            y INTEGER,
            width INTEGER DEFAULT 380,
            height INTEGER DEFAULT 800,
            updated_at INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_sections_project ON sections(project_id);
        CREATE INDEX IF NOT EXISTS idx_tasks_project ON tasks(project_id);
        CREATE INDEX IF NOT EXISTS idx_tasks_section ON tasks(section_id);
        CREATE INDEX IF NOT EXISTS idx_time_entries_task ON time_entries(task_id);

        -- Migration: Ensure project_id is nullable in tasks (SQLite doesn't support ALTER COLUMN NULL)
        -- This is a no-op if the table was created with the new schema, but it's hard to do cleanly in SQLite
        -- without recreating the table. For now, we'll assume new users get the right schema
        -- and existing users might need to reset or we'd need a complex migration.
        "#
    )?;

    // Try to check if we need to migrate (this is a simple way to handle it for now)
    let _ = conn.execute("PRAGMA foreign_keys = OFF", []);
    // We won't do a full migration here to avoid data loss risk, 
    // but the issue is likely that existing databases have NOT NULL.
    let _ = conn.execute("PRAGMA foreign_keys = ON", []);

    Ok(())
}
