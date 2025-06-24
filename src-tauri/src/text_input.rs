// Type text at cursor position using platform-specific APIs
#[tauri::command]
pub async fn type_text(text: String) -> Result<(), String> {
    println!("⌨️ type_text called with: '{}'", text);
    
    if text.trim().is_empty() {
        println!("⚠️ Empty text provided, skipping typing");
        return Ok(());
    }
    
    #[cfg(target_os = "macos")]
    {
        println!("🍎 Using AppleScript to type text on macOS");
        
        // Escape the text for AppleScript
        let escaped_text = text
            .replace("\\", "\\\\")
            .replace("\"", "\\\"")
            .replace("\n", "\\n")
            .replace("\r", "\\r")
            .replace("\t", "\\t");
        
        let script = format!(
            r#"tell application "System Events" to keystroke "{}""#,
            escaped_text
        );
        
        println!("📝 AppleScript: {}", script);
        
        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| format!("Failed to execute AppleScript: {}", e))?;
        
        if output.status.success() {
            println!("✅ Text typed successfully via AppleScript");
            Ok(())
        } else {
            let error = format!(
                "AppleScript failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            println!("❌ {}", error);
            Err(error)
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        println!("🪟 Using PowerShell to type text on Windows");
        
        // Escape the text for PowerShell
        let escaped_text = text
            .replace("'", "''");
        
        let script = format!(
            r#"Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait('{}')"#,
            escaped_text
        );
        
        let output = std::process::Command::new("powershell")
            .arg("-Command")
            .arg(&script)
            .output()
            .map_err(|e| format!("Failed to execute PowerShell: {}", e))?;
        
        if output.status.success() {
            println!("✅ Text typed successfully via PowerShell");
            Ok(())
        } else {
            let error = format!(
                "PowerShell failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            println!("❌ {}", error);
            Err(error)
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        println!("🐧 Using xdotool to type text on Linux");
        
        let output = std::process::Command::new("xdotool")
            .arg("type")
            .arg("--delay")
            .arg("12") // 12ms delay between keystrokes
            .arg(&text)
            .output()
            .map_err(|e| format!("Failed to execute xdotool: {}", e))?;
        
        if output.status.success() {
            println!("✅ Text typed successfully via xdotool");
            Ok(())
        } else {
            let error = format!(
                "xdotool failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            println!("❌ {}", error);
            Err(error)
        }
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        let error = "Text typing not supported on this platform".to_string();
        println!("❌ {}", error);
        Err(error)
    }
} 