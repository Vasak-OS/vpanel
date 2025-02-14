use super::{TrayBackend, TrayItem};
use std::sync::{Arc, Mutex};
use wayland_client::{
    protocol::wl_registry,
    Connection, Dispatch, QueueHandle, Proxy,
};
use wayland_protocols::ext::foreign_toplevel_list::v1::client::{
    ext_foreign_toplevel_list_v1,
    ext_foreign_toplevel_handle_v1,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use image::{ImageBuffer, Rgba};
use tauri::Emitter;

#[derive(Default)]
struct WaylandData;

pub struct WaylandTrayBackend {
    items: Arc<Mutex<Vec<TrayItem>>>,
    connection: Arc<Connection>,
    manager: Option<ext_foreign_toplevel_list_v1::ExtForeignToplevelListV1>,
}

impl WaylandTrayBackend {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let connection = Connection::connect_to_env()?;
        let display = connection.display();
        let mut queue = connection.new_event_queue();
        let qh = queue.handle();

        let backend = Self {
            items: Arc::new(Mutex::new(Vec::new())),
            connection: Arc::new(connection),
            manager: None,
        };

        let _registry = display.get_registry(&qh, ());
        
        queue.roundtrip(&mut WaylandData::default())?;

        Ok(backend)
    }

    fn create_icon_data(&self, width: u32, height: u32, rgba_data: &[u8]) -> Option<String> {
        if rgba_data.is_empty() {
            return None;
        }

        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, rgba_data.to_vec())?;
        let mut png_data = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png).ok()?;
        
        Some(BASE64.encode(&png_data))
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for WaylandData {
    fn event(
        _state: &mut Self,
        _registry: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // Implementación vacía por ahora
    }
}

impl Dispatch<ext_foreign_toplevel_list_v1::ExtForeignToplevelListV1, ()> for WaylandTrayBackend {
    fn event(
        state: &mut Self,
        _: &ext_foreign_toplevel_list_v1::ExtForeignToplevelListV1,
        event: ext_foreign_toplevel_list_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            ext_foreign_toplevel_list_v1::Event::Toplevel { toplevel } => {
                let mut items = state.items.lock().unwrap();
                items.push(TrayItem {
                    id: toplevel.id().to_string(),
                    wid: toplevel.id().protocol_id() as i32,
                    icon_data: None,
                    title: None,
                });
            },
            _ => {}
        }
    }
}

impl Dispatch<ext_foreign_toplevel_handle_v1::ExtForeignToplevelHandleV1, ()> for WaylandTrayBackend {
    fn event(
        state: &mut Self,
        toplevel: &ext_foreign_toplevel_handle_v1::ExtForeignToplevelHandleV1,
        event: ext_foreign_toplevel_handle_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let mut items = state.items.lock().unwrap();
        let id = toplevel.id().to_string();
        
        match event {
            ext_foreign_toplevel_handle_v1::Event::Title { title } => {
                if let Some(item) = items.iter_mut().find(|i| i.id == id) {
                    item.title = Some(title);
                }
            },
            ext_foreign_toplevel_handle_v1::Event::AppId { app_id: _ } => {
                // Aquí podrías buscar el icono basado en el app_id
            },
            ext_foreign_toplevel_handle_v1::Event::Closed => {
                items.retain(|i| i.id != id);
            },
            _ => {}
        }
    }
}

impl TrayBackend for WaylandTrayBackend {
    fn get_tray_items(&self) -> Result<Vec<TrayItem>, Box<dyn std::error::Error>> {
        Ok(self.items.lock().unwrap().clone())
    }

    fn setup_monitoring(&self, app_handle: tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        let connection = self.connection.clone();
            
        std::thread::spawn(move || {
            let mut event_queue = connection.new_event_queue();
            loop {
                if let Err(e) = event_queue.blocking_dispatch(&mut ()) {
                    eprintln!("Error en el dispatch de Wayland: {}", e);
                    break;
                }
                
                let _ = app_handle.emit("tray-update", ());
            }
        });

        Ok(())
    }
} 