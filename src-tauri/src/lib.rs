pub mod app;
pub mod commands;
pub mod db;
pub mod error;
pub mod events;
pub mod google;
pub mod services;

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
    app::startup::initialize_database_state(&db_conn)
        .expect("Failed to initialize database schema");

    let google_state = google::create_google_state();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app.get_webview_window("main").map(|w| {
                let _ = w.show();
                let _ = w.unminimize();
                let _ = w.set_focus();
            });
        }))
        .manage(db_conn.clone())
        .manage(google_state)
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let db_clone = db_conn.clone();

            app::startup::apply_saved_window_dock_preference(app, &db_clone);
            app::startup::initialize_runtime(app_handle.clone(), db_clone.clone());
            app::tray::setup_system_tray(app, app_handle.clone(), db_clone.clone())?;
            app::window_lifecycle::register_main_window_close_behavior(app, app_handle, db_clone);
            app::startup::apply_launch_visibility(app);

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
            commands::center_window,
            commands::set_window_dock_preference,
            commands::get_window_dock_preference,
            commands::set_collapsed,
            commands::move_window,
            commands::start_window_drag,
            commands::get_tasks_by_deadline_range,
            commands::update_task_deadline,
            commands::create_calendar_event,
            commands::get_calendar_events_in_range,
            commands::get_window_orientation,
            commands::open_break_window,
            commands::close_break_window,

            commands::log_break_time,
            commands::get_time_entries_with_tasks,
            commands::google_auth_start,
            commands::google_auth_status,
            commands::google_auth_disconnect,
            commands::google_sync_all_tasks,
            commands::open_resume_window,
            commands::close_resume_window,
            commands::focus_main_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
