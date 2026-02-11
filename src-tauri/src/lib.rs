pub mod commands;
pub mod db;
pub mod error;
pub mod events;
pub mod google;

use db::{initialize_connection, initialize_schema, DbConnection};
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher::LaunchAgent;

#[tauri::command]
fn initialize_database(db: tauri::State<DbConnection>) -> Result<(), String> {
    let conn = db.lock();
    initialize_schema(&conn).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_conn = initialize_connection().expect("Failed to initialize database connection");

    {
        let conn = db_conn.lock();
        initialize_schema(&conn).expect("Failed to initialize database schema");
    }

    let google_state = google::create_google_state();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(LaunchAgent, Some(vec![])))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(db_conn.clone())
        .manage(google_state)
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let db_clone = db_conn.clone();

            // Initialize system event listeners for auto-pausing timer
            events::initialize_system_listeners(app_handle.clone(), db_clone.clone());

            // Handle window close (shutdown)
            if let Some(window) = app.get_webview_window("main") {
                let shutdown_db = db_clone.clone();
                let shutdown_handle = app_handle.clone();

                window.on_window_event(move |event| {
                    if matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
                        events::auto_pause_if_running(
                            &shutdown_handle,
                            &shutdown_db,
                            events::AutoPauseReason::Shutdown,
                        );
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            initialize_database,
            commands::get_all_projects,
            commands::get_project,
            commands::create_project,
            commands::update_project,
            commands::delete_project,
            commands::reorder_projects,
            commands::get_project_stats,
            commands::get_sections_by_project,
            commands::create_section,
            commands::update_section,
            commands::delete_section,
            commands::reorder_sections,
            commands::get_tasks_by_project,
            commands::get_unassigned_tasks,
            commands::get_tasks_by_section,
            commands::create_task,
            commands::update_task,
            commands::delete_task,
            commands::toggle_task_completion,
            commands::reorder_tasks,
            commands::reset_task_time,
            commands::get_active_timer,
            commands::start_timer,
            commands::pause_timer,
            commands::resume_timer,
            commands::stop_timer,
            commands::reset_timer,
            commands::create_manual_entry,
            commands::get_time_entries_by_task,
            commands::update_time_entry,
            commands::delete_time_entry,
            commands::get_daily_total_time,
            commands::get_time_stats,
            commands::save_window_state,
            commands::get_window_state,
            commands::minimize_window,
            commands::toggle_maximize,
            commands::close_window,
            commands::dock_window,
            commands::set_collapsed,
            commands::move_window,
            commands::start_window_drag,
            commands::get_tasks_by_deadline_range,
            commands::update_task_deadline,
            commands::create_calendar_event,
            commands::get_calendar_events_in_range,
            commands::get_window_orientation,
            commands::get_time_entries_with_tasks,
            commands::google_auth_start,
            commands::google_auth_status,
            commands::google_auth_disconnect,
            commands::google_sync_all_tasks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
