use crate::types::{AppStateType, get_recording_control};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// Start recording audio with platform-specific tools
#[tauri::command]
pub async fn start_recording(
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
pub async fn stop_recording_and_transcribe(
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
pub async fn stop_recording_and_transcribe_internal(backend_url: String) -> Result<String, String> {
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