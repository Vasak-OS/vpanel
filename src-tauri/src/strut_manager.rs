use std::collections::HashMap;
use tauri::Window;

#[cfg(feature = "x11")]
use x11rb::{
    connection::Connection,
    protocol::xproto::*,
};

pub struct StrutManager;

impl StrutManager {
    pub fn new() -> Self {
        StrutManager
    }

    pub fn setup_strut(&self, window: &Window) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(feature = "x11")]
        if std::env::var("DISPLAY").is_ok() {
            self.setup_x11_strut(window)?;
        }

        #[cfg(feature = "wayland")]
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            self.setup_wayland_strut(window)?;
        }

        Ok(())
    }

    #[cfg(feature = "x11")]
    fn setup_x11_strut(&self, window: &Window) -> Result<(), Box<dyn std::error::Error>> {
        let (conn, _) = x11rb::connect(None)?;
        let window_id = window.ns_window().map(|id| id as u32).map_err(|e| e.to_string())?;
        
        let atoms = ["_NET_WM_STRUT", "_NET_WM_STRUT_PARTIAL"];
        let mut atom_cookies = Vec::new();
        
        for name in atoms.iter() {
            atom_cookies.push(conn.intern_atom(false, name.as_bytes())?);
        }
        
        let mut atom_values = HashMap::new();
        for (cookie, name) in atom_cookies.into_iter().zip(atoms.iter()) {
            if let Ok(reply) = cookie.reply() {
                atom_values.insert(*name, reply.atom);
            }
        }

        // Get window position and monitor size
        let monitor = window.current_monitor()?.unwrap();
        let position = window.outer_position()?;
        let size = window.outer_size()?;
        
        // Calculate strut values based on window position
        let strut_values = if position.y < (monitor.size().height as i32) / 2 {
            // Window is at the top
            [0, 0, size.height, 0]
        } else {
            // Window is at the bottom
            [0, 0, 0, size.height]
        };
        
        if let Some(strut_atom) = atom_values.get("_NET_WM_STRUT") {
            conn.change_property(
                PropMode::REPLACE,
                window_id,
                *strut_atom,
                AtomEnum::CARDINAL,
                32,
                4,
                &strut_values,
            )?;
        }
        
        if let Some(strut_partial_atom) = atom_values.get("_NET_WM_STRUT_PARTIAL") {
            let mut strut_partial = vec![0; 12];
            if position.y < (monitor.size().height as i32) / 2 {
                // Top strut
                strut_partial[2] = size.height; // top
                strut_partial[8] = position.x as u32; // top_start_x
                strut_partial[9] = (position.x + size.width as i32) as u32; // top_end_x
            } else {
                // Bottom strut
                strut_partial[3] = size.height; // bottom
                strut_partial[10] = position.x as u32; // bottom_start_x
                strut_partial[11] = (position.x + size.width as i32) as u32; // bottom_end_x
            }
            
            conn.change_property(
                PropMode::REPLACE,
                window_id,
                *strut_partial_atom,
                AtomEnum::CARDINAL,
                32,
                12,
                &strut_partial,
            )?;
        }
        
        conn.flush()?;
        Ok(())
    }

    #[cfg(feature = "wayland")]
    fn setup_wayland_strut(&self, window: &Window) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement layer-shell protocol for Wayland
        Ok(())
    }
}