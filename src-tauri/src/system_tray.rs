use serde::Serialize;
use serde_json;
use tauri::Emitter;
use zbus::export::futures_util::StreamExt;
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use gdk_pixbuf::{Pixbuf, Colorspace};
use gtk::glib::{Bytes as GlibBytes, Error as GlibError}; // Import Error too


// Asumiendo que notifier_host::Host ahora es pub trait Host: Send { ... }
use notifier_host;

#[derive(Clone, Serialize)]
struct IconData {
    width: i32,
    height: i32,
    data: String, // Base64 encoded PNG data
}

#[derive(Clone, Serialize)]
struct TrayItemData {
    id: String,
    title: String,
    status: String, // "Passive", "Active", "NeedsAttention"
    icon: Option<IconData>, // Enviaremos el icono como datos Base64
    has_menu: bool, // Indicador simple si tiene menú contextual
}

struct DBusSession {
    snw: notifier_host::proxy::StatusNotifierWatcherProxy<'static>,
}

async fn dbus_session() -> zbus::Result<&'static DBusSession> {
    static DBUS_STATE: tokio::sync::OnceCell<DBusSession> = tokio::sync::OnceCell::const_new();
    DBUS_STATE
        .get_or_try_init(|| async {
            let con = zbus::Connection::session().await?;
            notifier_host::Watcher::new().attach_to(&con).await?;
            let (_, snw) = notifier_host::register_as_host(&con).await?;
            Ok(DBusSession { snw })
        })
        .await
}


struct Tray {
    app_handle: tauri::AppHandle,
    item_tasks: std::collections::HashMap<String, tauri::async_runtime::JoinHandle<()>>,
}

pub fn spawn_systray(app_handle: tauri::AppHandle) {
    log::info!("Spawning system tray handler");

    tauri::async_runtime::spawn(async move { // <--- CAMBIO AQUÍ
        let dbus_session = match dbus_session().await {
            Ok(x) => x,
            Err(e) => {
                log::error!("Could not initialise dbus connection for tray: {}", e);
                // Usar app_handle aquí está bien porque se movió al closure
                let _ = app_handle.emit("tray://error", format!("DBus init failed: {}", e));
                return;
            }
        };

        let mut systray = Box::new(Tray {
            app_handle: app_handle.clone(), // Clonar para el struct Tray
            item_tasks: Default::default(),
        });

        // Usar app_handle aquí también está bien
        let _ = app_handle.emit("tray://ready", ());
        log::info!("Starting notifier host loop...");

        let result = notifier_host::run_host(&mut *systray, &dbus_session.snw).await;

        log::error!("Notifier host loop exited: {}", result);
        // Usar app_handle aquí también está bien
        let _ = app_handle.emit("tray://stopped", format!("Host loop error: {}", result));
    });
}


// Asumiendo pub trait Host: Send { ... } en notifier_host
impl notifier_host::Host for Tray {
    fn add_item(&mut self, id: &str, item: notifier_host::Item) {
        log::info!("Adding tray item: {}", id);
        let app_handle = self.app_handle.clone();
        let id_owned = id.to_string();

        if let Some(old_task) = self.item_tasks.remove(id) {
            log::warn!("Replacing existing item task for id: {}", id);
            old_task.abort();
        }

        let listener_task = spawn_item_listener(
            id_owned.clone(),
            item,
            app_handle,
        );
        self.item_tasks.insert(id_owned, listener_task);
    }

    fn remove_item(&mut self, id: &str) {
        log::info!("Removing tray item: {}", id);
        if let Some(task) = self.item_tasks.remove(id) {
            task.abort();
            log::debug!("Aborted listener task for item: {}", id);
        } else {
            log::warn!("Tried to remove nonexistent item task for id: {}", id);
        }
        if let Err(e) = self.app_handle.emit("tray://item-removed", id) {
             log::error!("Failed to emit item-removed event for {}: {}", id, e);
        }
    }
}

fn pixbuf_to_base64_png(pixbuf: &Pixbuf) -> Result<String, String> {
    let buffer = pixbuf.save_to_bufferv("png", &[("compression", "1")])
        .map_err(|e| format!("Failed to save Pixbuf to buffer: {}", e))?;
    Ok(BASE64_STANDARD.encode(&buffer))
}

fn pixmap_data_to_pixbuf(width: i32, height: i32, data: Vec<u8>) -> Result<Pixbuf, String> {
    if width <= 0 || height <= 0 {
        return Err("Invalid pixmap dimensions".to_string());
    }
    let expected_len = (width as usize) * (height as usize) * 4; // ARGB = 4 bytes
    if data.len() != expected_len {
        return Err(format!(
            "Pixmap data size mismatch: expected {} bytes, got {}",
            expected_len,
            data.len()
        ));
    }

    let bytes = GlibBytes::from_owned(data);

    // Sigue la sugerencia del compilador y envuelve la llamada en Ok()
    let creation_result: Result<Pixbuf, GlibError> = Ok(Pixbuf::from_bytes( // <-- Envolver en Ok()
        &bytes,
        Colorspace::Rgb,
        true, // has_alpha
        8,    // bits_per_sample
        width,
        height,
        width * 4, // rowstride
    )); // <-- Cerrar paréntesis de Ok()

    // Match on the explicitly typed variable
    match creation_result {
        // Si from_bytes realmente devolvía Result, ahora tendremos Ok(Ok(pixbuf)) o Ok(Err(e))
        // Si devolvía Pixbuf, tendremos Ok(pixbuf) o pánico si falló.
        // Ajustaremos el match si esto compila o da un error diferente.
        // Por ahora, dejemos el match como estaba para ver el siguiente error.
        Ok(pixbuf) => Ok(pixbuf),
        Err(e) => Err(format!("Failed to create Pixbuf from bytes: {}", e)),
    }
}


// --- Funciones get_icon_pixbuf, get_item_data, spawn_item_listener ---
// (Sin cambios)
async fn get_icon_pixbuf(item: &notifier_host::Item, requested_size: i32) -> zbus::Result<Option<Pixbuf>> {
    let pixmaps = match item.sni.icon_pixmap().await {
        Ok(p) => p,
        Err(e) => {
            log::warn!("Failed to get icon_pixmap: {}", e);
            return Ok(None);
        }
    };

    if pixmaps.is_empty() {
        log::debug!("No pixmaps found via icon_pixmap().");
        return Ok(None);
    }

    let best_pixmap = pixmaps.into_iter()
        .min_by_key(|(w, h, _data)| {
            let width_diff = w - requested_size;
            let height_diff = h - requested_size;
            let penalty = if width_diff < 0 || height_diff < 0 { 10000 } else { 0 };
            width_diff.abs() + height_diff.abs() + penalty
        });


    match best_pixmap {
        Some((w, h, data)) => {
            log::debug!("Selected pixmap size: {}x{}", w, h);
            match pixmap_data_to_pixbuf(w, h, data) {
                Ok(pixbuf) => Ok(Some(pixbuf)),
                Err(e) => {
                    log::error!("Failed to convert selected pixmap data: {}", e);
                    Ok(None)
                }
            }
        }
        None => {
            log::debug!("Could not select a suitable pixmap.");
            Ok(None)
        }
    }
}


async fn get_item_data(
    item: &notifier_host::Item,
    id: &str,
    size: i32,
    _scale: i32,
) -> zbus::Result<TrayItemData> {
    let title = item.sni.title().await.unwrap_or_else(|_| String::from(""));
    let status = item.sni.status().await?;
    let has_menu = item.sni.menu().await.is_ok();

    let icon_pixbuf = get_icon_pixbuf(item, size).await?;

    let icon_data = match icon_pixbuf {
        Some(pixbuf) => {
            match pixbuf_to_base64_png(&pixbuf) {
                Ok(data) => Some(IconData {
                    width: pixbuf.width(),
                    height: pixbuf.height(),
                    data,
                }),
                Err(e) => {
                    log::warn!("[{}] Failed to encode icon pixbuf: {}", id, e);
                    None
                }
            }
        },
        None => {
             log::debug!("[{}] Icon pixbuf not available or conversion failed.", id);
             None
        }
    };

    Ok(TrayItemData {
        id: id.to_string(),
        title,
        status: format!("{:?}", status),
        icon: icon_data,
        has_menu,
    })
}

fn spawn_item_listener(
    id: String,
    item: notifier_host::Item,
    app_handle: tauri::AppHandle,
) -> tauri::async_runtime::JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        // Fetch inicial
        let initial_fetch_app_handle = app_handle.clone();
        let initial_fetch_id = id.clone();
        let scale = 1;
        let size = 32;
        match get_item_data(&item, &initial_fetch_id, size, scale).await {
            Ok(data) => {
                log::debug!("Emitting tray://item-added for {}", initial_fetch_id);
                if let Err(e) = initial_fetch_app_handle.emit("tray://item-added", data) {
                     log::error!("Failed to emit item-added event for {}: {}", initial_fetch_id, e);
                }
            }
            Err(e) => {
                log::error!("Failed to get initial item data for {}: {}", initial_fetch_id, e);
                let _ = initial_fetch_app_handle.emit("tray://item-error", serde_json::json!({
                    "id": initial_fetch_id,
                    "error": format!("Failed to get initial data: {}", e),
                }));
            }
        }

        // Obtener streams
        let mut status_updates = match item.sni.receive_new_status().await {
            Ok(stream) => stream,
            Err(e) => { log::error!("[{}] Failed to get status stream: {}", id, e); return; }
        };
        let mut title_updates = match item.sni.receive_new_title().await {
             Ok(stream) => stream,
             Err(e) => { log::error!("[{}] Failed to get title stream: {}", id, e); return; }
        };
        let mut icon_updates = match item.sni.receive_new_icon().await {
             Ok(stream) => stream,
             Err(e) => { log::error!("[{}] Failed to get icon stream: {}", id, e); return; }
        };

        log::debug!("[{}] Started update listener task", id);

        loop {
            let size = 32;

            tokio::select! {
                maybe_update = status_updates.next() => {
                    match maybe_update {
                        Some(_signal_args) => {
                            match item.sni.status().await {
                                Ok(status) => {
                                    log::debug!("[{}] Status updated: {:?}", id, status);
                                    let _ = app_handle.emit("tray://item-updated", serde_json::json!({
                                        "id": id.clone(),
                                        "status": format!("{:?}", status),
                                    }));
                                }
                                Err(e) => log::warn!("[{}] Failed to get updated status: {}", id, e),
                            }
                        }
                        None => { log::debug!("[{}] Status update stream ended", id); break; }
                    }
                }

                maybe_update = title_updates.next() => {
                     match maybe_update {
                        Some(_signal_args) => {
                            match item.sni.title().await {
                                Ok(title) => {
                                    log::debug!("[{}] Title updated: {}", id, title);
                                    let _ = app_handle.emit("tray://item-updated", serde_json::json!({
                                        "id": id.clone(),
                                        "title": title,
                                    }));
                                }
                                Err(e) => log::warn!("[{}] Failed to get updated title: {}", id, e),
                            }
                        }
                        None => { log::debug!("[{}] Title update stream ended", id); break; }
                    }
                }

                maybe_update = icon_updates.next() => {
                     match maybe_update {
                        Some(_signal_args) => {
                            log::debug!("[{}] Icon updated signal received, fetching pixmap for size {}", id, size);
                            match get_icon_pixbuf(&item, size).await {
                                Ok(Some(pixbuf)) => {
                                    match pixbuf_to_base64_png(&pixbuf) {
                                        Ok(data) => {
                                            let icon_payload = IconData { width: pixbuf.width(), height: pixbuf.height(), data };
                                            let _ = app_handle.emit("tray://item-updated", serde_json::json!({
                                                "id": id.clone(),
                                                "icon": icon_payload,
                                            }));
                                        },
                                        Err(e) => log::warn!("[{}] Failed to encode updated icon: {}", id, e),
                                    }
                                },
                                Ok(None) => {
                                    log::warn!("[{}] Updated icon pixbuf not available or conversion failed.", id);
                                     let _ = app_handle.emit("tray://item-updated", serde_json::json!({
                                         "id": id.clone(),
                                         "icon": null,
                                     }));
                                }
                                Err(e) => {
                                    log::warn!("[{}] Failed to get updated icon pixbuf: {}", id, e);
                                     let _ = app_handle.emit("tray://item-updated", serde_json::json!({
                                         "id": id.clone(),
                                         "icon": null,
                                     }));
                                }
                            }
                        }
                        None => { log::debug!("[{}] Icon update stream ended", id); break; }
                    }
                }
            }
        }
        log::debug!("[{}] Update listener task finished for item {}", id, id);
    })
}
