use super::{WindowInfo, WindowManagerBackend};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use wayland_client::protocol::{wl_output, wl_registry, wl_seat};
use wayland_client::{Connection, Dispatch, EventQueue, Proxy, QueueHandle};
use wayland_protocols_wlr::foreign_toplevel::v1::client::{
    zwlr_foreign_toplevel_handle_v1::{self, ZwlrForeignToplevelHandleV1},
    zwlr_foreign_toplevel_manager_v1::{self, ZwlrForeignToplevelManagerV1},
};

#[derive(Debug, Clone)]
struct ToplevelInfo {
    handle: ZwlrForeignToplevelHandleV1,
    title: String,
    app_id: String,
    is_maximized: bool,
    is_minimized: bool,
    is_activated: bool,
    is_fullscreen: bool,
}

impl ToplevelInfo {
    fn new(handle: ZwlrForeignToplevelHandleV1) -> Self {
        Self {
            handle,
            title: String::new(),
            app_id: String::new(),
            is_maximized: false,
            is_minimized: false,
            is_activated: false,
            is_fullscreen: false,
        }
    }

    fn to_window_info(&self, id: &str) -> WindowInfo {
        WindowInfo {
            id: id.to_string(),
            title: self.title.clone(),
            is_minimized: self.is_minimized,
            icon: self.app_id.clone(),
            demands_attention: None, // Wayland doesn't have direct equivalent
        }
    }

    fn should_show(&self) -> bool {
        let skip_apps = [
            "vpanel",
            "tauri", 
            "vasak-control-center",
            "plank",
            "docky",
            "cairo-dock",
            "polybar",
            "waybar",
            "tint2"
        ];
        
        let app_id_lower = self.app_id.to_lowercase();
        !skip_apps.iter().any(|app| app_id_lower.contains(app))
    }
}

struct AppState {
    toplevels: HashMap<u32, ToplevelInfo>,
    manager: Option<ZwlrForeignToplevelManagerV1>,
    seat: Option<wl_seat::WlSeat>,
    event_sender: Option<Sender<()>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            toplevels: HashMap::new(),
            manager: None,
            seat: None,
            event_sender: None,
        }
    }
}

pub struct WaylandManager {
    conn: Connection,
    event_queue: EventQueue<AppState>,
    state: Arc<Mutex<AppState>>,
}

impl WaylandManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::connect_to_env()
            .map_err(|e| format!("Failed to connect to Wayland: {}", e))?;
        
        let event_queue = conn.new_event_queue::<AppState>();
        let qh = event_queue.handle();
        
        let _registry = conn.display().get_registry(&qh, ());
        
        let state = Arc::new(Mutex::new(AppState::new()));

        Ok(WaylandManager {
            conn,
            event_queue,
            state,
        })
    }

    fn setup_protocol_bindings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Dispatch pending events to set up protocol bindings
        self.event_queue.blocking_dispatch(&mut *self.state.lock().unwrap())
            .map_err(|e| format!("Failed to dispatch events: {}", e))?;
        
        // Check if we have the required protocols
        let state_guard = self.state.lock().unwrap();
        if state_guard.manager.is_none() {
            return Err("wlr-foreign-toplevel-management protocol not available".into());
        }
        
        drop(state_guard);
        log::info!("Wayland protocols initialized successfully");
        Ok(())
    }
}

impl WindowManagerBackend for WaylandManager {
    fn get_window_list(&mut self) -> Result<Vec<WindowInfo>, Box<dyn std::error::Error>> {
        // Dispatch any pending events first
        if let Err(e) = self.event_queue.blocking_dispatch(&mut *self.state.lock().unwrap()) {
            log::warn!("Failed to dispatch events: {}", e);
            return Ok(Vec::new());
        }

        let state = self.state.lock().unwrap();
        let windows: Vec<WindowInfo> = state
            .toplevels
            .iter()
            .filter_map(|(id, toplevel)| {
                if toplevel.should_show() {
                    Some(toplevel.to_window_info(&id.to_string()))
                } else {
                    None
                }
            })
            .collect();

        Ok(windows)
    }

    fn setup_event_monitoring(&mut self, tx: Sender<()>) -> Result<(), Box<dyn std::error::Error>> {
        match self.setup_protocol_bindings() {
            Ok(_) => {
                log::info!("Wayland window management initialized successfully");
            }
            Err(e) => {
                log::error!("Failed to initialize Wayland protocols: {}", e);
                log::warn!("Window management monitoring will not be available");
                return Err(e);
            }
        }
        
        // Store the sender for events
        {
            let mut state = self.state.lock().unwrap();
            state.event_sender = Some(tx);
        }

        let state_clone = Arc::clone(&self.state);
        let mut event_queue = self.conn.new_event_queue::<AppState>();
        
        std::thread::spawn(move || {
            loop {
                match event_queue.blocking_dispatch(&mut *state_clone.lock().unwrap()) {
                    Ok(_) => {
                        // Notify of window list changes
                        if let Some(sender) = &state_clone.lock().unwrap().event_sender {
                            let _ = sender.send(());
                        }
                    }
                    Err(e) => {
                        log::error!("Error in Wayland event loop: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    fn toggle_window(&self, win_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let id: u32 = win_id.parse()
            .map_err(|_| "Invalid window ID format")?;

        let state = self.state.lock().unwrap();
        let toplevel = state.toplevels.get(&id)
            .ok_or("Window not found")?;

        if let Some(seat) = &state.seat {
            if toplevel.is_minimized {
                // Unminimize and activate
                toplevel.handle.unset_minimized();
                toplevel.handle.activate(seat);
            } else if toplevel.is_activated {
                // If focused, minimize
                toplevel.handle.set_minimized();
            } else {
                // If not focused, activate
                toplevel.handle.activate(seat);
            }
        } else {
            return Err("No seat available for window operations".into());
        }

        Ok(())
    }
}

// Implement Dispatch for the registry to bind protocols
impl Dispatch<wl_registry::WlRegistry, ()> for AppState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppState>,
    ) {
        if let wl_registry::Event::Global { name, interface, version } = event {
            match interface.as_str() {
                "zwlr_foreign_toplevel_manager_v1" => {
                    if version >= 1 {
                        let manager = registry.bind::<ZwlrForeignToplevelManagerV1, _, _>(
                            name, 
                            1.min(version), 
                            qh, 
                            ()
                        );
                        state.manager = Some(manager);
                    }
                }
                "wl_seat" => {
                    if version >= 1 {
                        let seat = registry.bind::<wl_seat::WlSeat, _, _>(
                            name,
                            1.min(version),
                            qh,
                            ()
                        );
                        state.seat = Some(seat);
                    }
                }
                _ => {}
            }
        }
    }
}

// Implement Dispatch for the foreign toplevel manager
impl Dispatch<ZwlrForeignToplevelManagerV1, ()> for AppState {
    fn event(
        state: &mut Self,
        _: &ZwlrForeignToplevelManagerV1,
        event: zwlr_foreign_toplevel_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppState>,
    ) {
        match event {
            zwlr_foreign_toplevel_manager_v1::Event::Toplevel { toplevel } => {
                let id = toplevel.id().protocol_id();
                let info = ToplevelInfo::new(toplevel);
                state.toplevels.insert(id, info);
            }
            zwlr_foreign_toplevel_manager_v1::Event::Finished => {
                log::info!("Foreign toplevel manager finished");
            }
            _ => {}
        }
    }
}

// Implement Dispatch for individual toplevel handles
impl Dispatch<ZwlrForeignToplevelHandleV1, ()> for AppState {
    fn event(
        state: &mut Self,
        handle: &ZwlrForeignToplevelHandleV1,
        event: zwlr_foreign_toplevel_handle_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppState>,
    ) {
        let id = handle.id().protocol_id();
        
        if let Some(toplevel_info) = state.toplevels.get_mut(&id) {
            match event {
                zwlr_foreign_toplevel_handle_v1::Event::Title { title } => {
                    toplevel_info.title = title;
                }
                zwlr_foreign_toplevel_handle_v1::Event::AppId { app_id } => {
                    toplevel_info.app_id = app_id;
                }
                zwlr_foreign_toplevel_handle_v1::Event::State { state: window_state } => {
                    // Parse the state array
                    let states: Vec<u32> = window_state
                        .chunks_exact(4)
                        .map(|chunk| u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                        .collect();

                    // Reset all states
                    toplevel_info.is_maximized = false;
                    toplevel_info.is_minimized = false;
                    toplevel_info.is_activated = false;
                    toplevel_info.is_fullscreen = false;

                    // Set current states using the enum values from the protocol
                    for state_value in states {
                        match state_value {
                            0 => toplevel_info.is_maximized = true,  // maximized
                            1 => toplevel_info.is_minimized = true,  // minimized  
                            2 => toplevel_info.is_activated = true,  // activated
                            3 => toplevel_info.is_fullscreen = true, // fullscreen
                            _ => {}
                        }
                    }
                }
                zwlr_foreign_toplevel_handle_v1::Event::Closed => {
                    state.toplevels.remove(&id);
                }
                zwlr_foreign_toplevel_handle_v1::Event::Done => {
                    // All properties have been sent, can notify of changes
                    if let Some(sender) = &state.event_sender {
                        let _ = sender.send(());
                    }
                }
                _ => {} // Handle other events as needed
            }
        }
    }
}

// Implement Dispatch for wl_seat (required but we don't need to handle events)
impl Dispatch<wl_seat::WlSeat, ()> for AppState {
    fn event(
        _: &mut Self,
        _: &wl_seat::WlSeat,
        _: wl_seat::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppState>,
    ) {
        // We don't need to handle seat events for this use case
    }
}

// Implement Dispatch for wl_output (may be needed for output events)
impl Dispatch<wl_output::WlOutput, ()> for AppState {
    fn event(
        _: &mut Self,
        _: &wl_output::WlOutput,
        _: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppState>,
    ) {
        // We don't need to handle output events for this use case
    }
}