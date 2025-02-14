#[derive(Debug, Clone, serde::Serialize)]
pub struct TrayItem {
    pub id: String,
    pub wid: i32,
    pub icon_data: Option<String>,
    pub title: Option<String>,
}

pub trait TrayBackend {
    fn get_tray_items(&self) -> Result<Vec<TrayItem>, Box<dyn std::error::Error>>;
    fn setup_monitoring(&self, app_handle: tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct TrayManager {
    backend: Box<dyn TrayBackend + Send + Sync>,
}

impl TrayManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        #[cfg(feature = "x11")]
        {
            if std::env::var("DISPLAY").is_ok() {
                return Ok(Self {
                    backend: Box::new(x11::X11TrayBackend::new()?),
                });
            }
        }

        #[cfg(feature = "wayland")]
        {
            if std::env::var("WAYLAND_DISPLAY").is_ok() {
                return Ok(Self {
                    backend: Box::new(wayland::WaylandTrayBackend::new()?),
                });
            }
        }

        Err("No supported display server found".into())
    }

    pub fn get_items(&self) -> Result<Vec<TrayItem>, Box<dyn std::error::Error>> {
        self.backend.get_tray_items()
    }

    pub fn setup_monitoring(&self, app_handle: tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        self.backend.setup_monitoring(app_handle)
    }
}

#[tauri::command]
pub async fn get_tray_items(
    state: tauri::State<'_, TrayManager>,
) -> Result<Vec<TrayItem>, String> {
    state.get_items().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn handle_tray_click(
    _id: String,
    _button: i32,
    _x: i32,
    _y: i32,
    _window: tauri::Window,
) -> Result<(), String> {
    Ok(())
}

#[cfg(feature = "x11")]
pub mod x11;

#[cfg(feature = "wayland")]
pub mod wayland; 