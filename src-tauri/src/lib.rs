mod window_manager;

use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use tauri::{Manager, Emitter, generate_context};
use tauri_plugin_positioner::{Position, WindowExt};
use tauri_plugin_config_manager;
use window_manager::{WindowInfo, WindowManager};

// Estado principal de la aplicación
struct AppState {
    window_manager: Arc<Mutex<WindowManager>>,
}

// Comandos de la API
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

// Configuración del monitoreo de eventos
fn setup_event_monitoring(
    window_manager: Arc<Mutex<WindowManager>>,
    app_handle: tauri::AppHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = channel();
    
    if let Ok(mut wm) = window_manager.lock() {
        wm.backend.setup_event_monitoring(tx)?;
    }

    std::thread::spawn(move || {
        for _ in rx {
            let _ = app_handle.emit("window-update", ());
        }
    });

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let window_manager = Arc::new(Mutex::new(
        WindowManager::new().expect("Failed to initialize window manager")
    ));

    let app_state = AppState {
        window_manager: window_manager.clone(),//strut_manager: strut_manager.clone(),
    };

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_config_manager::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_vicons::init())
        .setup(move |app| {
            let window = app
                .get_webview_window("main")
                .expect("main window not found");

            let _ = &window.move_window(Position::BottomLeft);
            setup_event_monitoring(window_manager.clone(), app.handle().clone())?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_windows,
            toggle_window
        ])
        .run(generate_context!())
        .expect("error while running tauri application");
}
