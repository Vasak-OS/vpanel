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

        Ok(WaylandManager {
            conn,
            manager: event_queue,
            toplevels: Vec::new(),
        })
    }
}

impl WindowManagerBackend for WaylandManager {
    fn get_window_list(&self) -> Result<Vec<WindowInfo>, Box<dyn std::error::Error>> {
        Ok(self.toplevels.clone())
    }

    fn setup_event_monitoring(&mut self, _tx: Sender<()>) -> Result<(), Box<dyn std::error::Error>> {
        self.manager.dispatch_pending(&mut self.conn)?;
        Ok(())
    }

    fn toggle_window(&self, _win_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
