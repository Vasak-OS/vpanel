use super::{WindowInfo, WindowManagerBackend};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use gtk::prelude::*;
use gtk::IconTheme;
use std::sync::atomic::{AtomicBool, Ordering};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

pub struct X11Manager {
    conn: Arc<x11rb::rust_connection::RustConnection>,
    root: Window,
    windows: Arc<Mutex<HashMap<Window, WindowInfo>>>,
    icon_cache: Arc<Mutex<HashMap<String, String>>>,
    running: Arc<AtomicBool>,
}

impl X11Manager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (conn, screen_num) = x11rb::connect(None)?;
        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;
        let conn = Arc::new(conn);

        // Inicializar GTK solo si no está inicializado
        if gtk::is_initialized() == false {
            if let Err(e) = gtk::init() {
                eprintln!("Failed to initialize GTK: {}", e);
            }
        }
        
        Ok(X11Manager {
            conn,
            root,
            windows: Arc::new(Mutex::new(HashMap::new())),
            icon_cache: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(AtomicBool::new(true)),
        })
    }

    fn get_window_icon(&self, class_name: &str) -> Option<String> {
        // Primero intentar obtener del cache
        if let Ok(cache) = self.icon_cache.lock() {
            if let Some(icon) = cache.get(class_name) {
                return Some(icon.clone());
            }
        }

        // Si no está en cache, buscar el icono
        if gtk::is_initialized() {
            if let Some(icon_theme) = IconTheme::default() {
                let icon_names = [
                    class_name.to_lowercase(),
                    format!("{}.desktop", class_name.to_lowercase()),
                    format!("{}-symbolic", class_name.to_lowercase()),
                    "application-x-executable".to_string(),
                ];

                for icon_name in icon_names.iter() {
                    if let Some(icon_info) = icon_theme.lookup_icon(
                        icon_name,
                        48,
                        gtk::IconLookupFlags::FORCE_SIZE
                    ) {
                        if let Some(path) = icon_info.filename() {
                            if let Ok(icon_data) = std::fs::read(path) {
                                let icon_base64 = BASE64.encode(&icon_data);
                                // Guardar en cache
                                if let Ok(mut cache) = self.icon_cache.lock() {
                                    cache.insert(class_name.to_string(), icon_base64.clone());
                                }
                                return Some(icon_base64);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn get_required_atoms(&self) -> Result<HashMap<&'static str, Atom>, Box<dyn std::error::Error>> {
        let atom_names = [
            "_NET_CLIENT_LIST",
            "_NET_WM_NAME",
            "_NET_WM_STATE",
            "_NET_WM_STATE_HIDDEN",
            "_NET_WM_STATE_SKIP_TASKBAR",
            "_NET_WM_STATE_SKIP_PAGER",
            "_NET_WM_STATE_MODAL",
            "_NET_WM_WINDOW_TYPE",
            "_NET_WM_WINDOW_TYPE_DOCK",
            "_NET_WM_WINDOW_TYPE_DESKTOP",
            "_NET_WM_WINDOW_TYPE_NOTIFICATION",
            "_NET_WM_WINDOW_TYPE_TOOLBAR",
            "_NET_WM_WINDOW_TYPE_MENU",
            "_NET_WM_WINDOW_TYPE_SPLASH",
            "_NET_WM_WINDOW_TYPE_DIALOG",
            "_NET_WM_WINDOW_TYPE_UTILITY",
            "_NET_WM_WINDOW_TYPE_TOOLTIP",
            "_NET_WM_WINDOW_TYPE_POPUP_MENU",
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

    fn focus_window(
        &self,
        window: Window,
        atoms: &HashMap<&str, u32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let net_active_window_atom = atoms["_NET_ACTIVE_WINDOW"];
        let root = self.conn.setup().roots[0].root;

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

    fn should_show_window(&self, win: Window, atoms: &HashMap<&str, Atom>) -> Result<bool, Box<dyn std::error::Error>> {
        let state = self.get_window_state(win, atoms)?;
        let skip_states = [
            atoms["_NET_WM_STATE_SKIP_TASKBAR"],
            atoms["_NET_WM_STATE_SKIP_PAGER"],
            atoms["_NET_WM_STATE_MODAL"],
        ];

        if state.iter().any(|s| skip_states.contains(s)) {
            return Ok(false);
        }

        let class_name = self.get_window_class(win)?;
        let skip_classes = [
            "desktop_window",
            "Plank",
            "Tint2",
            "Wrapper",
            "wrapper-2.0",
            "notification",
            "Notification",
            "vpanel",
            "panel",
            "dock",
            "toolbar",
            "menu",
        ];

        if skip_classes.iter().any(|c| class_name.to_lowercase().contains(c)) {
            return Ok(false);
        }

        let title = self.get_window_title(win, atoms)?;
        if title.trim().is_empty() {
            return Ok(false);
        }

        let attrs = self.conn.get_window_attributes(win)?.reply()?;
        if attrs.map_state != MapState::VIEWABLE {
            return Ok(false);
        }

        if let Ok(geom) = self.conn.get_geometry(win)?.reply() {
            if geom.width < 50 || geom.height < 50 {
                return Ok(false);
            }
        }

        Ok(true)
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

impl Drop for X11Manager {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
