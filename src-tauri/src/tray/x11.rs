use std::sync::Arc;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event as X11Event;
use x11rb::connection::Connection;
use image::codecs::png::PngEncoder;
use image::{ImageEncoder, RgbaImage};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use tauri::Emitter;
use super::{TrayBackend, TrayItem};

pub struct X11TrayBackend {
    conn: Arc<x11rb::rust_connection::RustConnection>,
    root: Window,
}

impl X11TrayBackend {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (conn, screen_num) = x11rb::connect(None)?;
        let root = conn.setup().roots[screen_num].root;
        Ok(Self {
            conn: Arc::new(conn),
            root,
        })
    }

    fn get_window_icon(&self, window: Window) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let wm_icon = self.conn.intern_atom(false, b"_NET_WM_ICON")?.reply()?.atom;
        
        let icon_prop = self.conn.get_property(
            false,
            window,
            wm_icon,
            AtomEnum::CARDINAL,
            0,
            1024 * 1024
        )?.reply()?;

        if icon_prop.value.is_empty() {
            return Ok(None);
        }

        let mut values = icon_prop.value32().unwrap();
        let width = values.next().unwrap() as u32;
        let height = values.next().unwrap() as u32;
        
        let data: Vec<u8> = values
            .flat_map(|pixel| {
                let r = ((pixel >> 16) & 0xFF) as u8;
                let g = ((pixel >> 8) & 0xFF) as u8;
                let b = (pixel & 0xFF) as u8;
                let a = ((pixel >> 24) & 0xFF) as u8;
                vec![r, g, b, a]
            })
            .collect();

        let img = RgbaImage::from_raw(width, height, data)
            .ok_or("Failed to create image")?;
        
        let mut png_data = Vec::new();
        let encoder = PngEncoder::new(&mut png_data);
        encoder.write_image(
            &img.into_raw(),
            width,
            height,
            image::ColorType::Rgba8.into()
        )?;
        
        Ok(Some(BASE64.encode(&png_data)))
    }

    fn get_window_title(&self, window: Window) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let atoms = [
            "_NET_WM_NAME",
            "WM_NAME",
            "_NET_WM_VISIBLE_NAME"
        ];

        for atom_name in atoms {
            let atom = self.conn.intern_atom(false, atom_name.as_bytes())?.reply()?.atom;
            let utf8_string = self.conn.intern_atom(false, b"UTF8_STRING")?.reply()?.atom;
            
            let title_prop = self.conn.get_property(
                false,
                window,
                atom,
                utf8_string,
                0,
                1024
            )?.reply()?;

            if !title_prop.value.is_empty() {
                return Ok(Some(String::from_utf8_lossy(&title_prop.value).into_owned()));
            }
        }

        Ok(None)
    }
}

impl TrayBackend for X11TrayBackend {
    fn get_tray_items(&self) -> Result<Vec<TrayItem>, Box<dyn std::error::Error>> {
        let protocols = [
            "_NET_SYSTEM_TRAY_S0",
            "_NET_SYSTEM_TRAY_S1",
            "SYSTEM_TRAY_S0",
            "_XEMBED_INFO"
        ];

        let mut tray_windows = Vec::new();

        for protocol in protocols {
            let tray_atom = self.conn.intern_atom(false, protocol.as_bytes())?.reply()?.atom;
            let owner = self.conn.get_selection_owner(tray_atom)?.reply()?.owner;
            
            if owner != x11rb::NONE {
                let tree = self.conn.query_tree(owner)?.reply()?;
                if !tree.children.is_empty() {
                    tray_windows.extend(tree.children);
                    break;
                }
            }
        }

        let mut items = Vec::new();
        for window in tray_windows {
            let attrs = self.conn.get_window_attributes(window)?.reply()?;
            if attrs.map_state != MapState::VIEWABLE {
                continue;
            }

            let icon_data = self.get_window_icon(window)?;
            let title = self.get_window_title(window)?;

            items.push(TrayItem {
                id: window.to_string(),
                wid: window as i32,
                icon_data,
                title,
            });
        }

        Ok(items)
    }

    fn setup_monitoring(&self, app_handle: tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.clone();
        let root = self.root;

        let events = EventMask::STRUCTURE_NOTIFY
            | EventMask::PROPERTY_CHANGE
            | EventMask::SUBSTRUCTURE_NOTIFY;

        conn.change_window_attributes(
            root,
            &ChangeWindowAttributesAux::new().event_mask(events),
        )?;

        std::thread::spawn(move || {
            while let Ok(event) = conn.wait_for_event() {
                match event {
                    X11Event::PropertyNotify(_) | 
                    X11Event::CreateNotify(_) | 
                    X11Event::DestroyNotify(_) => {
                        let _ = app_handle.emit("tray-update", ());
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }
} 