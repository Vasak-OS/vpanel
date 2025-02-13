use super::{WindowInfo, WindowManagerBackend};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;

pub struct X11Manager {
    conn: Arc<x11rb::rust_connection::RustConnection>,
    root: Window,
    windows: Arc<Mutex<HashMap<Window, WindowInfo>>>,
}

impl X11Manager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (conn, screen_num) = x11rb::connect(None)?;
        let root = conn.setup().roots[screen_num].root;
        let conn = Arc::new(conn);
        
        Ok(X11Manager {
            conn,
            root,
            windows: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    // Funciones auxiliares para manejo de átomos y propiedades de ventanas
    fn get_required_atoms(&self) -> Result<HashMap<&'static str, Atom>, Box<dyn std::error::Error>> {
        let atom_names = [
            "_NET_CLIENT_LIST",
            "_NET_WM_NAME",
            "_NET_WM_STATE",
            "_NET_WM_STATE_HIDDEN",
            "UTF8_STRING",
            "_NET_ACTIVE_WINDOW",
        ];

        let mut atoms = HashMap::new();
        for name in atom_names.iter() {
            let reply = self.conn.intern_atom(false, name.as_bytes())?.reply()?;
            atoms.insert(*name, reply.atom);
        }
        Ok(atoms)
    }

    // Funciones principales de gestión de ventanas
    fn get_window_title(&self, win: Window, atoms: &HashMap<&str, Atom>) 
        -> Result<String, Box<dyn std::error::Error>> {
        let reply = self.conn.get_property(
            false,
            win,
            atoms["_NET_WM_NAME"],
            atoms["UTF8_STRING"],
            0,
            u32::MAX,
        )?.reply()?;

        Ok(String::from_utf8_lossy(&reply.value).into_owned())
    }

    fn get_window_state(&self, win: Window, atoms: &HashMap<&str, Atom>) 
        -> Result<Vec<Atom>, Box<dyn std::error::Error>> {
        let reply = self.conn.get_property(
            false,
            win,
            atoms["_NET_WM_STATE"],
            AtomEnum::ATOM,
            0,
            u32::MAX,
        )?.reply()?;

        Ok(reply.value32().map(|iter| iter.collect()).unwrap_or_default())
    }

    fn get_window_class(&self, win: Window) -> Result<String, Box<dyn std::error::Error>> {
        let cookie = self.conn.get_property(
            false,
            win,
            AtomEnum::WM_CLASS,
            AtomEnum::STRING,
            0,
            u32::MAX,
        )?;
        let reply = cookie.reply()?;

        if let Some(data) = reply.value8() {
            let data_vec: Vec<u8> = data.collect();
            let utf8_string = String::from_utf8_lossy(&data_vec);
            let parts: Vec<&str> = utf8_string.split('\0').collect();
            if parts.len() > 1 {
                return Ok(parts[0].to_string());
            }
        }

        Ok(String::new())
    }

    fn is_window_focused(
        &self,
        window: Window,
        atoms: &HashMap<&str, u32>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let net_active_window_atom = atoms["_NET_ACTIVE_WINDOW"];
        let active_window_reply = self
            .conn
            .get_property(
                false,
                self.conn.setup().roots[0].root,
                net_active_window_atom,
                AtomEnum::WINDOW,
                0,
                1,
            )?
            .reply()?;

        // Verificar si la ventana actual está en foco
        Ok(active_window_reply
            .value32()
            .map(|mut v| v.next() == Some(window))
            .unwrap_or(false))
    }

    fn minimize_window(
        &self,
        window: Window,
        atoms: &HashMap<&str, u32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let net_wm_state_atom = atoms["_NET_WM_STATE"];
        let net_wm_state_hidden_atom = atoms["_NET_WM_STATE_HIDDEN"];
        let root = self.conn.setup().roots[0].root;

        // Crear y enviar el evento de minimización
        let event = ClientMessageEvent {
            response_type: x11rb::protocol::xproto::CLIENT_MESSAGE_EVENT,
            format: 32,
            window,
            type_: net_wm_state_atom,
            data: [1, net_wm_state_hidden_atom, 0, 0, 0].into(),
            sequence: 0,
        };

        self.conn.send_event(
            false,
            root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            event,
        )?;
        Ok(())
    }

    /// Lleva la ventana al foco
    fn focus_window(
        &self,
        window: Window,
        atoms: &HashMap<&str, u32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let net_active_window_atom = atoms["_NET_ACTIVE_WINDOW"];
        let root = self.conn.setup().roots[0].root;

        // Crear y enviar el evento para traer la ventana al foco
        let event = ClientMessageEvent {
            response_type: x11rb::protocol::xproto::CLIENT_MESSAGE_EVENT,
            format: 32,
            window,
            type_: net_active_window_atom,
            data: [1, 0, 0, 0, 0].into(),
            sequence: 0,
        };

        self.conn.send_event(
            false,
            root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            event,
        )?;
        Ok(())
    }
}

impl WindowManagerBackend for X11Manager {
    fn get_window_list(&self) -> Result<Vec<WindowInfo>, Box<dyn std::error::Error>> {
        let atoms = self.get_required_atoms()?;
        let reply = self.conn.get_property(
            false,
            self.root,
            atoms["_NET_CLIENT_LIST"],
            AtomEnum::WINDOW,
            0,
            u32::MAX,
        )?.reply()?;

        let windows: Vec<Window> = reply.value32().map_or_else(Vec::new, |iter| iter.collect());
        let mut window_list = Vec::new();

        for win in windows {
            let title = self.get_window_title(win, &atoms)?;
            let state = self.get_window_state(win, &atoms)?;
            
            window_list.push(WindowInfo {
                id: win.to_string(),
                title,
                is_minimized: state.contains(&atoms["_NET_WM_STATE_HIDDEN"]),
                icon: self.get_window_class(win)?,
            });
        }

        Ok(window_list)
    }

    fn toggle_window(&self, win_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let win = win_id.parse::<u32>()?;
        let atoms = self.get_required_atoms()?;
        let root = self.conn.setup().roots[0].root;

        // Evento para alternar el estado de la ventana
        let event = ClientMessageEvent {
            response_type: CLIENT_MESSAGE_EVENT,
            format: 32,
            window: win,
            type_: atoms["_NET_ACTIVE_WINDOW"],
            data: [1, 0, 0, 0, 0].into(),
            sequence: 0,
        };

        self.conn.send_event(
            false,
            root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            event,
        )?;

        Ok(())
    }

    fn setup_event_monitoring(&mut self, tx: Sender<()>) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.clone();
        let events = EventMask::STRUCTURE_NOTIFY
            | EventMask::PROPERTY_CHANGE
            | EventMask::SUBSTRUCTURE_NOTIFY;

        conn.change_window_attributes(
            self.root,
            &ChangeWindowAttributesAux::new().event_mask(events),
        )?;

        thread::spawn(move || {
            while let Ok(event) = conn.wait_for_event() {
                match event {
                    Event::CreateNotify(_) | Event::DestroyNotify(_) | Event::PropertyNotify(_) => {
                        let _ = tx.send(());
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }
}
