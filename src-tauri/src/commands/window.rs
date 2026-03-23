use super::common::get_timestamp;
use crate::db::{DbConnection, WindowState};
use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, PhysicalPosition, PhysicalSize, State,
    WebviewWindow, WebviewWindowBuilder,
};

const DOCK_WIDTH_LOGICAL: f64 = 380.0;
const COLLAPSE_HANDLE_WIDTH_LOGICAL: f64 = 44.0;
const COLLAPSE_HANDLE_HEIGHT_LOGICAL: f64 = 140.0;
const DOCK_PREFERENCE_LEFT: &str = "left";
const DOCK_PREFERENCE_CENTER: &str = "center";
const DOCK_PREFERENCE_RIGHT: &str = "right";

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowOrientation {
    pub side: String,
    pub is_portrait: bool,
    pub width: f64,
    pub height: f64,
}

fn logical_to_physical_pixels(value: f64, scale_factor: f64) -> i32 {
    (value * scale_factor).round() as i32
}

fn logical_to_physical_size(value: f64, scale_factor: f64) -> u32 {
    logical_to_physical_pixels(value, scale_factor).max(1) as u32
}

fn validate_dock_preference(dock_preference: &str) -> Result<&str> {
    match dock_preference {
        DOCK_PREFERENCE_LEFT | DOCK_PREFERENCE_CENTER | DOCK_PREFERENCE_RIGHT => {
            Ok(dock_preference)
        }
        _ => Err(AppError::InvalidInput(format!(
            "Unsupported dock preference: {}",
            dock_preference
        ))),
    }
}

fn set_client_area_position(window: &WebviewWindow, client_x: i32, client_y: i32) -> Result<()> {
    let outer_position = window
        .outer_position()
        .map_err(|e| AppError::Other(e.to_string()))?;
    let inner_position = window
        .inner_position()
        .map_err(|e| AppError::Other(e.to_string()))?;

    let left_inset = inner_position.x - outer_position.x;
    let top_inset = inner_position.y - outer_position.y;

    window
        .set_position(PhysicalPosition::new(
            client_x - left_inset,
            client_y - top_inset,
        ))
        .map_err(|e| AppError::Other(e.to_string()))
}

#[tauri::command]
pub fn minimize_window(window: WebviewWindow) -> Result<()> {
    window
        .minimize()
        .map_err(|e| AppError::Other(e.to_string()))
}

#[tauri::command]
pub fn toggle_maximize(window: WebviewWindow) -> Result<()> {
    if window.is_maximized().unwrap_or(false) {
        window
            .unmaximize()
            .map_err(|e| AppError::Other(e.to_string()))
    } else {
        window
            .maximize()
            .map_err(|e| AppError::Other(e.to_string()))
    }
}

#[tauri::command]
pub fn close_window(window: WebviewWindow) -> Result<()> {
    window.close().map_err(|e| AppError::Other(e.to_string()))
}

fn center_webview_window(window: &WebviewWindow) -> Result<()> {
    let monitor = window
        .current_monitor()
        .map_err(|e| AppError::Other(e.to_string()))?
        .ok_or_else(|| AppError::Other("Could not find current monitor".to_string()))?;

    let scale_factor = monitor.scale_factor();
    let work_area = monitor.work_area();

    let logical_work_x = (work_area.position.x as f64) / scale_factor;
    let logical_work_y = (work_area.position.y as f64) / scale_factor;
    let logical_work_width = (work_area.size.width as f64) / scale_factor;
    let logical_work_height = (work_area.size.height as f64) / scale_factor;

    let width: f64 = 1000.0;
    let height: f64 = 800.0;

    // Ensure it fits
    let width = width.min(logical_work_width);
    let height = height.min(logical_work_height);

    let x = logical_work_x + (logical_work_width - width) / 2.0;
    let y = logical_work_y + (logical_work_height - height) / 2.0;

    window
        .set_size(LogicalSize::new(width, height))
        .map_err(|e| AppError::Other(e.to_string()))?;

    window
        .set_position(LogicalPosition::new(x, y))
        .map_err(|e| AppError::Other(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub fn center_window(window: WebviewWindow) -> Result<()> {
    center_webview_window(&window)
}

fn dock_webview_window(window: &WebviewWindow, side: &str) -> Result<()> {
    validate_dock_preference(side)?;
    if side == DOCK_PREFERENCE_CENTER {
        return Err(AppError::InvalidInput(
            "Use center_window for the center preference".to_string(),
        ));
    }

    let monitor = window
        .current_monitor()
        .map_err(|e| AppError::Other(e.to_string()))?
        .ok_or_else(|| AppError::Other("Could not find current monitor".to_string()))?;

    let scale_factor = monitor.scale_factor();
    let work_area = monitor.work_area();

    let physical_width =
        logical_to_physical_size(DOCK_WIDTH_LOGICAL, scale_factor).min(work_area.size.width);
    let physical_height = work_area.size.height;

    window
        .set_size(PhysicalSize::new(physical_width, physical_height))
        .map_err(|e| AppError::Other(e.to_string()))?;

    let inner_size = window
        .inner_size()
        .map_err(|e| AppError::Other(e.to_string()))?;

    let client_x = if side == DOCK_PREFERENCE_LEFT {
        work_area.position.x
    } else {
        work_area.position.x + work_area.size.width as i32 - inner_size.width as i32
    };

    set_client_area_position(window, client_x, work_area.position.y)?;

    Ok(())
}

#[tauri::command]
pub fn dock_window(window: WebviewWindow, side: String) -> Result<()> {
    dock_webview_window(&window, &side)
}

pub fn apply_dock_preference_to_window(
    window: &WebviewWindow,
    dock_preference: &str,
) -> Result<()> {
    match validate_dock_preference(dock_preference)? {
        DOCK_PREFERENCE_LEFT | DOCK_PREFERENCE_RIGHT => {
            dock_webview_window(window, dock_preference)
        }
        DOCK_PREFERENCE_CENTER => center_webview_window(window),
        _ => unreachable!(),
    }
}

pub fn set_saved_window_dock_preference(
    db: &DbConnection,
    dock_preference: &str,
) -> Result<String> {
    let dock_preference = validate_dock_preference(dock_preference)?.to_string();
    let conn = db.lock();
    let now = get_timestamp();

    conn.execute(
        "INSERT INTO window_state (id, dock_preference, updated_at)
         VALUES (1, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
            dock_preference = excluded.dock_preference,
            updated_at = excluded.updated_at",
        (&dock_preference, now),
    )?;

    Ok(dock_preference)
}

pub fn get_saved_window_dock_preference(db: &DbConnection) -> Result<Option<String>> {
    let conn = db.lock();
    let result = conn.query_row(
        "SELECT dock_preference FROM window_state WHERE id = 1",
        [],
        |row| row.get::<_, Option<String>>(0),
    );

    match result {
        Ok(Some(dock_preference)) => {
            validate_dock_preference(&dock_preference)?;
            Ok(Some(dock_preference))
        }
        Ok(None) => Ok(None),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

#[tauri::command]
pub fn set_window_dock_preference(db: State<DbConnection>, dock_preference: String) -> Result<()> {
    set_saved_window_dock_preference(db.inner(), &dock_preference)?;
    Ok(())
}

#[tauri::command]
pub fn get_window_dock_preference(db: State<DbConnection>) -> Result<Option<String>> {
    get_saved_window_dock_preference(db.inner())
}

#[tauri::command]
pub fn set_collapsed(window: WebviewWindow, collapsed: bool, top: f64) -> Result<()> {
    let monitor = window
        .current_monitor()
        .map_err(|e| AppError::Other(e.to_string()))?
        .ok_or_else(|| AppError::Other("Could not find current monitor".to_string()))?;

    let scale_factor = monitor.scale_factor();
    let work_area = monitor.work_area();
    let top_offset = logical_to_physical_pixels(top, scale_factor);

    if collapsed {
        let handle_width = logical_to_physical_size(COLLAPSE_HANDLE_WIDTH_LOGICAL, scale_factor);
        let handle_height = logical_to_physical_size(COLLAPSE_HANDLE_HEIGHT_LOGICAL, scale_factor);

        window
            .set_min_size(Some(PhysicalSize::new(handle_width, handle_height)))
            .map_err(|e| AppError::Other(e.to_string()))?;

        window
            .set_size(PhysicalSize::new(handle_width, handle_height))
            .map_err(|e| AppError::Other(e.to_string()))?;

        let inner_size = window
            .inner_size()
            .map_err(|e| AppError::Other(e.to_string()))?;
        let client_x = work_area.position.x + work_area.size.width as i32 - inner_size.width as i32;
        let client_y = work_area.position.y + top_offset;

        set_client_area_position(&window, client_x, client_y)?;

        window
            .set_always_on_top(true)
            .map_err(|e| AppError::Other(e.to_string()))?;
    } else {
        let full_width =
            logical_to_physical_size(DOCK_WIDTH_LOGICAL, scale_factor).min(work_area.size.width);

        window
            .set_min_size(Some(PhysicalSize::new(
                logical_to_physical_size(320.0, scale_factor),
                logical_to_physical_size(400.0, scale_factor),
            )))
            .map_err(|e| AppError::Other(e.to_string()))?;

        window
            .set_size(PhysicalSize::new(full_width, work_area.size.height))
            .map_err(|e| AppError::Other(e.to_string()))?;

        let inner_size = window
            .inner_size()
            .map_err(|e| AppError::Other(e.to_string()))?;
        let client_x = work_area.position.x + work_area.size.width as i32 - inner_size.width as i32;

        set_client_area_position(&window, client_x, work_area.position.y)?;

        window
            .set_always_on_top(false)
            .map_err(|e| AppError::Other(e.to_string()))?;
    }

    Ok(())
}

#[tauri::command]
pub fn move_window(window: WebviewWindow, x: f64, y: f64) -> Result<()> {
    window
        .set_position(LogicalPosition::new(x, y))
        .map_err(|e| AppError::Other(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn start_window_drag(window: WebviewWindow) -> Result<()> {
    window
        .start_dragging()
        .map_err(|e| AppError::Other(e.to_string()))
}

#[tauri::command]
pub fn save_window_state(
    db: State<DbConnection>,
    x: Option<i32>,
    y: Option<i32>,
    width: i32,
    height: i32,
) -> Result<()> {
    let conn = db.lock();
    let now = get_timestamp();

    conn.execute(
        "INSERT INTO window_state (id, x, y, width, height, updated_at)
         VALUES (1, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
            x = excluded.x,
            y = excluded.y,
            width = excluded.width,
            height = excluded.height,
            updated_at = excluded.updated_at",
        (x, y, width, height, now),
    )?;

    Ok(())
}

#[tauri::command]
pub fn get_window_state(db: State<DbConnection>) -> Result<Option<WindowState>> {
    let conn = db.lock();

    let result = conn.query_row(
        "SELECT x, y, width, height, updated_at FROM window_state WHERE id = 1",
        [],
        |row| {
            Ok(WindowState {
                x: row.get(0)?,
                y: row.get(1)?,
                width: row.get(2)?,
                height: row.get(3)?,
                updated_at: row.get(4)?,
            })
        },
    );

    match result {
        Ok(state) => Ok(Some(state)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

#[tauri::command]
pub async fn open_break_window(
    app: AppHandle,
    message: String,
    theme: Option<String>,
) -> Result<()> {
    // If window already exists, bring it to focus
    if let Some(existing) = app.get_webview_window("break") {
        existing
            .set_focus()
            .map_err(|e| AppError::Other(e.to_string()))?;
        return Ok(());
    }

    // Get the main window's current monitor for positioning
    let main_window = app
        .get_webview_window("main")
        .ok_or_else(|| AppError::Other("Could not find main window".to_string()))?;

    let monitor = main_window
        .current_monitor()
        .map_err(|e| AppError::Other(e.to_string()))?
        .ok_or_else(|| AppError::Other("Could not find current monitor".to_string()))?;

    let scale_factor = monitor.scale_factor();
    let work_area = monitor.work_area();

    let logical_work_x = (work_area.position.x as f64) / scale_factor;
    let logical_work_y = (work_area.position.y as f64) / scale_factor;
    let logical_work_width = (work_area.size.width as f64) / scale_factor;
    let logical_work_height = (work_area.size.height as f64) / scale_factor;

    let width: f64 = 420.0;
    let height: f64 = 340.0;

    let x = logical_work_x + (logical_work_width - width) / 2.0;
    let y = logical_work_y + (logical_work_height - height) / 2.0;

    let message_json = serde_json::to_string(&message)
        .map_err(|e| AppError::Other(format!("Failed to serialize break message: {}", e)))?;
    let theme_str = theme.as_deref().unwrap_or("light");
    let theme_json = serde_json::to_string(theme_str)
        .map_err(|e| AppError::Other(format!("Failed to serialize theme: {}", e)))?;
    let init_script = format!(
        "window.__BREAK_MESSAGE__ = {}; window.__BREAK_THEME__ = {};",
        message_json, theme_json
    );

    // Load the same SPA (index.html) as the main window.
    // The frontend checks the window label to decide what UI to render.
    // This matches the proven Tauri multi-window pattern (bookcicle/tauri-window-testing).

    let break_window = match WebviewWindowBuilder::new(&app, "break", Default::default())
        .title("Break Reminder")
        .inner_size(width, height)
        .position(x, y)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .shadow(true)
        .visible(true)
        .focused(true)
        .initialization_script(&init_script)
        .on_navigation(|_url| true)
        .on_page_load(|_window, _payload| {})
        .build()
    {
        Ok(window) => window,
        Err(e) => {
            return Err(AppError::Other(format!(
                "Failed to build break window: {}",
                e
            )));
        }
    };

    break_window.on_window_event(|_event| {});

    break_window
        .set_focus()
        .map_err(|e| AppError::Other(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub fn close_break_window(app: AppHandle) -> Result<()> {
    if let Some(window) = app.get_webview_window("break") {
        window.close().map_err(|e| AppError::Other(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub fn log_break_diagnostic(message: String) -> Result<()> {
    println!("[break:diag:js] {}", message);
    Ok(())
}

#[tauri::command]
pub fn get_window_orientation(window: WebviewWindow) -> Result<WindowOrientation> {
    let monitor = window
        .current_monitor()
        .map_err(|e| AppError::Other(e.to_string()))?
        .ok_or_else(|| AppError::Other("Could not find current monitor".to_string()))?;

    let scale_factor = monitor.scale_factor();
    let work_area = monitor.work_area();
    let window_position = window
        .outer_position()
        .map_err(|e| AppError::Other(e.to_string()))?;
    let window_size = window
        .inner_size()
        .map_err(|e| AppError::Other(e.to_string()))?;

    let logical_work_x = (work_area.position.x as f64) / scale_factor;
    let logical_work_width = (work_area.size.width as f64) / scale_factor;
    let window_x = (window_position.x as f64) / scale_factor;
    let window_width = (window_size.width as f64) / scale_factor;

    let threshold = 50.0;
    let side = if window_x <= logical_work_x + threshold {
        "left"
    } else if window_x + window_width >= logical_work_x + logical_work_width - threshold {
        "right"
    } else {
        "center"
    };

    let is_portrait = window_size.height > window_size.width;

    Ok(WindowOrientation {
        side: side.to_string(),
        is_portrait,
        width: window_width,
        height: (window_size.height as f64) / scale_factor,
    })
}

#[tauri::command]
pub async fn open_resume_window(
    app: AppHandle,
    task_id: Option<i64>,
    task_title: String,
    away_time_seconds: i64,
    theme: Option<String>,
) -> Result<()> {
    if let Some(existing) = app.get_webview_window("resume") {
        existing
            .set_focus()
            .map_err(|e| AppError::Other(e.to_string()))?;
        return Ok(());
    }

    let main_window = app
        .get_webview_window("main")
        .ok_or_else(|| AppError::Other("Could not find main window".to_string()))?;

    let monitor = main_window
        .current_monitor()
        .map_err(|e| AppError::Other(e.to_string()))?
        .ok_or_else(|| AppError::Other("Could not find current monitor".to_string()))?;

    let scale_factor = monitor.scale_factor();
    let work_area = monitor.work_area();

    let logical_work_x = (work_area.position.x as f64) / scale_factor;
    let logical_work_y = (work_area.position.y as f64) / scale_factor;
    let logical_work_width = (work_area.size.width as f64) / scale_factor;
    let logical_work_height = (work_area.size.height as f64) / scale_factor;

    let width: f64 = 420.0;
    let height: f64 = 400.0;

    let x = logical_work_x + (logical_work_width - width) / 2.0;
    let y = logical_work_y + (logical_work_height - height) / 2.0;

    let task_id_json = serde_json::to_string(&task_id)
        .map_err(|e| AppError::Other(format!("Failed to serialize task_id: {}", e)))?;
    let task_title_json = serde_json::to_string(&task_title)
        .map_err(|e| AppError::Other(format!("Failed to serialize task_title: {}", e)))?;
    let away_time_json = serde_json::to_string(&away_time_seconds)
        .map_err(|e| AppError::Other(format!("Failed to serialize away_time: {}", e)))?;
    let theme_str = theme.as_deref().unwrap_or("light");
    let theme_json = serde_json::to_string(theme_str)
        .map_err(|e| AppError::Other(format!("Failed to serialize theme: {}", e)))?;

    let init_script = format!(
        "window.__RESUME_DATA__ = {{ \
            taskId: {}, \
            taskTitle: {}, \
            awayTimeSeconds: {}, \
            theme: {} \
        }};",
        task_id_json, task_title_json, away_time_json, theme_json
    );

    let resume_window = match WebviewWindowBuilder::new(&app, "resume", Default::default())
        .title("Resume Work")
        .inner_size(width, height)
        .position(x, y)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .shadow(true)
        .visible(true)
        .focused(true)
        .initialization_script(&init_script)
        .on_navigation(|_url| true)
        .on_page_load(|_window, _payload| {})
        .build()
    {
        Ok(window) => window,
        Err(e) => {
            return Err(AppError::Other(format!(
                "Failed to build resume window: {}",
                e
            )));
        }
    };

    resume_window.on_window_event(|_event| {});

    resume_window
        .set_focus()
        .map_err(|e| AppError::Other(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub fn close_resume_window(app: AppHandle) -> Result<()> {
    if let Some(window) = app.get_webview_window("resume") {
        window.close().map_err(|e| AppError::Other(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub fn focus_main_window(app: AppHandle) -> Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_focus()
            .map_err(|e| AppError::Other(e.to_string()))?;
        window
            .unminimize()
            .map_err(|e| AppError::Other(e.to_string()))?;
    }
    Ok(())
}
