use crate::commands::common::get_timestamp;
use crate::db::{ActiveWindowTracking, DbConnection};
use crate::error::Result;
use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;
use std::path::Path;
use std::time::Duration;

const WINDOW_TRACKING_ENABLED_KEY: &str = "window_tracking_enabled";
const WINDOW_TRACKING_PAUSED_KEY: &str = "window_tracking_paused";
const TRACKER_POLL_SECONDS: u64 = 5;
const MIN_SEGMENT_SECONDS: i64 = 1;

#[derive(Debug, Clone, Serialize)]
pub struct WindowTrackingSettings {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowTrackingState {
    pub enabled: bool,
    pub paused: bool,
    pub active: Option<ActiveWindowTracking>,
    pub today_total_seconds: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppTimeEntry {
    pub app_identifier: String,
    pub app_name: String,
    pub total_seconds: i64,
    pub color: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowDailyAggregate {
    pub date: String,
    pub total_seconds: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowActivityStats {
    pub today_apps: Vec<AppTimeEntry>,
    pub week_daily: Vec<WindowDailyAggregate>,
    pub apps: Vec<AppTimeEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForegroundApp {
    pub app_identifier: String,
    pub app_name: String,
}

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

fn app_color(app_identifier: &str) -> String {
    const COLORS: [&str; 12] = [
        "#2563eb", "#16a34a", "#dc2626", "#ca8a04", "#0891b2", "#7c3aed", "#db2777", "#ea580c",
        "#4f46e5", "#059669", "#be123c", "#0d9488",
    ];
    let hash = app_identifier.bytes().fold(0usize, |acc, byte| {
        acc.wrapping_mul(31).wrapping_add(byte as usize)
    });
    COLORS[hash % COLORS.len()].to_string()
}

fn read_bool_setting(conn: &Connection, key: &str) -> Result<bool> {
    let value = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = ?",
            [key],
            |row| row.get::<_, String>(0),
        )
        .optional()?;

    Ok(matches!(value.as_deref(), Some("true")))
}

fn read_enabled(conn: &Connection) -> Result<bool> {
    read_bool_setting(conn, WINDOW_TRACKING_ENABLED_KEY)
}

fn read_paused(conn: &Connection) -> Result<bool> {
    read_bool_setting(conn, WINDOW_TRACKING_PAUSED_KEY)
}

fn write_bool_setting(conn: &Connection, key: &str, value: bool, now: i64) -> Result<()> {
    conn.execute(
        "INSERT INTO app_settings (key, value, updated_at)
         VALUES (?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET
            value = excluded.value,
            updated_at = excluded.updated_at",
        (key, if value { "true" } else { "false" }, now),
    )?;
    Ok(())
}

fn get_active_from_conn(conn: &Connection) -> Result<Option<ActiveWindowTracking>> {
    let active = conn
        .query_row(
            "SELECT app_identifier, app_name, app_started_at, work_started_at, last_seen_at
             FROM active_window_tracking
             WHERE id = 1",
            [],
            |row| {
                Ok(ActiveWindowTracking {
                    app_identifier: row.get(0)?,
                    app_name: row.get(1)?,
                    app_started_at: row.get(2)?,
                    work_started_at: row.get(3)?,
                    last_seen_at: row.get(4)?,
                })
            },
        )
        .optional()?;

    Ok(active)
}

fn insert_segment(
    conn: &Connection,
    app_identifier: &str,
    app_name: &str,
    started_at: i64,
    ended_at: i64,
) -> Result<()> {
    let safe_ended_at = ended_at.max(started_at);
    let duration_seconds = safe_ended_at - started_at;
    if duration_seconds < MIN_SEGMENT_SECONDS {
        return Ok(());
    }

    conn.execute(
        "INSERT INTO window_activity_entries (
            app_identifier,
            app_name,
            started_at,
            ended_at,
            duration_seconds,
            created_at
         ) VALUES (?, ?, ?, ?, ?, ?)",
        (
            app_identifier,
            app_name,
            started_at,
            safe_ended_at,
            duration_seconds,
            safe_ended_at,
        ),
    )?;

    Ok(())
}

fn close_active_segment(conn: &Connection, now: i64) -> Result<()> {
    if let Some(active) = get_active_from_conn(conn)? {
        insert_segment(
            conn,
            &active.app_identifier,
            &active.app_name,
            active.app_started_at,
            now,
        )?;
        conn.execute("DELETE FROM active_window_tracking WHERE id = 1", [])?;
    }

    Ok(())
}

fn upsert_active_segment(
    conn: &Connection,
    foreground: &ForegroundApp,
    now: i64,
    work_started_at: i64,
) -> Result<()> {
    conn.execute(
        "INSERT INTO active_window_tracking (
            id,
            app_identifier,
            app_name,
            app_started_at,
            work_started_at,
            last_seen_at
         ) VALUES (1, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
            app_identifier = excluded.app_identifier,
            app_name = excluded.app_name,
            app_started_at = excluded.app_started_at,
            work_started_at = excluded.work_started_at,
            last_seen_at = excluded.last_seen_at",
        (
            &foreground.app_identifier,
            &foreground.app_name,
            now,
            work_started_at,
            now,
        ),
    )?;

    Ok(())
}

fn touch_active_segment(conn: &Connection, now: i64) -> Result<()> {
    conn.execute(
        "UPDATE active_window_tracking
         SET last_seen_at = ?
         WHERE id = 1",
        [now],
    )?;
    Ok(())
}

pub fn get_settings(db: &DbConnection) -> Result<WindowTrackingSettings> {
    let conn = db.lock();
    Ok(WindowTrackingSettings {
        enabled: read_enabled(&conn)?,
    })
}

pub fn set_enabled(db: &DbConnection, enabled: bool) -> Result<WindowTrackingSettings> {
    let conn = db.lock();
    let now = get_timestamp();
    write_bool_setting(&conn, WINDOW_TRACKING_ENABLED_KEY, enabled, now)?;
    write_bool_setting(&conn, WINDOW_TRACKING_PAUSED_KEY, false, now)?;
    if !enabled {
        close_active_segment(&conn, now)?;
    }

    Ok(WindowTrackingSettings { enabled })
}

pub fn set_paused(db: &DbConnection, paused: bool) -> Result<WindowTrackingState> {
    let conn = db.lock();
    let now = get_timestamp();
    write_bool_setting(&conn, WINDOW_TRACKING_PAUSED_KEY, paused, now)?;
    if paused {
        close_active_segment(&conn, now)?;
    }
    drop(conn);
    get_state(db)
}

pub fn is_enabled(db: &DbConnection) -> Result<bool> {
    let conn = db.lock();
    read_enabled(&conn)
}

pub fn pause_tracking(db: &DbConnection) -> Result<()> {
    let conn = db.lock();
    close_active_segment(&conn, get_timestamp())
}

pub fn clear_activity(db: &DbConnection) -> Result<()> {
    let conn = db.lock();
    conn.execute("DELETE FROM active_window_tracking", [])?;
    conn.execute("DELETE FROM window_activity_entries", [])?;
    Ok(())
}

pub fn record_foreground_app(db: &DbConnection, foreground: ForegroundApp) -> Result<()> {
    let conn = db.lock();
    if !read_enabled(&conn)? || read_paused(&conn)? {
        close_active_segment(&conn, get_timestamp())?;
        return Ok(());
    }

    let now = get_timestamp();
    match get_active_from_conn(&conn)? {
        Some(active) if active.app_identifier == foreground.app_identifier => {
            touch_active_segment(&conn, now)?;
        }
        Some(active) => {
            let work_started_at = active.work_started_at;
            insert_segment(
                &conn,
                &active.app_identifier,
                &active.app_name,
                active.app_started_at,
                now,
            )?;
            upsert_active_segment(&conn, &foreground, now, work_started_at)?;
        }
        None => {
            upsert_active_segment(&conn, &foreground, now, now)?;
        }
    }

    Ok(())
}

pub fn get_state(db: &DbConnection) -> Result<WindowTrackingState> {
    let conn = db.lock();
    let enabled = read_enabled(&conn)?;
    let paused = read_paused(&conn)?;
    let active = get_active_from_conn(&conn)?;
    let today_total_seconds = get_total_since(&conn, get_start_of_today())?
        + active
            .as_ref()
            .map(|active| (get_timestamp() - active.app_started_at).max(0))
            .unwrap_or(0);

    Ok(WindowTrackingState {
        enabled,
        paused,
        active,
        today_total_seconds,
    })
}

fn get_total_since(conn: &Connection, since: i64) -> Result<i64> {
    let total = conn.query_row(
        "SELECT COALESCE(SUM(duration_seconds), 0)
         FROM window_activity_entries
         WHERE ended_at >= ?",
        [since],
        |row| row.get::<_, i64>(0),
    )?;

    Ok(total)
}

fn add_active_to_apps(apps: &mut Vec<AppTimeEntry>, active: &ActiveWindowTracking, now: i64) {
    let duration = (now - active.app_started_at).max(0);
    if duration == 0 {
        return;
    }

    if let Some(item) = apps
        .iter_mut()
        .find(|item| item.app_identifier == active.app_identifier)
    {
        item.total_seconds += duration;
        return;
    }

    apps.push(AppTimeEntry {
        app_identifier: active.app_identifier.clone(),
        app_name: active.app_name.clone(),
        total_seconds: duration,
        color: app_color(&active.app_identifier),
    });
}

pub fn get_stats(db: &DbConnection) -> Result<WindowActivityStats> {
    let conn = db.lock();
    let today_start = get_start_of_today();
    let week_start = get_start_of_week();
    let active = get_active_from_conn(&conn)?;
    let now = get_timestamp();

    let mut stmt = conn.prepare(
        "SELECT app_identifier, app_name, SUM(duration_seconds) as total_seconds
         FROM window_activity_entries
         WHERE ended_at >= ?
         GROUP BY app_identifier, app_name
         ORDER BY total_seconds DESC",
    )?;
    let mut today_apps: Vec<AppTimeEntry> = stmt
        .query_map([today_start], |row| {
            let app_identifier: String = row.get(0)?;
            Ok(AppTimeEntry {
                color: app_color(&app_identifier),
                app_identifier,
                app_name: row.get(1)?,
                total_seconds: row.get(2)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut stmt = conn.prepare(
        "SELECT date(ended_at, 'unixepoch', 'localtime') as entry_date,
                SUM(duration_seconds) as total_seconds
         FROM window_activity_entries
         WHERE ended_at >= ?
         GROUP BY entry_date
         ORDER BY entry_date ASC",
    )?;
    let mut week_daily: Vec<WindowDailyAggregate> = stmt
        .query_map([week_start], |row| {
            Ok(WindowDailyAggregate {
                date: row.get(0)?,
                total_seconds: row.get(1)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut apps = today_apps.clone();

    if let Some(active) = &active {
        add_active_to_apps(&mut today_apps, active, now);
        add_active_to_apps(&mut apps, active, now);

        let today_date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let active_duration = (now - active.app_started_at).max(0);
        if active_duration > 0 {
            if let Some(day) = week_daily.iter_mut().find(|day| day.date == today_date) {
                day.total_seconds += active_duration;
            } else {
                week_daily.push(WindowDailyAggregate {
                    date: today_date,
                    total_seconds: active_duration,
                });
            }
        }
    }

    today_apps.sort_by(|a, b| b.total_seconds.cmp(&a.total_seconds));
    apps.sort_by(|a, b| b.total_seconds.cmp(&a.total_seconds));

    Ok(WindowActivityStats {
        today_apps,
        week_daily,
        apps,
    })
}

pub fn initialize_tracker(db: DbConnection) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(TRACKER_POLL_SECONDS));

        match get_foreground_app() {
            Some(foreground) => {
                if let Err(error) = record_foreground_app(&db, foreground) {
                    eprintln!("Failed to record active window: {}", error);
                }
            }
            None => {
                if let Ok(true) = is_enabled(&db) {
                    eprintln!("Window tracking is enabled but no foreground app was detected.");
                }
            }
        }
    });
}

#[cfg(target_os = "windows")]
fn get_foreground_app() -> Option<ForegroundApp> {
    use windows::core::PWSTR;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }

        let mut process_id = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id as *mut u32));
        if process_id == 0 {
            return None;
        }

        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id).ok()?;
        let mut buffer = [0u16; 1024];
        let mut size = buffer.len() as u32;
        let result = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut size,
        );
        let _ = CloseHandle(process);
        result.ok()?;

        let path = String::from_utf16_lossy(&buffer[..size as usize]);
        Some(app_from_path(&path))
    }
}

#[cfg(target_os = "macos")]
fn get_foreground_app() -> Option<ForegroundApp> {
    use std::ffi::{c_char, c_void, CStr};

    type ObjcId = *mut c_void;
    type ObjcSel = *mut c_void;

    #[link(name = "AppKit", kind = "framework")]
    extern "C" {}

    #[link(name = "objc")]
    extern "C" {
        fn objc_getClass(name: *const c_char) -> ObjcId;
        fn sel_registerName(name: *const c_char) -> ObjcSel;
        fn objc_msgSend();
    }

    unsafe fn send_id(receiver: ObjcId, selector: ObjcSel) -> ObjcId {
        let function: extern "C" fn(ObjcId, ObjcSel) -> ObjcId =
            std::mem::transmute(objc_msgSend as *const ());
        function(receiver, selector)
    }

    unsafe fn nsstring_to_string(value: ObjcId) -> Option<String> {
        if value.is_null() {
            return None;
        }

        let utf8_selector = sel_registerName(c"UTF8String".as_ptr());
        let function: extern "C" fn(ObjcId, ObjcSel) -> *const c_char =
            std::mem::transmute(objc_msgSend as *const ());
        let raw = function(value, utf8_selector);
        if raw.is_null() {
            return None;
        }

        CStr::from_ptr(raw).to_str().ok().map(ToOwned::to_owned)
    }

    unsafe {
        let workspace_class = objc_getClass(c"NSWorkspace".as_ptr());
        if workspace_class.is_null() {
            return None;
        }

        let shared_workspace = send_id(
            workspace_class,
            sel_registerName(c"sharedWorkspace".as_ptr()),
        );
        if shared_workspace.is_null() {
            return None;
        }

        let app = send_id(
            shared_workspace,
            sel_registerName(c"frontmostApplication".as_ptr()),
        );
        if app.is_null() {
            return None;
        }

        let bundle_identifier =
            nsstring_to_string(send_id(app, sel_registerName(c"bundleIdentifier".as_ptr())));
        let app_name =
            nsstring_to_string(send_id(app, sel_registerName(c"localizedName".as_ptr())))
                .or_else(|| bundle_identifier.clone())
                .unwrap_or_else(|| "Unknown".to_string());

        let app_identifier = bundle_identifier
            .unwrap_or_else(|| app_name.clone())
            .trim()
            .to_ascii_lowercase();
        if app_identifier.is_empty() {
            return None;
        }

        Some(ForegroundApp {
            app_identifier,
            app_name,
        })
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn get_foreground_app() -> Option<ForegroundApp> {
    None
}

fn app_from_path(path: &str) -> ForegroundApp {
    let file_name = Path::new(path)
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Unknown");
    let app_name = prettify_app_name(file_name);

    ForegroundApp {
        app_identifier: file_name.to_ascii_lowercase(),
        app_name,
    }
}

fn prettify_app_name(value: &str) -> String {
    match value.to_ascii_lowercase().as_str() {
        "code" => "Visual Studio Code".to_string(),
        "chrome" => "Google Chrome".to_string(),
        "msedge" => "Microsoft Edge".to_string(),
        "firefox" => "Firefox".to_string(),
        "excel" => "Excel".to_string(),
        "winword" => "Word".to_string(),
        "powerpnt" => "PowerPoint".to_string(),
        "explorer" => "File Explorer".to_string(),
        "windowsterminal" => "Windows Terminal".to_string(),
        "my-todos" => "MyTodos".to_string(),
        other => other
            .split(['-', '_'])
            .filter(|part| !part.is_empty())
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
    }
}
