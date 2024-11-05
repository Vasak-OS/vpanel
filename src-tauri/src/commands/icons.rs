use gtk::prelude::IconThemeExt;

#[tauri::command]
pub fn get_icon_path(name: &str) -> String {
    let themed = gtk::IconTheme::default().unwrap();

    let icon = themed
        .lookup_icon(
            name,
            64,
            gtk::IconLookupFlags::FORCE_SVG | gtk::IconLookupFlags::FORCE_REGULAR,
        ).unwrap().filename();

    return icon.unwrap().to_str().unwrap().to_string();
}

#[tauri::command]
pub fn get_symbol_path(name: &str) -> String {
    let themed = gtk::IconTheme::default().unwrap();

    let icon = themed
        .lookup_icon(
            name,
            64,
            gtk::IconLookupFlags::FORCE_SYMBOLIC | gtk::IconLookupFlags::FORCE_SVG,
        ).unwrap().filename();

    return icon.unwrap().to_str().unwrap().to_string();
}