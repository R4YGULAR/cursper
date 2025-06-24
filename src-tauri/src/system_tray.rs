use tauri::{AppHandle, Manager, menu::{Menu, MenuItem, PredefinedMenuItem}, tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent}};
use crate::backend::start_backend;

// Test if global shortcuts and accessibility are working
#[tauri::command]
pub async fn test_global_shortcut_system() -> Result<(), String> {
    println!("🧪 Testing global shortcut system...");
    
    #[cfg(target_os = "macos")]
    {
        println!("🍎 Checking macOS accessibility permissions...");
        
        // Test AppleScript access first
        let test_script = "tell application \"System Events\" to return \"test\"";
        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg(test_script)
            .output()
            .map_err(|e| format!("Failed to test AppleScript: {}", e))?;
        
        if output.status.success() {
            println!("✅ AppleScript access working");
        } else {
            println!("⚠️ AppleScript access may be restricted");
            println!("📋 AppleScript stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        // Test mouse position access
        let mouse_test = "tell application \"System Events\" to return (get position of mouse cursor)";
        let mouse_output = std::process::Command::new("osascript")
            .arg("-e")
            .arg(mouse_test)
            .output()
            .map_err(|e| format!("Failed to test mouse position: {}", e))?;
        
        if mouse_output.status.success() {
            println!("✅ Mouse position access working: {}", String::from_utf8_lossy(&mouse_output.stdout).trim());
        } else {
            println!("⚠️ Mouse position access may be restricted");
            println!("📋 Mouse test stderr: {}", String::from_utf8_lossy(&mouse_output.stderr));
        }
    }
    
    println!("✅ Global shortcut system test completed");
    Ok(())
}

// Open settings window from tray
#[tauri::command]
pub async fn open_settings(app_handle: AppHandle) -> Result<(), String> {
    show_settings_window(&app_handle)
}

// Create system tray with menu
pub fn setup_system_tray(app: &AppHandle) -> Result<(), tauri::Error> {
    let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let start_backend_i = MenuItem::with_id(app, "start_backend", "Start Backend", true, None::<&str>)?;
    let test_recording_i = MenuItem::with_id(app, "test_recording", "Test Recording", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    
    let menu = Menu::with_items(app, &[
        &settings_i,
        &PredefinedMenuItem::separator(app)?,
        &start_backend_i,
        &test_recording_i,
        &PredefinedMenuItem::separator(app)?,
        &quit_i,
    ])?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            match event.id().as_ref() {
                "settings" => {
                    println!("⚙️ Settings clicked from tray menu");
                    if let Err(e) = show_settings_window(app) {
                        println!("❌ Failed to show settings window: {}", e);
                    }
                }
                "start_backend" => {
                    println!("🐍 Start backend clicked from tray menu");
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = start_backend().await {
                            println!("❌ Failed to start backend: {}", e);
                        }
                    });
                }
                "test_recording" => {
                    println!("🎤 Test recording clicked from tray menu");
                    // We'll need to access state here properly later
                }
                "quit" => {
                    println!("🚪 Quit clicked from tray menu");
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } => {
                    println!("🖱️ Tray icon left clicked");
                    let app = tray.app_handle();
                    if let Err(e) = show_settings_window(app) {
                        println!("❌ Failed to show settings window: {}", e);
                    }
                }
                TrayIconEvent::DoubleClick { .. } => {
                    println!("🖱️ Tray icon double clicked");
                    let app = tray.app_handle();
                    if let Err(e) = show_settings_window(app) {
                        println!("❌ Failed to show settings window: {}", e);
                    }
                }
                _ => {}
            }
        })
        .build(app)?;

    println!("✅ System tray created successfully");
    Ok(())
}

// Show settings window
pub fn show_settings_window(app: &AppHandle) -> Result<(), String> {
    println!("⚙️ show_settings_window called");
    
    match app.get_webview_window("main") {
        Some(window) => {
            println!("📋 Found existing main window");
            window.show().map_err(|e| {
                let error = format!("Failed to show existing window: {}", e);
                println!("❌ {}", error);
                error
            })?;
            window.set_focus().map_err(|e| {
                let error = format!("Failed to focus window: {}", e);
                println!("❌ {}", error);
                error
            })?;
            println!("✅ Existing main window shown and focused");
        }
        None => {
            println!("🔍 No existing main window found, would need to create one");
            // In a real implementation, you might want to create a new window here
            return Err("Main window not found and window creation not implemented".to_string());
        }
    }
    
    Ok(())
} 