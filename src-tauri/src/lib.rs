// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

// Module declarations
mod types;
mod window_manager;
mod audio;
mod text_input;
mod shortcuts;
mod backend;
mod system_tray;

// Re-export commonly used items
use types::AppState;
use shortcuts::setup_shortcuts;
use system_tray::setup_system_tray;

// Import required traits and types
use std::sync::{Arc, Mutex};

pub fn run() {
    println!("🚀 CURSPER TAURI APP STARTING");
    println!("🔧 Creating app state...");
    let state = Arc::new(Mutex::new(AppState::default()));
    
    println!("🏗️ Building Tauri app...");
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_os::init())
        .manage(state.clone())
        .invoke_handler(tauri::generate_handler![
            window_manager::get_cursor_position,
            window_manager::show_overlay,
            audio::start_recording,
            audio::stop_recording_and_transcribe,
            text_input::type_text,
            backend::set_whisper_model,
            backend::get_available_models,
            backend::start_backend,
            shortcuts::toggle_recording,
            shortcuts::update_global_shortcut,
            shortcuts::emit_recording_state,
            system_tray::test_global_shortcut_system,
            system_tray::open_settings
        ])
        .setup(move |app| {
            println!("🔧 Setting up application...");
            
            // Setup system tray
            println!("🔄 Setting up system tray...");
            if let Err(e) = setup_system_tray(app.handle()) {
                println!("❌ Failed to setup system tray: {}", e);
            } else {
                println!("✅ System tray setup completed");
            }
            
            // Setup global shortcuts
            println!("🔄 Setting up global shortcuts...");
            if let Err(e) = setup_shortcuts(app.handle(), state.clone()) {
                println!("❌ Failed to setup shortcuts: {}", e);
            } else {
                println!("✅ Global shortcuts setup completed");
            }
            
            println!("🎉 Application setup completed successfully");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
