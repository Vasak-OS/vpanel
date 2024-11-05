use super::{WindowInfo, WindowManagerBackend};
use std::sync::mpsc::Sender;
use wayland_client::Connection;

pub struct WaylandManager {
    conn: Connection,
    manager: wayland_client::EventQueue<Connection>,
    toplevels: Vec<WindowInfo>,
}

impl WaylandManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::connect_to_env()?;
        let event_queue = conn.new_event_queue();
        let read_guard = event_queue;

        Ok(WaylandManager {
            conn,
            manager: read_guard,
            toplevels: Vec::new(),
        })
    }
}

impl WindowManagerBackend for WaylandManager {
    fn get_window_list(&self) -> Result<Vec<WindowInfo>, Box<dyn std::error::Error>> {
        Ok(self.toplevels.clone())
    }

    fn focus_window(&self, win_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Implementación para activar ventanas en Wayland
        Ok(())
    }

    fn minimize_window(&self, win_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Implementación para minimizar ventanas en Wayland
        Ok(())
    }

    fn setup_event_monitoring(&mut self, tx: Sender<()>) -> Result<(), Box<dyn std::error::Error>> {
        let manager = &mut self.manager;

        manager.dispatch_pending(&mut self.conn)?;

        Ok(())
    }
}
