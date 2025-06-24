use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, GlobalShortcutExt};
use crate::types::{AppStateType, get_recording_control};
use crate::window_manager::show_overlay;
use crate::audio::stop_recording_and_transcribe_internal;
use crate::text_input::type_text;
use std::time::Duration;
use tokio;

// Add a new command to emit recording state changes
#[tauri::command]
pub async fn emit_recording_state(app_handle: AppHandle, is_recording: bool) -> Result<(), String> {
    println!("📡 Emitting recording state: {}", is_recording);
    
    app_handle
        .emit("recording-state-changed", is_recording)
        .map_err(|e| format!("Failed to emit recording state: {}", e))?;
    
    Ok(())
}

// Toggle recording state
#[tauri::command]
pub async fn toggle_recording(
    app_handle: AppHandle, 
    state: tauri::State<'_, AppStateType>
) -> Result<(), String> {
    let is_recording = {
        let app_state = state.lock().map_err(|e| e.to_string())?;
        app_state.is_recording
    };
    
    if is_recording {
        // Stop recording and transcribe
        match crate::audio::stop_recording_and_transcribe(state.clone()).await {
            Ok(text) => {
                // Emit recording state change
                let _ = emit_recording_state(app_handle.clone(), false).await;
                
                // Hide overlay
                let _ = show_overlay(app_handle.clone(), false).await;
                
                // Type the transcribed text
                let _ = type_text(text).await;
            }
            Err(e) => println!("Transcription error: {}", e),
        }
    } else {
        // Start recording
        let _ = crate::audio::start_recording(state.clone()).await;
        
        // Emit recording state change
        let _ = emit_recording_state(app_handle.clone(), true).await;
        
        // Show overlay
        let _ = show_overlay(app_handle.clone(), true).await;
    }
    
    Ok(())
}

// Update global shortcut
#[tauri::command]
pub async fn update_global_shortcut(app_handle: AppHandle, shortcut: String, state: tauri::State<'_, AppStateType>) -> Result<(), String> {
    println!("🔄 Updating global shortcut to: {}", shortcut);
    
    // First unregister existing shortcuts (we'll need to track this properly)
    // For now, we'll just try to register the new one
    
    // Update state
    {
        let mut app_state = state.lock().map_err(|e| e.to_string())?;
        app_state.current_shortcut = shortcut.clone();
    }
    
    // Re-setup shortcuts with new shortcut
    setup_shortcuts(&app_handle, state.inner().clone())?;
    
    println!("✅ Global shortcut updated successfully");
    Ok(())
}

// Parse shortcut string into Shortcut struct
pub fn parse_shortcut(shortcut_str: &str) -> Result<Shortcut, String> {
    println!("🔍 Parsing shortcut: '{}'", shortcut_str);
    
    let parts: Vec<&str> = shortcut_str.split('+').collect();
    let mut modifiers = Modifiers::empty();
    let mut key_code: Option<Code> = None;
    
    for part in parts {
        let trimmed_part = part.trim();
        println!("🔍 Processing shortcut part: '{}'", trimmed_part);
        
        match trimmed_part {
            "Ctrl" | "Control" => {
                modifiers.insert(Modifiers::CONTROL);
                println!("✅ Added CONTROL modifier");
            },
            "Alt" | "Option" => {
                modifiers.insert(Modifiers::ALT);
                println!("✅ Added ALT modifier");
            },
            "Shift" => {
                modifiers.insert(Modifiers::SHIFT);
                println!("✅ Added SHIFT modifier");
            },
            "Cmd" | "Command" | "Meta" => {
                modifiers.insert(Modifiers::META);
                println!("✅ Added META modifier");
            },
            "Space" => key_code = Some(Code::Space),
            "Enter" => key_code = Some(Code::Enter),
            "Tab" => key_code = Some(Code::Tab),
            "Escape" => key_code = Some(Code::Escape),
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

// Register global shortcuts with proper event handling
pub fn setup_shortcuts(app: &AppHandle, state: AppStateType) -> Result<(), String> {
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
                
                // Emit recording state change
                let _ = emit_recording_state(app_handle_clone.clone(), false).await;
                
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
                
                // Emit recording state change
                let _ = emit_recording_state(app_handle_clone.clone(), true).await;
                
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