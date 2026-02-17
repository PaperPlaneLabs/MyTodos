use super::{auto_pause_if_running, AutoPauseReason};
use crate::db::DbConnection;
use futures_util::stream::StreamExt;
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::Mutex;
use zbus::Connection;

/// Initialize Linux systemd D-Bus listener
pub fn initialize_linux_listener(app_handle: AppHandle, db: DbConnection) {
    tokio::spawn(async move {
        println!("Initializing Linux systemd D-Bus listener");

        let app_handle = Arc::new(Mutex::new(app_handle));
        let db = Arc::new(Mutex::new(db));

        match Connection::system().await {
            Ok(connection) => {
                // Subscribe to PrepareForSleep signal from systemd
                // Create a proxy to the login manager
                match zbus::Proxy::new(
                    &connection,
                    "org.freedesktop.login1",
                    "/org/freedesktop/login1",
                    "org.freedesktop.login1.Manager",
                )
                .await
                {
                    Ok(proxy) => {
                        match proxy.receive_signal("PrepareForSleep").await {
                            Ok(mut stream) => {
                                println!(
                                    "Successfully subscribed to systemd PrepareForSleep signal"
                                );

                                while let Some(msg) = stream.next().await {
                                    // PrepareForSleep signal has a boolean argument:
                                    // true = about to sleep, false = waking up
                                    let body = msg.body();
                                    if let Ok(is_sleeping) = body.deserialize::<bool>() {
                                        if is_sleeping {
                                            println!("Linux system suspend detected");

                                            let app = app_handle.lock().await;
                                            let db = db.lock().await;
                                            auto_pause_if_running(
                                                &app,
                                                &db,
                                                AutoPauseReason::SystemSleep,
                                            );
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to subscribe to PrepareForSleep signal: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to create login1 proxy: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to connect to system D-Bus: {}", e);
            }
        }
    });
}
