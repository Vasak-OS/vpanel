#[path = "./commands/icons.rs"]
mod icons;
mod window_manager;

use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use window_manager::{WindowInfo, WindowManager};

struct AppState {
    window_manager: Arc<Mutex<WindowManager>>,
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let window_manager = WindowManager::new().expect("Failed to initialize window manager");
    let window_manager = Arc::new(Mutex::new(window_manager));
    let app_state = AppState {
        window_manager: window_manager.clone(),
    };
    let (tx, rx) = channel();
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
        .setup(move |app| {
            let app_handle = app.handle().clone();
            
            std::thread::spawn(move || {
                for _ in rx {
                    let _ = tauri::Emitter::emit(&app_handle, "window-update", ());
                }
            });
            
            Ok(())
        })
        //.invoke_handler(tauri::generate_handler![icons::get_icon])
        .invoke_handler(tauri::generate_handler![
            get_windows,
            toggle_window,
            icons::get_icon_base64,
            icons::get_symbol_base64
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
