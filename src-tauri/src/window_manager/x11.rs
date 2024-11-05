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
        let conn = Arc::new(conn);
        let setup = conn.setup();
        let screen = &setup.roots[screen_num];
        let root = screen.root;
        let windows = Arc::new(Mutex::new(HashMap::new()));

        Ok(X11Manager {
            conn,
            root,
            windows,
        })
    }

    fn get_required_atoms(
        &self,
    ) -> Result<HashMap<&'static str, Atom>, Box<dyn std::error::Error>> {
        let mut atoms = HashMap::new();
        let names = [
            "_NET_CLIENT_LIST",
            "_NET_WM_NAME",
            "_NET_WM_ICON_NAME",
            "_NET_WM_STATE",
            "_NET_WM_STATE_HIDDEN",
            "UTF8_STRING",
        ];

        for name in names.iter() {
            let cookie = self.conn.intern_atom(false, name.as_bytes())?;
            let reply = cookie.reply()?;
            atoms.insert(*name, reply.atom);
        }

        Ok(atoms)
    }

    fn get_window_title(
        &self,
        win: Window,
        atoms: &HashMap<&'static str, Atom>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let cookie = self.conn.get_property(
            false,
            win,
            atoms["_NET_WM_NAME"],
            atoms["UTF8_STRING"],
            0,
            u32::MAX,
        )?;
        let reply = cookie.reply()?;

        Ok(String::from_utf8_lossy(&reply.value).into_owned())
    }

    fn get_window_icon(
        &self,
        win: Window,
        atoms: &HashMap<&'static str, Atom>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let cookie = self.conn.get_property(
            false,
            win,
            atoms["_NET_WM_ICON_NAME"],
            AtomEnum::CARDINAL,
            0,
            u32::MAX,
        )?;
        let reply = cookie.reply()?;

        Ok(String::from_utf8_lossy(&reply.value).into_owned())
    }

    fn get_window_state(
        &self,
        win: Window,
        atoms: &HashMap<&'static str, Atom>,
    ) -> Result<Vec<Atom>, Box<dyn std::error::Error>> {
        let cookie = self.conn.get_property(
            false,
            win,
            atoms["_NET_WM_STATE"],
            AtomEnum::ATOM,
            0,
            u32::MAX,
        )?;
        let reply = cookie.reply()?;

        Ok(reply
            .value32()
            .map(|iter| iter.collect())
            .unwrap_or_default())
    }
}

impl WindowManagerBackend for X11Manager {
    fn get_window_list(&self) -> Result<Vec<WindowInfo>, Box<dyn std::error::Error>> {
        let atoms = self.get_required_atoms()?;
        let cookie = self.conn.get_property(
            false,
            self.root,
            atoms["_NET_CLIENT_LIST"],
            AtomEnum::WINDOW,
            0,
            u32::MAX,
        )?;
        let reply = cookie.reply()?;

        let windows: Vec<Window> = reply.value32().map_or_else(Vec::new, |iter| iter.collect());

        let mut window_list = Vec::new();
        for win in windows {
            let title = self.get_window_title(win, &atoms)?;
            let state = self.get_window_state(win, &atoms)?;
            let icon = self.get_window_icon(win, &atoms)?;

            println!("{}: {:?}", title, icon);

            window_list.push(WindowInfo {
                id: win.to_string(),
                title,
                is_minimized: state.contains(&atoms["_NET_WM_STATE_HIDDEN"]),
                icon,
            });
        }

        Ok(window_list)
    }

    fn focus_window(&self, win_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let win = win_id.parse::<u32>()?;
        self.conn
            .set_input_focus(InputFocus::PARENT, win, x11rb::CURRENT_TIME)?;
        self.conn.flush()?;
        Ok(())
    }

    fn minimize_window(&self, win_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let win = win_id.parse::<u32>()?;
        let atoms = self.get_required_atoms()?;

        let event = ClientMessageEvent::new(
            32,
            win,
            atoms["_NET_WM_STATE"],
            [
                1, // _NET_WM_STATE_ADD
                atoms["_NET_WM_STATE_HIDDEN"] as u32,
                0,
                0,
                0,
            ],
        );

        self.conn.send_event(
            false,
            self.root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            event,
        )?;
        self.conn.flush()?;
        Ok(())
    }

    fn setup_event_monitoring(&mut self, tx: Sender<()>) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.clone();
        let root = self.root;
        let windows = self.windows.clone();

        let events = EventMask::STRUCTURE_NOTIFY
            | EventMask::PROPERTY_CHANGE
            | EventMask::SUBSTRUCTURE_NOTIFY;

        conn.change_window_attributes(root, &ChangeWindowAttributesAux::new().event_mask(events))?;
        conn.flush()?;

        thread::spawn(move || loop {
            if let Ok(event) = conn.wait_for_event() {
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
