fn main() {
    tauri_build::build();
    
    // For now, we'll manually include the protocol in our Rust code
    // In a production environment, you'd want to use wayland-scanner properly
}
