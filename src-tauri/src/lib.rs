mod window_manager;
mod tray;

use tray::{sni_watcher::SniWatcher, TrayManager, TrayItem, TrayMenu, create_tray_manager};
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

#[tauri::command]
async fn init_sni_watcher(
    app_handle: tauri::AppHandle,
    tray_manager: tauri::State<'_, TrayManager>,
) -> Result<(), String> {
    let manager = tray_manager.inner().clone();
    let watcher = SniWatcher::new(manager, app_handle)
        .await
        .map_err(|e| format!("Error inicializando SNI watcher: {}", e))?;
    
    watcher
        .start_watching()
        .await
        .map_err(|e| format!("Error iniciando watcher: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn get_tray_items(
    tray_manager: tauri::State<'_, TrayManager>,
) -> Result<Vec<TrayItem>, String> {
    let manager = tray_manager.read().await;
    Ok(manager.values().cloned().collect())
}

#[tauri::command]
async fn tray_item_activate(
    _service_name: String,
    _x: i32,
    _y: i32,
) -> Result<(), String> {
    // Implementation for activating tray item
    Ok(())
}

#[tauri::command]
async fn tray_item_secondary_activate(
    _service_name: String,
    _x: i32,
    _y: i32,
) -> Result<(), String> {
    // Implementation for secondary activation
    Ok(())
}

#[tauri::command]
async fn get_tray_menu(
    _service_name: String,
) -> Result<Vec<TrayMenu>, String> {
    // Implementation for getting menu items
    Ok(vec![])
}

#[tauri::command]
async fn tray_menu_item_click(
    _service_name: String,
    _menu_id: i32,
) -> Result<(), String> {
    // Implementation for menu item click
    Ok(())
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

    let tray_manager = create_tray_manager();

    tauri::Builder::default()
        .manage(app_state)
        .manage(tray_manager)
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
            toggle_window,
            init_sni_watcher,
            get_tray_items,
            tray_item_activate,
            tray_item_secondary_activate,
            get_tray_menu,
            tray_menu_item_click
        ])
        .run(generate_context!())
        .expect("error while running tauri application");
}
