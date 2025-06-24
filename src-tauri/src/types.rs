use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AppState {
    pub is_recording: bool,
    pub current_model: String,
    pub current_shortcut: String,
    pub shortcuts: HashMap<String, String>,
    pub backend_url: String,
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

pub type AppStateType = Arc<Mutex<AppState>>;

// Global recording control
static RECORDING_CONTROL: std::sync::OnceLock<Arc<Mutex<bool>>> = std::sync::OnceLock::new();

pub fn get_recording_control() -> Arc<Mutex<bool>> {
    RECORDING_CONTROL.get_or_init(|| Arc::new(Mutex::new(false))).clone()
} 