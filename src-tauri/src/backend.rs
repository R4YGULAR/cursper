use crate::types::AppStateType;

// Set Whisper model
#[tauri::command]
pub async fn set_whisper_model(model: String, state: tauri::State<'_, AppStateType>) -> Result<(), String> {
    println!("🔄 Setting Whisper model to: {}", model);
    
    // Update app state
    {
        let mut app_state = state.lock().map_err(|e| e.to_string())?;
        app_state.current_model = model.clone();
    }
    
    // Send model change to backend
    let backend_url = {
        let app_state = state.lock().map_err(|e| e.to_string())?;
        app_state.backend_url.clone()
    };
    
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/set_model", backend_url))
        .json(&serde_json::json!({ "model": model }))
        .send()
        .await
        .map_err(|e| format!("Failed to send model change to backend: {}", e))?;
    
    if response.status().is_success() {
        println!("✅ Model set successfully");
        Ok(())
    } else {
        let error = format!("Backend returned error: {}", response.status());
        println!("❌ {}", error);
        Err(error)
    }
}

// Get available models
#[tauri::command]
pub async fn get_available_models(state: tauri::State<'_, AppStateType>) -> Result<Vec<String>, String> {
    println!("📋 Getting available models...");
    
    let backend_url = {
        let app_state = state.lock().map_err(|e| e.to_string())?;
        app_state.backend_url.clone()
    };
    
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/models", backend_url))
        .send()
        .await
        .map_err(|e| format!("Failed to get models from backend: {}", e))?;
    
    if response.status().is_success() {
        let models: Vec<String> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse models response: {}", e))?;
        
        println!("✅ Available models: {:?}", models);
        Ok(models)
    } else {
        let error = format!("Backend returned error: {}", response.status());
        println!("❌ {}", error);
        Err(error)
    }
}

// Start backend server
#[tauri::command]
pub async fn start_backend() -> Result<(), String> {
    println!("🐍 Starting Python backend...");
    
    let output = std::process::Command::new("python3")
        .arg("python/app.py")
        .spawn()
        .map_err(|e| format!("Failed to start backend: {}", e))?;
    
    println!("✅ Backend started with PID: {}", output.id());
    
    // Wait a moment for the backend to start
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    
    Ok(())
} 