use super::{auto_pause_if_running, AutoPauseReason};
use crate::db::DbConnection;
use futures_util::stream::StreamExt;
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::Mutex;
use zbus::{proxy, Connection};

#[proxy(
    interface = "org.freedesktop.login1.Session",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1/session/auto"
)]
trait Session {
    #[zbus(property)]
    fn locked_hint(&self) -> zbus::Result<bool>;
}

/// Initialize Linux systemd D-Bus listener
pub fn initialize_linux_listener(app_handle: AppHandle, db: DbConnection) {
    tokio::spawn(async move {
        println!("Initializing Linux systemd D-Bus listener");

        let app_handle = Arc::new(Mutex::new(app_handle));
        let db = Arc::new(Mutex::new(db));

        match Connection::system().await {
            Ok(connection) => {
                // 1. Listen for PrepareForSleep (Suspend/Sleep)
                let app_handle_sleep = app_handle.clone();
                let db_sleep = db.clone();
                let connection_sleep = connection.clone();

                tokio::spawn(async move {
                    if let Ok(proxy) = zbus::Proxy::new(
                        &connection_sleep,
                        "org.freedesktop.login1",
                        "/org/freedesktop/login1",
                        "org.freedesktop.login1.Manager",
                    )
                    .await
                    {
                        if let Ok(mut stream) = proxy.receive_signal("PrepareForSleep").await {
                            while let Some(msg) = stream.next().await {
                                if let Ok(is_sleeping) = msg.body().deserialize::<bool>() {
                                    if is_sleeping {
                                        println!("Linux system suspend detected");
                                        let app = app_handle_sleep.lock().await;
                                        let db = db_sleep.lock().await;
                                        auto_pause_if_running(
                                            &app,
                                            &db,
                                            AutoPauseReason::SystemSleep,
                                        );
                                    }
                                }
                            }
                        }
                    }
                });

                // 2. Listen for LockedHint (Screen Lock)
                // We use "/org/freedesktop/login1/session/self" to refer to the current session
                if let Ok(session_proxy) = SessionProxy::builder(&connection)
                    .path("/org/freedesktop/login1/session/self")
                    .unwrap()
                    .build()
                    .await
                {
                    let mut stream = session_proxy.receive_locked_hint_changed().await;
                    println!("Successfully subscribed to Linux LockedHint changes");
                    while let Some(change) = stream.next().await {
                        if let Ok(is_locked) = change.get::<bool>().await {
                            if is_locked {
                                println!("Linux screen lock detected");
                                let app = app_handle.lock().await;
                                let db = db.lock().await;
                                auto_pause_if_running(
                                    &app,
                                    &db,
                                    AutoPauseReason::ScreenLock,
                                );
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to connect to system D-Bus: {}", e);
            }
        }
    });
}
