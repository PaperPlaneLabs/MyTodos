use crate::db::DbConnection;
use crate::events::system_events::{auto_pause_if_running, AutoPauseReason};
use crate::events::SHUTTING_DOWN;
use std::sync::atomic::Ordering;
use tauri::AppHandle;
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::HBRUSH;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::RemoteDesktop::{
    WTSRegisterSessionNotification, NOTIFY_FOR_THIS_SESSION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW,
    SetWindowLongPtrW, TranslateMessage, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, GWLP_USERDATA, MSG,
    WM_CREATE, WM_ENDSESSION, WM_POWERBROADCAST, WM_QUERYENDSESSION, WM_WTSSESSION_CHANGE,
    WNDCLASSW, WTS_SESSION_LOCK,
};

// Power event constants if missing from the crate version
const PBT_APMSUSPEND: u32 = 0x0004;

/// Initialize Windows power and session event listener
pub fn initialize_windows_listener(app_handle: AppHandle, db: DbConnection) {
    println!("Windows event listener: Starting background thread...");

    std::thread::spawn(move || {
        unsafe {
            let instance: HINSTANCE = GetModuleHandleW(None).unwrap_or_default().into();
            let window_class_name = HSTRING::from("MyTodosSystemEvents");

            let wnd_class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                hInstance: instance,
                hbrBackground: HBRUSH::default(),
                lpszClassName: PCWSTR(window_class_name.as_ptr()),
                ..Default::default()
            };

            RegisterClassW(&wnd_class);

            // Create a message-only window (hidden)
            let hwnd_result = CreateWindowExW(
                Default::default(),
                PCWSTR(window_class_name.as_ptr()),
                PCWSTR(window_class_name.as_ptr()),
                Default::default(),
                0,
                0,
                0,
                0,
                None, // Parent
                None, // Menu
                Some(instance),
                Some(Box::into_raw(Box::new(ListenerState { app_handle, db })) as *const _),
            );

            let hwnd = match hwnd_result {
                Ok(h) => h,
                Err(e) => {
                    eprintln!(
                        "Windows event listener: Failed to create hidden window: {:?}",
                        e
                    );
                    return;
                }
            };

            // Register for session notifications (lock/unlock)
            if let Err(e) = WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION) {
                eprintln!(
                    "Windows event listener: Failed to register for session notifications: {:?}",
                    e
                );
            }

            println!("Windows event listener: Hidden window created, entering message loop");

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    });
}

struct ListenerState {
    app_handle: AppHandle,
    db: DbConnection,
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_WTSSESSION_CHANGE => {
            let reason_code = wparam.0 as u32;
            const WTS_SESSION_UNLOCK: u32 = 0x8;
            if reason_code == WTS_SESSION_LOCK {
                println!("Windows Event: System Locked (WTS_SESSION_LOCK)");
                let state_ptr =
                    windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(hwnd, GWLP_USERDATA)
                        as *mut ListenerState;
                if !state_ptr.is_null() {
                    let state = unsafe { &*state_ptr };
                    crate::events::system_events::handle_screen_locked(&state.app_handle, &state.db);
                }
            } else if reason_code == WTS_SESSION_UNLOCK {
                println!("Windows Event: System Unlocked (WTS_SESSION_UNLOCK)");
                let state_ptr =
                    windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(hwnd, GWLP_USERDATA)
                        as *mut ListenerState;
                if !state_ptr.is_null() {
                    let state = unsafe { &*state_ptr };
                    crate::events::system_events::handle_screen_unlocked(&state.app_handle, &state.db);
                }
            } else {
                // Other session changes are ignored but logged for debug
                println!("Windows Event: Session change code: {}", reason_code);
            }
            LRESULT(0)
        }
        WM_POWERBROADCAST => {
            if wparam.0 as u32 == PBT_APMSUSPEND {
                println!("Windows Event: System Suspend (PBT_APMSUSPEND)");
                let state_ptr =
                    windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(hwnd, GWLP_USERDATA)
                        as *mut ListenerState;
                if !state_ptr.is_null() {
                    let state = unsafe { &*state_ptr };
                    auto_pause_if_running(&state.app_handle, &state.db, AutoPauseReason::SystemSleep);
                }
            }
            LRESULT(1)
        }
        WM_QUERYENDSESSION | WM_ENDSESSION => {
            println!("Windows Event: System Shutdown Detected ({})", msg);
            SHUTTING_DOWN.store(true, Ordering::SeqCst);

            let state_ptr =
                windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(hwnd, GWLP_USERDATA)
                    as *mut ListenerState;
            if !state_ptr.is_null() {
                let state = unsafe { &*state_ptr };
                auto_pause_if_running(&state.app_handle, &state.db, AutoPauseReason::Shutdown);
            }
            LRESULT(1)
        }
        WM_CREATE => {
            let create_struct = lparam.0 as *const CREATESTRUCTW;
            if !create_struct.is_null() {
                let state_ptr = (*create_struct).lpCreateParams;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
