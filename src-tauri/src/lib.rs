// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, GlobalShortcutExt};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::process::Command;
use reqwest;
use tokio;
use tauri_plugin_os;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// Add platform-specific text typing
#[cfg(target_os = "macos")]
use std::process::Command as SystemCommand;

#[cfg(target_os = "windows")]
use std::process::Command as SystemCommand;

// Global recording control
static RECORDING_CONTROL: std::sync::OnceLock<Arc<Mutex<bool>>> = std::sync::OnceLock::new();

fn get_recording_control() -> Arc<Mutex<bool>> {
    RECORDING_CONTROL.get_or_init(|| Arc::new(Mutex::new(false))).clone()
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    x: i32,
    y: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AppState {
    is_recording: bool,
    current_model: String,
    current_shortcut: String,
    shortcuts: HashMap<String, String>,
    backend_url: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            is_recording: false,
            current_model: "base".to_string(),
            current_shortcut: "Option+Space".to_string(),
            shortcuts: HashMap::new(),
            backend_url: "http://127.0.0.1:8788".to_string(),
        }
    }
}

type AppStateType = Arc<Mutex<AppState>>;

// Get cursor position using platform-specific APIs
#[tauri::command]
async fn get_cursor_position() -> Result<CursorPosition, String> {
    println!("📍 get_cursor_position called");
    
    #[cfg(target_os = "macos")]
    {
        println!("🍎 Getting cursor position on macOS using NSEvent");
        // Use AppleScript to get actual cursor position on macOS
        let _script = r#"
            tell application "System Events"
                set mousePos to (do shell script "echo $(osascript -e 'tell application \"System Events\" to return (get position of mouse cursor)')")
                return mousePos
            end tell
        "#;
        
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
async fn show_overlay(app_handle: AppHandle, show: bool) -> Result<(), String> {
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
        
        overlay_window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
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

// Start recording audio with platform-specific tools
#[tauri::command]
async fn start_recording(
    state: tauri::State<'_, AppStateType>
) -> Result<(), String> {
    println!("🎤 Starting audio recording...");
    
    // Update app state
    {
        let mut app_state = state.lock().map_err(|e| e.to_string())?;
        app_state.is_recording = true;
    }
    
    println!("✅ Recording state updated");
    Ok(())
}

// Stop recording and transcribe with REAL Python backend
#[tauri::command]
async fn stop_recording_and_transcribe(
    state: tauri::State<'_, AppStateType>
) -> Result<String, String> {
    println!("🛑 Stopping audio recording...");
    
    let backend_url = {
        let mut app_state = state.lock().map_err(|e| e.to_string())?;
        app_state.is_recording = false;
        app_state.backend_url.clone()
    };
    
    stop_recording_and_transcribe_internal(backend_url).await
}

// Internal function for transcription that can be called from shortcut handler
async fn stop_recording_and_transcribe_internal(backend_url: String) -> Result<String, String> {
    println!("🎤 stop_recording_and_transcribe_internal called");
    println!("🌐 Backend URL: {}", backend_url);
    
    // Test backend connection first
    println!("🧪 Testing backend connection...");
    let client = reqwest::Client::new();
    match client.get(&format!("{}/health", backend_url)).send().await {
        Ok(response) => {
            if response.status().is_success() {
                println!("✅ Backend is responding");
                match response.text().await {
                    Ok(health_text) => println!("📋 Backend health: {}", health_text),
                    Err(e) => println!("⚠️ Could not read health response: {}", e)
                }
            } else {
                println!("⚠️ Backend responded with status: {}", response.status());
                return Err(format!("Backend unhealthy: {}", response.status()));
            }
        },
        Err(e) => {
            println!("❌ Backend connection failed: {}", e);
            return Err(format!("Backend not available: {}", e));
        }
    }
    
    // Record audio using CPAL
    println!("🎙️ Starting audio recording with CPAL...");
    let audio_data = record_audio_cpal().await?;
    
    println!("📤 Sending {} bytes to Python backend...", audio_data.len());
    
    // Send to Python backend
    let response = client
        .post(&format!("{}/transcribe_raw", backend_url))
        .header("Content-Type", "application/octet-stream")
        .body(audio_data)
        .send()
        .await
        .map_err(|e| format!("Failed to send audio to backend: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Backend returned error {}: {}", status, error_text));
    }
    
    // Parse the response
    let transcription_result: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse backend response: {}", e))?;
    
    let transcribed_text = transcription_result
        .get("text")
        .and_then(|t| t.as_str())
        .unwrap_or("No text returned")
        .to_string();
    
    println!("✅ Transcription received: '{}'", transcribed_text);
    Ok(transcribed_text)
}

// Record audio using CPAL (Cross-Platform Audio Library)
async fn record_audio_cpal() -> Result<Vec<u8>, String> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use std::sync::{Arc, Mutex};
    
    println!("🎤 Initializing CPAL audio recording...");
    
    // Get the default audio host and input device
    let host = cpal::default_host();
    let device = host.default_input_device()
        .ok_or("No input device available")?;
    
    println!("🎤 Using audio device: {}", device.name().unwrap_or("Unknown".to_string()));
    
    let config = device.default_input_config()
        .map_err(|e| format!("Failed to get default input config: {}", e))?;
    
    let sample_rate = config.sample_rate().0;
    let channels = config.channels();
    let sample_format = config.sample_format();
    
    println!("🎤 Audio config: {} Hz, {} channels", sample_rate, channels);
    
    // Create a channel to collect audio data
    let (tx, rx) = mpsc::channel::<Vec<f32>>();
    let tx = Arc::new(Mutex::new(tx));
    
    // Create the audio stream
    let stream = match sample_format {
        cpal::SampleFormat::F32 => {
            let tx_clone = tx.clone();
            device.build_input_stream(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if let Ok(sender) = tx_clone.lock() {
                        let _ = sender.send(data.to_vec());
                    }
                },
                |err| eprintln!("❌ Audio stream error: {}", err),
                None,
            )
        },
        cpal::SampleFormat::I16 => {
            let tx_clone = tx.clone();
            device.build_input_stream(
                &config.into(),
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let f32_data: Vec<f32> = data.iter().map(|&sample| sample as f32 / i16::MAX as f32).collect();
                    if let Ok(sender) = tx_clone.lock() {
                        let _ = sender.send(f32_data);
                    }
                },
                |err| eprintln!("❌ Audio stream error: {}", err),
                None,
            )
        },
        _ => return Err("Unsupported sample format".to_string()),
    }.map_err(|e| format!("Failed to build input stream: {}", e))?;
    
    // Start recording
    println!("🎤 Starting audio recording... (will record until stopped or max 30 seconds)");
    stream.play().map_err(|e| format!("Failed to start audio stream: {}", e))?;
    
    // Collect audio data until recording is stopped or max duration reached
    let mut all_audio_data = Vec::new();
    let start_time = std::time::Instant::now();
    let max_recording_duration = Duration::from_secs(30); // Maximum 30 seconds to prevent infinite recording
    
    // Get the global recording control
    let recording_control = get_recording_control();
    
    // Set recording state to true at the start
    {
        let mut should_record = recording_control.lock().unwrap();
        *should_record = true;
    }
    
    let recording_check_interval = Duration::from_millis(50); // Check more frequently
    
    while start_time.elapsed() < max_recording_duration {
        // Check if we should stop recording
        {
            let should_record = recording_control.lock().unwrap();
            if !*should_record {
                println!("🛑 Recording stopped by user input");
                break;
            }
        }
        
        match rx.try_recv() {
            Ok(data) => {
                all_audio_data.extend(data);
            },
            Err(mpsc::TryRecvError::Empty) => {
                thread::sleep(recording_check_interval);
            },
            Err(mpsc::TryRecvError::Disconnected) => {
                break;
            }
        }
    }
    
    // Stop the stream
    drop(stream);
    
    let recording_time = start_time.elapsed();
    println!("🎤 Audio recording completed. Recorded for {:.2} seconds, collected {} samples", 
             recording_time.as_secs_f64(), all_audio_data.len());
    
    if all_audio_data.is_empty() {
        return Err("No audio data recorded".to_string());
    }
    
    // Convert to WAV format
    let wav_data = convert_to_wav(&all_audio_data, sample_rate, channels)?;
    println!("🎵 Converted to WAV format: {} bytes", wav_data.len());
    
    Ok(wav_data)
}

// Convert audio samples to WAV format
fn convert_to_wav(samples: &[f32], sample_rate: u32, channels: u16) -> Result<Vec<u8>, String> {
    use std::io::Cursor;
    use hound::{WavWriter, WavSpec};
    
    let spec = WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut cursor = Cursor::new(Vec::new());
    let mut writer = WavWriter::new(&mut cursor, spec)
        .map_err(|e| format!("Failed to create WAV writer: {}", e))?;
    
    // Convert f32 samples to i16 and write
    for &sample in samples {
        let sample_i16 = (sample * i16::MAX as f32) as i16;
        writer.write_sample(sample_i16)
            .map_err(|e| format!("Failed to write sample: {}", e))?;
    }
    
    writer.finalize()
        .map_err(|e| format!("Failed to finalize WAV: {}", e))?;
    
    Ok(cursor.into_inner())
}

// Type text at cursor position using platform-specific APIs
#[tauri::command]
async fn type_text(text: String) -> Result<(), String> {
    println!("⌨️  type_text called with: '{}'", text);
    
    #[cfg(target_os = "macos")]
    {
        println!("🍎 Using macOS AppleScript to type text");
        // Use AppleScript to type text on macOS
        let escaped_text = text.replace("\\", "\\\\").replace("\"", "\\\"");
        let script = format!(r#"tell application "System Events" to keystroke "{}""#, escaped_text);
        
        println!("📜 AppleScript command: {}", script);
        
        let output = SystemCommand::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| {
                let error_msg = format!("Failed to execute AppleScript: {}", e);
                println!("❌ {}", error_msg);
                error_msg
            })?;
        
        println!("📤 AppleScript exit status: {}", output.status);
        println!("📤 AppleScript stdout: {}", String::from_utf8_lossy(&output.stdout));
        
        if !output.status.success() {
            let error_msg = format!("AppleScript failed: {}", String::from_utf8_lossy(&output.stderr));
            println!("❌ {}", error_msg);
            return Err(error_msg);
        } else {
            println!("✅ AppleScript executed successfully");
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        println!("🪟 Using Windows PowerShell to type text");
        // Use PowerShell to type text on Windows
        let escaped_text = text.replace("'", "''");
        let script = format!(r#"Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait('{}')"#, escaped_text);
        
        println!("📜 PowerShell command: {}", script);
        
        let output = SystemCommand::new("powershell")
            .arg("-Command")
            .arg(&script)
            .output()
            .map_err(|e| {
                let error_msg = format!("Failed to execute PowerShell: {}", e);
                println!("❌ {}", error_msg);
                error_msg
            })?;
        
        println!("📤 PowerShell exit status: {}", output.status);
        
        if !output.status.success() {
            let error_msg = format!("PowerShell failed: {}", String::from_utf8_lossy(&output.stderr));
            println!("❌ {}", error_msg);
            return Err(error_msg);
        } else {
            println!("✅ PowerShell executed successfully");
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        println!("🐧 Using Linux xdotool to type text");
        // Use xdotool on Linux (requires xdotool to be installed)
        let output = SystemCommand::new("xdotool")
            .arg("type")
            .arg(&text)
            .output()
            .map_err(|e| {
                let error_msg = format!("Failed to execute xdotool: {}", e);
                println!("❌ {}", error_msg);
                error_msg
            })?;
        
        println!("📤 xdotool exit status: {}", output.status);
        
        if !output.status.success() {
            let error_msg = format!("xdotool failed: {}", String::from_utf8_lossy(&output.stderr));
            println!("❌ {}", error_msg);
            return Err(error_msg);
        } else {
            println!("✅ xdotool executed successfully");
        }
    }
    
    println!("✅ type_text completed successfully");
    Ok(())
}

// Set Whisper model
#[tauri::command]
async fn set_whisper_model(model: String, state: tauri::State<'_, AppStateType>) -> Result<(), String> {
    let backend_url = {
        let app_state = state.lock().map_err(|e| e.to_string())?;
        app_state.backend_url.clone()
    };
    
    // Send request to Python backend
    let client = reqwest::Client::new();
    
    let mut body = HashMap::new();
    body.insert("model_size", model.clone());
    
    match client.post(&format!("{}/set_model", backend_url))
        .json(&body)
        .send()
        .await 
    {
        Ok(response) => {
            if response.status().is_success() {
                let mut app_state = state.lock().map_err(|e| e.to_string())?;
                app_state.current_model = model;
                Ok(())
            } else {
                Err("Failed to set model on backend".to_string())
            }
        }
        Err(e) => Err(format!("Backend connection error: {}", e))
    }
}

// Get available models
#[tauri::command]
async fn get_available_models(state: tauri::State<'_, AppStateType>) -> Result<Vec<String>, String> {
    let backend_url = {
        let app_state = state.lock().map_err(|e| e.to_string())?;
        app_state.backend_url.clone()
    };
    
    let client = reqwest::Client::new();
    
    match client.get(&format!("{}/models", backend_url)).send().await {
        Ok(response) => {
            if response.status().is_success() {
                Ok(vec!["tiny".to_string(), "base".to_string(), "small".to_string(), "medium".to_string(), "large".to_string()])
            } else {
                Err("Failed to get models from backend".to_string())
            }
        }
        Err(_) => Err("Backend not available".to_string())
    }
}

// Start Python backend
#[tauri::command]
async fn start_backend() -> Result<(), String> {
    tokio::spawn(async {
        let output = Command::new("python3")
            .arg("../python/app.py")
            .arg("--port")
            .arg("8788")
            .spawn();
            
        match output {
            Ok(_) => println!("Backend started successfully"),
            Err(e) => println!("Failed to start backend: {}", e),
        }
    });
    
    Ok(())
}

// Update global shortcut
#[tauri::command]
async fn update_global_shortcut(app_handle: AppHandle, shortcut: String, state: tauri::State<'_, AppStateType>) -> Result<(), String> {
    // Parse shortcut string and register new shortcut
    let parsed_shortcut = parse_shortcut(&shortcut)?;
    
    // Unregister old shortcut first
    let old_shortcut = {
        let app_state = state.lock().map_err(|e| e.to_string())?;
        app_state.current_shortcut.clone()
    };
    
    if let Ok(old_parsed) = parse_shortcut(&old_shortcut) {
        let _ = app_handle.global_shortcut().unregister(old_parsed);
    }
    
    // Register new shortcut
    app_handle.global_shortcut().register(parsed_shortcut).map_err(|e| e.to_string())?;
    
    // Update state
    let mut app_state = state.lock().map_err(|e| e.to_string())?;
    app_state.current_shortcut = shortcut;
    
    println!("Global shortcut updated");
    Ok(())
}

// Parse shortcut string into Shortcut struct
fn parse_shortcut(shortcut_str: &str) -> Result<Shortcut, String> {
    println!("🔤 parse_shortcut called with: '{}'", shortcut_str);
    
    let parts: Vec<&str> = shortcut_str.split('+').collect();
    if parts.is_empty() {
        let error = "Invalid shortcut format".to_string();
        println!("❌ {}", error);
        return Err(error);
    }
    
    println!("📝 Shortcut parts: {:?}", parts);
    
    let mut modifiers = Modifiers::empty();
    let mut key_code = None;
    
    for part in parts {
        let trimmed_part = part.trim();
        println!("🔍 Processing part: '{}'", trimmed_part);
        
        match trimmed_part {
            "Cmd" | "CmdOrCtrl" => {
                #[cfg(target_os = "macos")]
                {
                    modifiers |= Modifiers::META; // Use META for Cmd on macOS
                    println!("✅ Added META modifier (Cmd on macOS)");
                }
                #[cfg(not(target_os = "macos"))]
                {
                    modifiers |= Modifiers::CONTROL; // Use CONTROL for Ctrl on other platforms
                    println!("✅ Added CONTROL modifier (Ctrl on non-macOS)");
                }
            },
            "Ctrl" => {
                modifiers |= Modifiers::CONTROL;
                println!("✅ Added CONTROL modifier");
            },
            "Shift" => {
                modifiers |= Modifiers::SHIFT;
                println!("✅ Added SHIFT modifier");
            },
            "Alt" | "Option" | "RightAlt" | "RAlt" | "LeftAlt" | "LAlt" | "RightOption" | "ROption" | "LeftOption" | "LOption" => {
                modifiers |= Modifiers::ALT;
                println!("✅ Added ALT (Option) modifier via {}", trimmed_part);
            },
            "Space" => {
                key_code = Some(Code::Space);
                println!("✅ Set key code to Space");
            },
            "F1" => key_code = Some(Code::F1),
            "F2" => key_code = Some(Code::F2),
            "F3" => key_code = Some(Code::F3),
            "F4" => key_code = Some(Code::F4),
            "F5" => key_code = Some(Code::F5),
            "F6" => key_code = Some(Code::F6),
            "F7" => key_code = Some(Code::F7),
            "F8" => key_code = Some(Code::F8),
            "F9" => key_code = Some(Code::F9),
            "F10" => key_code = Some(Code::F10),
            "F11" => key_code = Some(Code::F11),
            "F12" => key_code = Some(Code::F12),
            "A" => key_code = Some(Code::KeyA),
            "B" => key_code = Some(Code::KeyB),
            "C" => key_code = Some(Code::KeyC),
            "V" => key_code = Some(Code::KeyV),
            _ => {
                let error = format!("Unknown key: {}", trimmed_part);
                println!("❌ {}", error);
                return Err(error);
            }
        }
    }
    
    match key_code {
        Some(code) => {
            println!("✅ Shortcut parsed successfully - Modifiers: {:?}, Key: {:?}", modifiers, code);
            // If no modifiers are set, pass None instead of empty modifiers
            let modifier_option = if modifiers.is_empty() { 
                None 
            } else { 
                Some(modifiers) 
            };
            Ok(Shortcut::new(modifier_option, code))
        },
        None => {
            let error = "No key code found in shortcut".to_string();
            println!("❌ {}", error);
            Err(error)
        }
    }
}

// Toggle recording state
#[tauri::command]
async fn toggle_recording(
    app_handle: AppHandle, 
    state: tauri::State<'_, AppStateType>
) -> Result<(), String> {
    let is_recording = {
        let app_state = state.lock().map_err(|e| e.to_string())?;
        app_state.is_recording
    };
    
    if is_recording {
        // Stop recording and transcribe
        match stop_recording_and_transcribe(state.clone()).await {
            Ok(text) => {
                // Hide overlay
                let _ = show_overlay(app_handle.clone(), false).await;
                
                // Type the transcribed text
                let _ = type_text(text).await;
            }
            Err(e) => println!("Transcription error: {}", e),
        }
    } else {
        // Start recording
        let _ = start_recording(state.clone()).await;
        
        // Show overlay
        let _ = show_overlay(app_handle.clone(), true).await;
    }
    
    Ok(())
}

// Register global shortcuts with proper event handling
fn setup_shortcuts(app: &AppHandle, state: AppStateType) -> Result<(), String> {
    println!("🎛️  setup_shortcuts called");
    
    // Get initial shortcut from state
    let shortcut_str = {
        let app_state = state.lock().map_err(|e| {
            let error = format!("Failed to lock state for shortcut setup: {}", e);
            println!("❌ {}", error);
            error
        })?;
        app_state.current_shortcut.clone()
    };
    
    println!("⌨️  Setting up global shortcut: {}", shortcut_str);
    
    // Parse and register the shortcut
    let shortcut = parse_shortcut(&shortcut_str)?;
    println!("✅ Shortcut parsed successfully");
    
    // Clone necessary variables for the closure
    let app_handle = app.clone();
    let state_clone = state.clone();
    
    println!("🔗 Registering shortcut event handler...");
    app.global_shortcut().on_shortcut(shortcut, move |_app, _event, _monitor| {
        let app_handle_clone = app_handle.clone();
        let state_clone = state_clone.clone();
        
        println!("🎯 GLOBAL SHORTCUT TRIGGERED! Option+Space pressed");
        
        // Handle shortcut press in async context
        tauri::async_runtime::spawn(async move {
            println!("🔄 Starting async shortcut handler...");
            
            // Handle the recording toggle directly without the State wrapper
            let is_recording = {
                let app_state = state_clone.lock().map_err(|e| e.to_string());
                match app_state {
                    Ok(state) => {
                        println!("📊 Current recording state: {}", state.is_recording);
                        state.is_recording
                    },
                    Err(e) => {
                        println!("❌ Failed to lock app state: {}", e);
                        return;
                    }
                }
            };
            
            if is_recording {
                println!("🛑 STOPPING RECORDING...");
                
                // Signal the recording to stop
                {
                    let recording_control = get_recording_control();
                    let mut should_record = recording_control.lock().unwrap();
                    *should_record = false;
                    println!("✅ Recording control signal set to false");
                }
                
                // Update app state
                let backend_url = {
                    let mut app_state = state_clone.lock().unwrap();
                    app_state.is_recording = false;
                    println!("✅ App recording state set to false");
                    app_state.backend_url.clone()
                };
                
                // Give a moment for the recording to stop gracefully
                tokio::time::sleep(Duration::from_millis(100)).await;
                
                // Call actual transcription function
                println!("🎤 Starting transcription process...");
                let transcription_result = stop_recording_and_transcribe_internal(backend_url).await;
                
                let transcribed_text = match transcription_result {
                    Ok(text) => {
                        println!("✅ Transcription successful: '{}'", text);
                        text
                    },
                    Err(e) => {
                        println!("❌ Transcription failed: {}", e);
                        println!("🔄 Using fallback text");
                        "Transcription failed".to_string()
                    }
                };
                
                // Hide overlay and type text
                println!("🔒 Hiding overlay...");
                match show_overlay(app_handle_clone.clone(), false).await {
                    Ok(_) => println!("✅ Overlay hidden successfully"),
                    Err(e) => println!("❌ Failed to hide overlay: {}", e),
                }
                
                // Only type text if it's not empty and not an error message
                if !transcribed_text.trim().is_empty() && !transcribed_text.contains("failed") {
                    println!("⌨️  Starting to type text...");
                    match type_text(transcribed_text.clone()).await {
                        Ok(_) => println!("✅ Text typed successfully: '{}'", transcribed_text),
                        Err(e) => println!("❌ Failed to type text: {}", e),
                    }
                } else {
                    println!("⚠️ Skipping text typing due to empty or error transcription");
                }
            } else {
                println!("🎙️ STARTING RECORDING...");
                
                // Start recording
                {
                    let mut app_state = state_clone.lock().unwrap();
                    app_state.is_recording = true;
                    println!("✅ App recording state set to true");
                }
                
                // Reset recording control to allow new recording
                {
                    let recording_control = get_recording_control();
                    let mut should_record = recording_control.lock().unwrap();
                    *should_record = true;
                    println!("✅ Recording control signal set to true");
                }
                
                // Show overlay
                println!("👁️ Showing overlay...");
                match show_overlay(app_handle_clone.clone(), true).await {
                    Ok(_) => println!("✅ Overlay shown successfully"),
                    Err(e) => println!("❌ Failed to show overlay: {}", e),
                }
                
                // Start the actual recording process in a separate task
                let backend_url = {
                    let app_state = state_clone.lock().unwrap();
                    app_state.backend_url.clone()
                };
                
                tokio::spawn(async move {
                    println!("🎤 Starting background recording task...");
                    // This will run until the recording control is set to false
                    let _result = stop_recording_and_transcribe_internal(backend_url).await;
                    println!("🎤 Background recording task completed");
                });
            }
            
            println!("🎉 Shortcut handler completed successfully");
        });
    }).map_err(|e| {
        let error = format!("Failed to register shortcut event handler: {}", e);
        println!("❌ {}", error);
        error
    })?;
    
    println!("📝 Registering shortcut with system...");
    // Actually register the shortcut
    app.global_shortcut().register(shortcut).map_err(|e| {
        let error = format!("Failed to register shortcut with system: {}", e);
        println!("❌ {}", error);
        error
    })?;
    
    println!("✅ Global shortcut '{}' registered successfully", shortcut_str);
    Ok(())
}

// Test if global shortcuts and accessibility are working
#[tauri::command]
async fn test_global_shortcut_system() -> Result<(), String> {
    println!("🧪 Testing global shortcut system...");
    
    #[cfg(target_os = "macos")]
    {
        println!("🍎 Checking macOS accessibility permissions...");
        
        // Test AppleScript access first
        let test_script = "tell application \"System Events\" to return \"test\"";
        let output = SystemCommand::new("osascript")
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
        let mouse_output = SystemCommand::new("osascript")
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
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
            get_cursor_position,
            show_overlay,
            start_recording,
            stop_recording_and_transcribe,
            type_text,
            set_whisper_model,
            get_available_models,
            start_backend,
            toggle_recording,
            update_global_shortcut,
            test_global_shortcut_system
        ])
        .setup(move |app| {
            println!("⚙️ TAURI SETUP STARTING");
            
            // Test global shortcut system first
            println!("🧪 Testing global shortcut system...");
            tauri::async_runtime::spawn(async {
                match test_global_shortcut_system().await {
                    Ok(_) => println!("✅ Global shortcut system test passed"),
                    Err(e) => println!("⚠️ Global shortcut system test failed: {}", e),
                }
            });
            
            println!("🎛️ Setting up global shortcuts...");
            
            // Setup global shortcuts
            if let Err(e) = setup_shortcuts(app.handle(), state.clone()) {
                eprintln!("❌ Failed to setup shortcuts: {}", e);
                // Don't fail the entire app if shortcuts fail
            } else {
                println!("✅ Global shortcuts setup completed");
            }
            
            // Start Python backend
            println!("🐍 Starting Python backend...");
            tauri::async_runtime::spawn(async {
                match start_backend().await {
                    Ok(_) => println!("✅ Python backend startup initiated"),
                    Err(e) => println!("❌ Failed to start Python backend: {}", e),
                }
            });
            
            println!("🎉 TAURI SETUP COMPLETED SUCCESSFULLY");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
