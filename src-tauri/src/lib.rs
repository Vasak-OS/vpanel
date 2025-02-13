#[path = "./commands/icons.rs"]
mod icons;
mod window_manager;
//mod strut_manager;

use gtk::prelude::*;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tauri_plugin_positioner::{Position, WindowExt};
use window_manager::{WindowInfo, WindowManager};
//use strut_manager::StrutManager;

// Estado principal de la aplicación
struct AppState {
    window_manager: Arc<Mutex<WindowManager>>,
    //strut_manager: Arc<Mutex<StrutManager>>,
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Inicialización del window manager
    let window_manager = Arc::new(Mutex::new(
        WindowManager::new().expect("Failed to initialize window manager")
    ));

    let app_state = AppState {
        window_manager: window_manager.clone(),//strut_manager: strut_manager.clone(),
    };

    // Configuración del canal para eventos de ventana
    let (tx, rx) = channel();
    let wm = window_manager.clone();

    std::thread::spawn(move || {
        if let Ok(mut wm) = wm.lock() {
            let _ = wm.backend.setup_event_monitoring(tx);
        }
    });

    // Configuración de Tauri
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

            // Configuración de la ventana principal
            let _ = window.as_ref().window().move_window(Position::BottomLeft);
            gtk_window.set_resizable(false);
            gtk_window.set_type_hint(gtk::gdk::WindowTypeHint::Dock);
            gtk_window.set_urgency_hint(true);
            gtk_window.set_skip_taskbar_hint(true);
            gtk_window.set_skip_pager_hint(true);
            gtk_window.set_keep_above(true);
            gtk_window.stick();

            // Manejador de eventos de actualización de ventana
            {
                let app_handle = app_handle.clone();
                std::thread::spawn(move || {
                    for _ in rx {
                        let _ = tauri::Emitter::emit(&app_handle, "window-update", ());
                    }
                });
            }

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
