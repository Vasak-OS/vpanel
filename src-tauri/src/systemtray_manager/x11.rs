use super::{SystemTrayBackend, TrayIcon};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;

pub struct X11TrayManager {
    conn: Arc<x11rb::xcb_ffi::XCBConnection>,
    root: Window,
    tray_icons: Arc<Mutex<HashMap<Window, TrayIcon>>>,
    event_sender: Option<Sender<()>>,
}

impl X11TrayManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (conn, screen_num) = x11rb::xcb_ffi::XCBConnection::connect(None)?;
        let conn = Arc::new(conn);
        let setup = conn.setup();
        let screen = &setup.roots[screen_num];
        let root = screen.root;

        Ok(X11TrayManager {
            conn,
            root,
            tray_icons: Arc::new(Mutex::new(HashMap::new())),
            event_sender: None,
        })
    }

    pub fn setup_event_monitoring(
        &mut self,
        tx: Sender<()>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.event_sender = Some(tx.clone());
        let conn = self.conn.clone();
        let tray_icons = self.tray_icons.clone();

        let atoms = self.get_required_atoms()?;

        // Register for property change events on the root window
        conn.change_window_attributes(
            self.root,
            &ChangeWindowAttributesAux::new()
                .event_mask(EventMask::PROPERTY_CHANGE | EventMask::STRUCTURE_NOTIFY),
        )?;

        thread::spawn(move || loop {
            if let Ok(event) = conn.wait_for_event() {
                match event {
                    Event::PropertyNotify(e) => {
                        if e.atom == atoms["_NET_SYSTEM_TRAY_S0"] {
                            let _ = tx.send(());
                        }
                    }
                    Event::DestroyNotify(e) => {
                        if tray_icons.lock().unwrap().remove(&e.window).is_some() {
                            let _ = tx.send(());
                        }
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    fn get_required_atoms(
        &self,
    ) -> Result<HashMap<&'static str, Atom>, Box<dyn std::error::Error>> {
        let mut atoms = HashMap::new();
        let names = [
            "_NET_SYSTEM_TRAY_S0",
            "_NET_SYSTEM_TRAY_OPCODE",
            "_XEMBED",
            "_XEMBED_INFO",
        ];

        for name in names.iter() {
            let cookie = self.conn.intern_atom(false, name.as_bytes())?;
            let reply = cookie.reply()?;
            atoms.insert(*name, reply.atom);
        }

        Ok(atoms)
    }

    fn get_tray_icon_info(&self, win: Window) -> Result<TrayIcon, Box<dyn std::error::Error>> {
        // Get icon properties like _NET_WM_ICON if available
        let atoms = self.get_required_atoms()?;
        let cookie = self.conn.get_property(
            false,
            win,
            atoms["_NET_WM_ICON"],
            AtomEnum::CARDINAL,
            0,
            u32::MAX,
        )?;
        let reply = cookie.reply()?;

        let icon_data = if !reply.value.is_empty() {
            Some(STANDARD.encode(&reply.value))
        } else {
            None
        };

        Ok(TrayIcon {
            id: win.to_string(),
            wid: win,
            icon_data,
        })
    }
}

impl SystemTrayBackend for X11TrayManager {
    fn get_tray_icons(&self) -> Result<Vec<TrayIcon>, Box<dyn std::error::Error>> {
        let atoms = self.get_required_atoms()?;
        let cookie = self.conn.get_property(
            false,
            self.root,
            atoms["_NET_SYSTEM_TRAY_S0"],
            AtomEnum::WINDOW,
            0,
            u32::MAX,
        )?;
        let reply = cookie.reply()?;

        let icons: Vec<Window> = match reply.value32() {
            Some(iter) => iter.collect(),
            None => Vec::new(),
        };

        let mut icon_list = Vec::new();
        let mut updated_icons = self.tray_icons.lock().unwrap();

        for win in icons {
            if let Ok(icon) = self.get_tray_icon_info(win) {
                updated_icons.insert(win, icon.clone());
                icon_list.push(icon);
            }
        }

        Ok(icon_list)
    }

    fn setup_event_monitoring(&mut self, tx: Sender<()>) -> Result<(), Box<dyn std::error::Error>> {
        self.setup_event_monitoring(tx)
    }
}
