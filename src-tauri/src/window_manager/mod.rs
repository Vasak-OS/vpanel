#[cfg(feature = "wayland")]
pub mod wayland;
#[cfg(feature = "x11")]
pub mod x11;

use serde::{Deserialize, Serialize};
use std::sync::mpsc::Sender;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WindowInfo {
    pub id: String,
    pub title: String,
    pub is_minimized: bool,
    pub icon: String,
}

pub trait WindowManagerBackend {
    fn get_window_list(&self) -> Result<Vec<WindowInfo>, Box<dyn std::error::Error>>;
    fn focus_window(&self, win_id: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn minimize_window(&self, win_id: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn setup_event_monitoring(&mut self, tx: Sender<()>) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct WindowManager {
    pub backend: Box<dyn WindowManagerBackend + Send>,
}

impl WindowManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        #[cfg(feature = "wayland")]
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            return Ok(Self {
                backend: Box::new(wayland::WaylandManager::new()?),
            });
        }

        #[cfg(feature = "x11")]
        if std::env::var("DISPLAY").is_ok() {
            return Ok(Self {
                backend: Box::new(x11::X11Manager::new()?),
            });
        }

        Err("No supported window system found".into())
    }

    pub fn get_window_list(&self) -> Result<Vec<WindowInfo>, Box<dyn std::error::Error>> {
        self.backend.get_window_list()
    }

    pub fn focus_window(&self, win_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.backend.focus_window(win_id)
    }

    pub fn minimize_window(&self, win_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.backend.minimize_window(win_id)
    }
}
