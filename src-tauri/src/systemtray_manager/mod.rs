use serde::{Deserialize, Serialize};
use std::sync::mpsc::Sender;

#[cfg(feature = "wayland")]
pub mod wayland;
#[cfg(feature = "x11")]
pub mod x11;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrayIcon {
    pub id: String,
    pub wid: u32,
    pub icon_data: Option<String>,
}

pub trait SystemTrayBackend {
    fn get_tray_icons(&self) -> Result<Vec<TrayIcon>, Box<dyn std::error::Error>>;
    fn setup_event_monitoring(&mut self, tx: Sender<()>) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct SystemTrayManager {
    backend: Box<dyn SystemTrayBackend + Send>,
}

impl SystemTrayManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        #[cfg(feature = "wayland")]
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            return Ok(Self {
                backend: Box::new(wayland::WaylandTrayManager::new()?),
            });
        }

        #[cfg(feature = "x11")]
        if std::env::var("DISPLAY").is_ok() {
            return Ok(Self {
                backend: Box::new(x11::X11TrayManager::new()?),
            });
        }

        Err("No supported window system found".into())
    }

    pub fn setup_event_monitoring(
        &mut self,
        tx: Sender<()>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.backend.setup_event_monitoring(tx)    
    }

    pub fn get_tray_icons(&self) -> Result<Vec<TrayIcon>, Box<dyn std::error::Error>> {
        self.backend.get_tray_icons()
    }
}
