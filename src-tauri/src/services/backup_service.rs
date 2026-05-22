use crate::commands::common::get_timestamp;
use crate::db::{get_database_file_path, DbConnection};
use crate::error::{AppError, Result};
use chrono::Local;
use rusqlite::{backup::Backup, Connection, OpenFlags, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const BACKUP_ENABLED_KEY: &str = "backup_enabled";
const BACKUP_FOLDER_KEY: &str = "backup_folder";
const BACKUP_INTERVAL_KEY: &str = "backup_interval_minutes";
const LAST_BACKUP_AT_KEY: &str = "last_backup_at";
const DEFAULT_INTERVAL_MINUTES: i64 = 15;
const MIN_INTERVAL_MINUTES: i64 = 1;
const BACKUP_CHECK_SECONDS: u64 = 60;
const BACKUP_KEEP_COUNT: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSettings {
    pub enabled: bool,
    pub folder: String,
    pub interval_minutes: i64,
    pub last_backup_at: Option<i64>,
}

fn io_error(context: &str, error: std::io::Error) -> AppError {
    AppError::Other(format!("{context}: {error}"))
}

fn read_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    let value = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = ?",
            [key],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    Ok(value)
}

fn write_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO app_settings (key, value, updated_at)
         VALUES (?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET
            value = excluded.value,
            updated_at = excluded.updated_at",
        (key, value, get_timestamp()),
    )?;
    Ok(())
}

fn parse_i64_setting(value: Option<String>, default_value: i64) -> i64 {
    value
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(default_value)
}

fn read_settings_from_conn(conn: &Connection) -> Result<BackupSettings> {
    let enabled = matches!(
        read_setting(conn, BACKUP_ENABLED_KEY)?.as_deref(),
        Some("true")
    );
    let folder = read_setting(conn, BACKUP_FOLDER_KEY)?.unwrap_or_default();
    let interval_minutes = parse_i64_setting(
        read_setting(conn, BACKUP_INTERVAL_KEY)?,
        DEFAULT_INTERVAL_MINUTES,
    )
    .max(MIN_INTERVAL_MINUTES);
    let last_backup_at =
        read_setting(conn, LAST_BACKUP_AT_KEY)?.and_then(|value| value.parse::<i64>().ok());

    Ok(BackupSettings {
        enabled,
        folder,
        interval_minutes,
        last_backup_at,
    })
}

pub fn get_settings(db: &DbConnection) -> Result<BackupSettings> {
    let conn = db.lock();
    read_settings_from_conn(&conn)
}

pub fn set_settings(
    db: &DbConnection,
    enabled: bool,
    folder: String,
    interval_minutes: i64,
) -> Result<BackupSettings> {
    let folder = folder.trim().to_string();
    let interval_minutes = interval_minutes.max(MIN_INTERVAL_MINUTES);

    if enabled && folder.is_empty() {
        return Err(AppError::InvalidInput(
            "Choose a backup folder before enabling auto backup".to_string(),
        ));
    }

    let conn = db.lock();
    write_setting(
        &conn,
        BACKUP_ENABLED_KEY,
        if enabled { "true" } else { "false" },
    )?;
    write_setting(&conn, BACKUP_FOLDER_KEY, &folder)?;
    write_setting(&conn, BACKUP_INTERVAL_KEY, &interval_minutes.to_string())?;
    read_settings_from_conn(&conn)
}

pub fn backup_now(db: &DbConnection) -> Result<BackupSettings> {
    let settings = get_settings(db)?;
    if settings.folder.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "Choose a backup folder before running a backup".to_string(),
        ));
    }

    run_backup(&get_database_file_path(), Path::new(&settings.folder))?;

    let now = get_timestamp();
    let conn = db.lock();
    write_setting(&conn, LAST_BACKUP_AT_KEY, &now.to_string())?;
    read_settings_from_conn(&conn)
}

pub fn initialize_backup(db: DbConnection) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(BACKUP_CHECK_SECONDS));

        let settings = match get_settings(&db) {
            Ok(settings) => settings,
            Err(error) => {
                eprintln!("Failed to read backup settings: {error}");
                continue;
            }
        };

        if !settings.enabled || settings.folder.trim().is_empty() {
            continue;
        }

        let now = get_timestamp();
        let last_backup_at = settings.last_backup_at.unwrap_or(0);
        if now - last_backup_at < settings.interval_minutes * 60 {
            continue;
        }

        if let Err(error) = run_backup(&get_database_file_path(), Path::new(&settings.folder)) {
            eprintln!("Backup error: {error}");
            continue;
        }

        let conn = db.lock();
        if let Err(error) = write_setting(&conn, LAST_BACKUP_AT_KEY, &now.to_string()) {
            eprintln!("Failed to update backup timestamp: {error}");
        }
    });
}

pub fn run_backup(src_path: &Path, dest_folder: &Path) -> Result<PathBuf> {
    if !src_path.exists() {
        return Err(AppError::NotFound(format!(
            "Database file not found: {}",
            src_path.display()
        )));
    }

    fs::create_dir_all(dest_folder).map_err(|error| {
        io_error(
            &format!("Could not create backup folder {}", dest_folder.display()),
            error,
        )
    })?;

    let filename = format!("todos_backup_{}.db", Local::now().format("%Y%m%d_%H%M%S"));
    let dest_path = dest_folder.join(filename);
    let src = Connection::open_with_flags(src_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut dst = Connection::open(&dest_path)?;
    let backup = Backup::new(&src, &mut dst)?;
    backup.run_to_completion(100, Duration::from_millis(5), None)?;
    prune_old_backups(dest_folder, BACKUP_KEEP_COUNT)?;
    Ok(dest_path)
}

pub fn restore_backup(db: &DbConnection, backup_path: &Path) -> Result<()> {
    if !backup_path.exists() {
        return Err(AppError::NotFound(format!(
            "Backup file not found: {}",
            backup_path.display()
        )));
    }

    let src = Connection::open_with_flags(backup_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    validate_backup_source(&src)?;

    let mut dst = db.lock();
    {
        let backup = Backup::new(&src, &mut dst)?;
        backup.run_to_completion(100, Duration::from_millis(5), None)?;
    }
    dst.execute("PRAGMA foreign_keys = ON", [])?;
    Ok(())
}

fn validate_backup_source(conn: &Connection) -> Result<()> {
    let quick_check = conn.query_row("PRAGMA quick_check", [], |row| row.get::<_, String>(0))?;
    if !quick_check.eq_ignore_ascii_case("ok") {
        return Err(AppError::InvalidInput(format!(
            "Backup database failed integrity check: {quick_check}"
        )));
    }

    for table in ["projects", "tasks", "time_entries", "app_settings"] {
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*)
             FROM sqlite_master
             WHERE type = 'table' AND name = ?",
            [table],
            |row| row.get(0),
        )?;
        if exists == 0 {
            return Err(AppError::InvalidInput(format!(
                "Backup database is missing required table: {table}"
            )));
        }
    }

    Ok(())
}

fn prune_old_backups(folder: &Path, keep: usize) -> Result<()> {
    let entries = fs::read_dir(folder)
        .map_err(|error| io_error(&format!("Could not read {}", folder.display()), error))?;
    let mut backups: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with("todos_backup_") && name.ends_with(".db"))
                .unwrap_or(false)
        })
        .collect();

    backups.sort();
    let delete_count = backups.len().saturating_sub(keep);
    for path in backups.into_iter().take(delete_count) {
        if let Err(error) = fs::remove_file(&path) {
            eprintln!("Failed to remove old backup {}: {}", path.display(), error);
        }
    }

    Ok(())
}

pub fn check_cloud_folders() -> Vec<String> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    for key in ["OneDrive", "OneDriveConsumer", "OneDriveCommercial"] {
        if let Ok(value) = std::env::var(key) {
            candidates.push(PathBuf::from(value));
        }
    }

    if let Some(home) = dirs::home_dir() {
        candidates.push(home.join("OneDrive"));
        candidates.push(home.join("Dropbox"));
    }

    let mut folders: Vec<String> = Vec::new();
    for path in candidates {
        if path.is_dir() {
            let display = path.to_string_lossy().to_string();
            if !folders.iter().any(|folder| folder == &display) {
                folders.push(display);
            }
        }
    }

    folders
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::initialize_schema;
    use parking_lot::Mutex;
    use std::sync::Arc;
    use tempfile::tempdir;

    fn create_test_db(path: &Path, project_name: &str) {
        let conn = Connection::open(path).expect("open test database");
        initialize_schema(&conn).expect("initialize schema");
        conn.execute(
            "INSERT INTO projects (name, description, color, position, created_at, updated_at)
             VALUES (?, '', '#6366f1', 0, 1, 1)",
            [project_name],
        )
        .expect("insert project");
    }

    #[test]
    fn backup_and_restore_round_trip_replaces_live_data() {
        let temp = tempdir().expect("create tempdir");
        let source_path = temp.path().join("source.db");
        let live_path = temp.path().join("live.db");
        let backup_folder = temp.path().join("backups");

        create_test_db(&source_path, "Backed Up Project");
        create_test_db(&live_path, "Live Project");

        let backup_path = run_backup(&source_path, &backup_folder).expect("run backup");
        let live_conn = Connection::open(&live_path).expect("open live database");
        let db = Arc::new(Mutex::new(live_conn));

        restore_backup(&db, &backup_path).expect("restore backup");

        let conn = db.lock();
        let project_name: String = conn
            .query_row("SELECT name FROM projects LIMIT 1", [], |row| row.get(0))
            .expect("read restored project");
        assert_eq!(project_name, "Backed Up Project");
    }

    #[test]
    fn run_backup_prunes_old_backup_files() {
        let temp = tempdir().expect("create tempdir");
        let source_path = temp.path().join("source.db");
        let backup_folder = temp.path().join("backups");
        fs::create_dir_all(&backup_folder).expect("create backup folder");
        create_test_db(&source_path, "Project");

        for i in 0..5 {
            fs::write(
                backup_folder.join(format!("todos_backup_20250101_00000{i}.db")),
                b"old",
            )
            .expect("write old backup");
        }

        run_backup(&source_path, &backup_folder).expect("run backup");

        let backup_count = fs::read_dir(&backup_folder)
            .expect("read backup folder")
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .map(|name| name.starts_with("todos_backup_") && name.ends_with(".db"))
                    .unwrap_or(false)
            })
            .count();
        assert_eq!(backup_count, BACKUP_KEEP_COUNT);
    }
}
