#[path = "./commands/icons.rs"]
mod icons;
mod window_manager;
//mod strut_manager;
mod systemtray_manager;

use gtk::prelude::*;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use systemtray_manager::{SystemTrayManager, TrayIcon};
use tauri::Manager;
use tauri_plugin_positioner::{Position, WindowExt};
use window_manager::{WindowInfo, WindowManager};
//use strut_manager::StrutManager;

struct AppState {
    window_manager: Arc<Mutex<WindowManager>>,
    system_tray_manager: Arc<Mutex<SystemTrayManager>>,
    //strut_manager: Arc<Mutex<StrutManager>>,
}

#[tauri::command]
async fn get_windows(state: tauri::State<'_, AppState>) -> Result<Vec<WindowInfo>, String> {
    state
        .window_manager
        .lock()
        .map_err(|e| e.to_string())?
        .get_window_list()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn toggle_window(window_id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state
        .window_manager
        .lock()
        .map_err(|e| e.to_string())?
        .toggle_window(&window_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_tray_icons(state: tauri::State<'_, AppState>) -> Result<Vec<TrayIcon>, String> {
    state
        .system_tray_manager
        .lock()
        .map_err(|e| e.to_string())?
        .get_tray_icons()
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let window_manager = WindowManager::new().expect("Failed to initialize window manager");
    let window_manager = Arc::new(Mutex::new(window_manager));
    let system_tray_manager = Arc::new(Mutex::new(
        SystemTrayManager::new().expect("Failed to initialize system tray manager"),
    ));
    //let strut_manager = Arc::new(Mutex::new(StrutManager::new()));

    let app_state = AppState {
        window_manager: window_manager.clone(),
        system_tray_manager: system_tray_manager.clone(),
        //strut_manager: strut_manager.clone(),
    };
    let (tx, rx) = channel();
    let (tray_tx, tray_rx): (std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>) = channel();
    let wm = window_manager.clone();

    std::thread::spawn(move || {
        if let Ok(mut wm) = wm.lock() {
            let _ = wm.backend.setup_event_monitoring(tx);
        }
    });

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let window = app_handle.get_webview_window("main").unwrap();
            let gtk_window = window.gtk_window().unwrap();

            let _ = window.as_ref().window().move_window(Position::BottomLeft);
            gtk_window.set_resizable(false);
            gtk_window.set_type_hint(gtk::gdk::WindowTypeHint::Dock);
            gtk_window.set_keep_above(true);
            gtk_window.stick();

            // Window update events
            {
                let app_handle = app_handle.clone();
                std::thread::spawn(move || {
                    for _ in rx {
                        let _ = tauri::Emitter::emit(&app_handle, "window-update", ());
                    }
                });
            }

            // System tray update events
            {
                let app_handle = app_handle.clone();
                std::thread::spawn(move || {
                    for _ in tray_rx {
                        let _ = tauri::Emitter::emit(&app_handle, "tray-update", ());
                    }
                });
            }

            Ok(())
        })
        //.invoke_handler(tauri::generate_handler![icons::get_icon])
        .invoke_handler(tauri::generate_handler![
            get_windows,
            toggle_window,
            get_tray_icons,
            icons::get_icon_base64,
            icons::get_symbol_base64
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
