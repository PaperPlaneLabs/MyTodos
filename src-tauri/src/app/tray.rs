use crate::db::DbConnection;
use crate::events;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager, Wry,
};

pub fn setup_system_tray(
    app: &App<Wry>,
    app_handle: AppHandle,
    db: DbConnection,
) -> Result<(), String> {
    let show_item = MenuItem::with_id(app, "show", "Show MyTodos", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let tray_menu =
        Menu::with_items(app, &[&show_item, &quit_item]).map_err(|error| error.to_string())?;

    let default_icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| "Missing default window icon".to_string())?;

    TrayIconBuilder::new()
        .icon(default_icon)
        .menu(&tray_menu)
        .show_menu_on_left_click(false)
        .on_menu_event({
            let handle = app_handle.clone();
            let quit_db = db.clone();
            move |app, event| match event.id.as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    events::auto_pause_if_running(
                        &handle,
                        &quit_db,
                        events::AutoPauseReason::Shutdown,
                    );
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)
        .map_err(|error| error.to_string())?;

    Ok(())
}
