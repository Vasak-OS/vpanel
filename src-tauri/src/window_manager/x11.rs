use super::{WindowInfo, WindowManagerBackend};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::CURRENT_TIME;

pub struct X11Manager {
    conn: Arc<x11rb::rust_connection::RustConnection>,
    root: Window,
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
            running: Arc::new(AtomicBool::new(true)),
        })
    }

    fn get_required_atoms(
        &self,
    ) -> Result<HashMap<&'static str, Atom>, Box<dyn std::error::Error>> {
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
            "_NET_WM_STATE_DEMANDS_ATTENTION",
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

    fn get_window_title(
        &self,
        win: Window,
        atoms: &HashMap<&str, Atom>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let reply = self
            .conn
            .get_property(
                false,
                win,
                atoms["_NET_WM_NAME"],
                atoms["UTF8_STRING"],
                0,
                u32::MAX,
            )?
            .reply()?;

        Ok(String::from_utf8_lossy(&reply.value).into_owned())
    }

    fn get_window_state(
        &self,
        win: Window,
        atoms: &HashMap<&str, Atom>,
    ) -> Result<Vec<Atom>, Box<dyn std::error::Error>> {
        let reply = self
            .conn
            .get_property(
                false,
                win,
                atoms["_NET_WM_STATE"],
                AtomEnum::ATOM,
                0,
                u32::MAX,
            )?
            .reply()?;

        Ok(reply
            .value32()
            .map(|iter| iter.collect())
            .unwrap_or_default())
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
            let mut parts = data_vec.split(|&b| b == 0);
            if let Some(first_part_bytes) = parts.next() {
                if !first_part_bytes.is_empty() {
                    return Ok(String::from_utf8_lossy(first_part_bytes).into_owned());
                }
                if let Some(second_part_bytes) = parts.next() {
                    if !second_part_bytes.is_empty() {
                        return Ok(String::from_utf8_lossy(second_part_bytes).into_owned());
                    }
                }
            }
        }

        Ok(String::new())
    }

    fn is_window_focused(
        &self,
        window: Window,
        atoms: &HashMap<&str, Atom>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let net_active_window_atom = atoms
            .get("_NET_ACTIVE_WINDOW")
            .ok_or("_NET_ACTIVE_WINDOW atom not found")?;
        let active_window_reply = self
            .conn
            .get_property(
                false,
                self.root,
                *net_active_window_atom,
                AtomEnum::WINDOW,
                0,
                1,
            )?
            .reply()?;

        Ok(active_window_reply.value32().and_then(|mut v| v.next()) == Some(window))
    }

    fn change_net_wm_state(
        &self,
        win: Window,
        action: u32,
        property_atom: Atom,
        atoms: &HashMap<&str, Atom>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let net_wm_state_atom = atoms
            .get("_NET_WM_STATE")
            .ok_or("_NET_WM_STATE atom not found")?;

        let event = ClientMessageEvent {
            response_type: CLIENT_MESSAGE_EVENT,
            format: 32,
            sequence: 0,
            window: win,
            type_: *net_wm_state_atom,
            data: ClientMessageData::from([action, property_atom, 0, 2, 0]),
        };

        self.conn.send_event(
            false,
            self.root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            event,
        )?;
        Ok(())
    }

    fn activate_window_ewmh(
        &self,
        window: Window,
        atoms: &HashMap<&str, Atom>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let net_active_window_atom = atoms
            .get("_NET_ACTIVE_WINDOW")
            .ok_or("_NET_ACTIVE_WINDOW atom not found")?;

        let current_active_window_for_source: u32 = 0;

        let event = ClientMessageEvent {
            response_type: CLIENT_MESSAGE_EVENT,
            format: 32,
            sequence: 0,
            window,
            type_: *net_active_window_atom,
            data: ClientMessageData::from([
                2,
                CURRENT_TIME,
                current_active_window_for_source,
                0,
                0,
            ]),
        };

        self.conn.send_event(
            false,
            self.root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            event,
        )?;
        Ok(())
    }

    fn should_show_window(
        &self,
        win: Window,
        atoms: &HashMap<&str, Atom>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let attributes = self.conn.get_window_attributes(win)?.reply()?;
        if attributes.override_redirect {
            return Ok(false);
        }
        // if attributes.map_state != MapState::VIEWABLE {
        //     return Ok(false);
        // }

        // Verificar el tipo de ventana
        if let Some(net_wm_window_type_atom) = atoms.get("_NET_WM_WINDOW_TYPE") {
            if let Ok(reply) = self
                .conn
                .get_property(
                    false,
                    win,
                    *net_wm_window_type_atom,
                    AtomEnum::ATOM,
                    0,
                    u32::MAX,
                )?
                .reply()
            {
                if let Some(types) = reply.value32() {
                    for window_type_atom_value in types {
                        if Some(&window_type_atom_value) == atoms.get("_NET_WM_WINDOW_TYPE_DOCK")
                            || Some(&window_type_atom_value)
                                == atoms.get("_NET_WM_WINDOW_TYPE_TOOLBAR")
                            || Some(&window_type_atom_value)
                                == atoms.get("_NET_WM_WINDOW_TYPE_DESKTOP")
                            || Some(&window_type_atom_value)
                                == atoms.get("_NET_WM_WINDOW_TYPE_SPLASH")
                            || Some(&window_type_atom_value)
                                == atoms.get("_NET_WM_WINDOW_TYPE_UTILITY")
                            || Some(&window_type_atom_value)
                                == atoms.get("_NET_WM_WINDOW_TYPE_DROPDOWN_MENU")
                            || Some(&window_type_atom_value)
                                == atoms.get("_NET_WM_WINDOW_TYPE_POPUP_MENU")
                            || Some(&window_type_atom_value)
                                == atoms.get("_NET_WM_WINDOW_TYPE_NOTIFICATION")
                        {
                            return Ok(false);
                        }
                    }
                }
            }
        }

        // Verificar la clase de la ventana
        let class_name = self.get_window_class(win)?;
        let skip_classes = [
            "vpanel",
            "tauri",
            "trayer",
            "plank",
            "docky",
            "cairo-dock",
            "tint2",
            "polybar",
            "lemonbar",
            "vmenu",
            "vasak-control-center",
        ];

        if skip_classes
            .iter()
            .any(|c| class_name.to_lowercase().contains(c))
        {
            return Ok(false);
        }

        Ok(true)
    }
}

impl WindowManagerBackend for X11Manager {
    fn get_window_list(&self) -> Result<Vec<WindowInfo>, Box<dyn std::error::Error>> {
        let atoms = self.get_required_atoms()?;
        let net_client_list_atom = atoms
            .get("_NET_CLIENT_LIST")
            .ok_or("_NET_CLIENT_LIST atom not found")?;
        let net_wm_state_hidden_atom = atoms
            .get("_NET_WM_STATE_HIDDEN")
            .ok_or("_NET_WM_STATE_HIDDEN atom not found")?;

        let reply = self
            .conn
            .get_property(
                false,
                self.root,
                *net_client_list_atom,
                AtomEnum::WINDOW,
                0,
                u32::MAX,
            )?
            .reply()?;

        let windows_prop: Vec<Window> =
            reply.value32().map_or_else(Vec::new, |iter| iter.collect());
        let mut window_list = Vec::new();

        for win in windows_prop {
            if !self.should_show_window(win, &atoms)? {
                continue;
            }
            let title = self.get_window_title(win, &atoms).unwrap_or_default();
            let state = self.get_window_state(win, &atoms)?;
            let class_name = self.get_window_class(win).unwrap_or_default();

            window_list.push(WindowInfo {
                id: win.to_string(),
                title,
                is_minimized: state.contains(net_wm_state_hidden_atom),
                icon: class_name,
            });
        }

        Ok(window_list)
    }

    fn toggle_window(&self, win_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let window_to_toggle = win_id.parse::<Window>()?;
        let atoms = self.get_required_atoms()?;

        let net_wm_state_hidden_atom =
            atoms.get("_NET_WM_STATE_HIDDEN").copied().ok_or_else(|| {
                Into::<Box<dyn std::error::Error>>::into("_NET_WM_STATE_HIDDEN atom not found")
            })?;
        
        let net_wm_state_demands_attention_atom = atoms.get("_NET_WM_STATE_DEMANDS_ATTENTION").copied();


        let window_state = self.get_window_state(window_to_toggle, &atoms)?;
        let is_hidden = window_state.contains(&net_wm_state_hidden_atom);

        eprintln!("[toggle_window] ID: {}, IsHidden: {}", win_id, is_hidden);

        if is_hidden {
            eprintln!("[toggle_window] Ventana está oculta. Intentando desminimizar y activar.");
            self.change_net_wm_state(window_to_toggle, 0, net_wm_state_hidden_atom, &atoms)?; // 0 = remove state
            
            if let Some(da_atom) = net_wm_state_demands_attention_atom {
                eprintln!("[toggle_window] Añadiendo DEMANDS_ATTENTION");
                self.change_net_wm_state(window_to_toggle, 1, da_atom, &atoms)?; // 1 = add state
            }

            self.activate_window_ewmh(window_to_toggle, &atoms)?;

            if let Some(da_atom) = net_wm_state_demands_attention_atom {
                 eprintln!("[toggle_window] Quitando DEMANDS_ATTENTION");
                self.change_net_wm_state(window_to_toggle, 0, da_atom, &atoms)?; // 0 = remove state
            }

        } else {
            let is_focused = self.is_window_focused(window_to_toggle, &atoms)?;
            eprintln!("[toggle_window] Ventana no oculta. IsFocused: {}", is_focused);
            if is_focused {
                eprintln!("[toggle_window] Ventana visible y enfocada. Intentando minimizar.");
                self.change_net_wm_state(window_to_toggle, 1, net_wm_state_hidden_atom, &atoms)?; // 1 = add state
            } else {
                eprintln!("[toggle_window] Ventana visible pero no enfocada. Intentando activar.");
                if let Some(da_atom) = net_wm_state_demands_attention_atom {
                    if window_state.contains(&da_atom) { // Solo quitar si estaba puesto
                        self.change_net_wm_state(window_to_toggle, 0, da_atom, &atoms)?;
                    }
                }
                self.activate_window_ewmh(window_to_toggle, &atoms)?;
            }
        }
        self.conn.flush()?;
        eprintln!("[toggle_window] Operación completada para ID: {}", win_id);
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
