use tauri::{AppHandle, Manager, Position, PhysicalPosition};
use crate::types::CursorPosition;

// Get cursor position using platform-specific APIs
#[tauri::command]
pub async fn get_cursor_position() -> Result<CursorPosition, String> {
    println!("📍 get_cursor_position called");
    
    #[cfg(target_os = "macos")]
    {
        println!("🍎 Getting cursor position on macOS using NSEvent");
        
        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to return (get position of mouse cursor)")
            .output();
            
        match output {
            Ok(result) => {
                let output_str = String::from_utf8_lossy(&result.stdout);
                println!("📍 AppleScript output: '{}'", output_str.trim());
                
                // Parse the output like "123, 456"
                let coords: Vec<&str> = output_str.trim().split(", ").collect();
                if coords.len() == 2 {
                    if let (Ok(x), Ok(y)) = (coords[0].parse::<i32>(), coords[1].parse::<i32>()) {
                        println!("✅ Parsed cursor position: x={}, y={}", x, y);
                        return Ok(CursorPosition { x, y });
                    }
                }
                println!("⚠️ Could not parse cursor position, using default");
            }
            Err(e) => {
                println!("❌ Failed to get cursor position: {}", e);
            }
        }
        
        // Fallback to center of screen
        println!("📍 Using fallback position (center of screen)");
        Ok(CursorPosition { x: 400, y: 300 })
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        println!("📍 Using default position for non-macOS platform");
        // Default position for other platforms
        Ok(CursorPosition { x: 400, y: 300 })
    }
}

// Show/hide overlay window at cursor position
#[tauri::command]
pub async fn show_overlay(app_handle: AppHandle, show: bool) -> Result<(), String> {
    println!("👁️ show_overlay called with show={}", show);
    
    let overlay_window = app_handle.get_webview_window("overlay")
        .ok_or_else(|| {
            let error = "Overlay window not found".to_string();
            println!("❌ {}", error);
            error
        })?;
    
    println!("✅ Overlay window found successfully");
    
    if show {
        println!("📍 Getting cursor position...");
        let cursor_pos = get_cursor_position().await?;
        println!("📍 Cursor position: x={}, y={}", cursor_pos.x, cursor_pos.y);
        
        // Position overlay near cursor
        let new_x = cursor_pos.x + 10;
        let new_y = cursor_pos.y + 10;
        println!("📍 Setting overlay position to: x={}, y={}", new_x, new_y);
        
        overlay_window.set_position(Position::Physical(PhysicalPosition {
            x: new_x,
            y: new_y,
        })).map_err(|e| {
            let error = format!("Failed to set overlay position: {}", e);
            println!("❌ {}", error);
            error
        })?;
        
        println!("✅ Overlay position set successfully");
        
        println!("👁️ Showing overlay window...");
        overlay_window.show().map_err(|e| {
            let error = format!("Failed to show overlay: {}", e);
            println!("❌ {}", error);
            error
        })?;
        
        println!("📌 Setting overlay always on top...");
        overlay_window.set_always_on_top(true).map_err(|e| {
            let error = format!("Failed to set overlay always on top: {}", e);
            println!("❌ {}", error);
            error
        })?;
        
        println!("✅ Overlay shown and set to always on top");
    } else {
        println!("🔒 Hiding overlay window...");
        overlay_window.hide().map_err(|e| {
            let error = format!("Failed to hide overlay: {}", e);
            println!("❌ {}", error);
            error
        })?;
        
        println!("✅ Overlay hidden successfully");
    }
    
    println!("✅ show_overlay completed successfully");
    Ok(())
} 