mod window_manager;
//mod strut_manager;
mod tray;

use gtk::prelude::*;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use tauri::{Manager, Emitter};
use tauri_plugin_positioner::{Position, WindowExt};
use window_manager::{WindowInfo, WindowManager};
use tray::{TrayManager, get_tray_items, handle_tray_click};
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

// Configuración de la ventana principal
fn setup_main_window(window: &tauri::WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    let gtk_window = window.gtk_window()?;
    let _ = window.move_window(Position::BottomLeft);

    gtk_window.set_resizable(false);
    gtk_window.set_type_hint(gtk::gdk::WindowTypeHint::Dock);
    gtk_window.set_urgency_hint(true);
    gtk_window.set_skip_taskbar_hint(true);
    gtk_window.set_skip_pager_hint(true);
    gtk_window.set_keep_above(true);
    gtk_window.stick();

    Ok(())
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
    let tray_manager = TrayManager::new()
        .expect("Failed to initialize tray manager");

    let window_manager = Arc::new(Mutex::new(
        WindowManager::new().expect("Failed to initialize window manager")
    ));

    let app_state = AppState {
        window_manager: window_manager.clone(),//strut_manager: strut_manager.clone(),
    };

    tauri::Builder::default()
        .manage(app_state)
        .manage(tray_manager)
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_shell::init())
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

            setup_main_window(&window)?;
            setup_event_monitoring(window_manager.clone(), app.handle().clone())?;

            let tray_manager = app.state::<TrayManager>();
            tray_manager.setup_monitoring(app.handle().clone())?;

            Ok(())
        })
        //.invoke_handler(tauri::generate_handler![icons::get_icon])
        .invoke_handler(tauri::generate_handler![
            get_windows,
            toggle_window,
            get_tray_items,
            handle_tray_click
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
