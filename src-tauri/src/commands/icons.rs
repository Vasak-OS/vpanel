use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use gtk::prelude::IconThemeExt;
use std::fs;

const ICON_SIZE: i32 = 64;
const FALLBACK_ICON: &str = "image-missing";

fn get_themed_icon(
    name: &str, 
    flags: gtk::IconLookupFlags
) -> Option<gtk::IconInfo> {
    let themed = gtk::IconTheme::default()?;
    let icon = themed.lookup_icon(name, ICON_SIZE, flags);

    if icon.is_none() {
        return themed.lookup_icon(FALLBACK_ICON, ICON_SIZE, flags);
    }

    icon
}

fn encode_icon(icon_info: gtk::IconInfo) -> Result<String, String> {
    let icon_path = icon_info
        .filename()
        .ok_or_else(|| "No icon filename".to_string())?;

    let icon_data = fs::read(icon_path)
        .map_err(|e| e.to_string())?;

    Ok(STANDARD.encode(icon_data))
}

#[tauri::command]
pub fn get_icon_base64(name: &str) -> Result<String, String> {
    let flags = gtk::IconLookupFlags::FORCE_SVG | gtk::IconLookupFlags::FORCE_REGULAR;
    let icon_info = get_themed_icon(name, flags)
        .ok_or_else(|| "Icon not found".to_string())?;
    
    encode_icon(icon_info)
}

#[tauri::command]
pub fn get_symbol_base64(name: &str) -> Result<String, String> {
    let flags = gtk::IconLookupFlags::FORCE_SYMBOLIC | gtk::IconLookupFlags::FORCE_SVG;
    let icon_info = get_themed_icon(name, flags)
        .ok_or_else(|| "Symbol not found".to_string())?;
    
    encode_icon(icon_info)
}
