#!/usr/bin/env python3
"""
Cursper - Voice to Text Backend
Uses OpenAI Whisper for speech recognition
"""

import os
import sys
import json
import tempfile
import argparse
from pathlib import Path
from typing import Optional, Dict, Any
try:
    from flask import Flask, request, jsonify
    from flask_cors import CORS
    import whisper
    import torch
except ImportError as e:
    print(f"Required dependency missing: {e}")
    print("Please install dependencies: pip install -r requirements.txt")
    sys.exit(1)
import threading
import time

app = Flask(__name__)
CORS(app)

# Global variables for model management
current_model: Optional[Any] = None
current_model_size: str = "base"
model_lock = threading.Lock()

# Available model sizes
AVAILABLE_MODELS = {
    "tiny": "Fastest, lowest quality (39 MB)",
    "base": "Good balance (74 MB)",
    "small": "Better quality (244 MB)", 
    "medium": "High quality (769 MB)",
    "large": "Best quality (1550 MB)"
}

def load_model(model_size):
    """Load or reload Whisper model"""
    global current_model, current_model_size
    
    print(f"🔄 load_model called with size: {model_size}")
    
    with model_lock:
        print(f"🔒 Acquired model lock")
        print(f"📊 Current model: {current_model}, Current size: {current_model_size}")
        
        if current_model is None or current_model_size != model_size:
            print(f"🚀 Loading Whisper model: {model_size}")
            try:
                print(f"⏳ Calling whisper.load_model('{model_size}')...")
                current_model = whisper.load_model(model_size)
                current_model_size = model_size
                print(f"✅ Model {model_size} loaded successfully")
                print(f"📋 Model type: {type(current_model)}")
                return True
            except Exception as e:
                error_msg = f"Error loading model {model_size}: {e}"
                print(f"❌ {error_msg}")
                return False
        else:
            print(f"♻️  Model {model_size} already loaded, skipping")
            return True

@app.route('/health', methods=['GET'])
def health_check():
    """Health check endpoint"""
    print("=" * 50)
    print("🔍 HEALTH CHECK REQUEST RECEIVED")
    print(f"🕐 Time: {time.strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"🌐 Request from: {request.remote_addr}")
    print(f"📋 Request headers: {dict(request.headers)}")
    print(f"🧠 Current model: {current_model_size}")
    print(f"🔍 Model loaded: {current_model is not None}")
    
    response = {
        "status": "healthy",
        "current_model": current_model_size,
        "model_loaded": current_model is not None,
        "available_models": AVAILABLE_MODELS,
        "timestamp": time.strftime('%Y-%m-%d %H:%M:%S')
    }
    print(f"✅ Health check response: {response}")
    print("=" * 50)
    return jsonify(response)

@app.route('/models', methods=['GET'])
def get_models():
    """Get available models"""
    print("📋 Available models requested")
    response = {
        "available_models": AVAILABLE_MODELS,
        "current_model": current_model_size
    }
    print(f"✅ Models response: {response}")
    return jsonify(response)

@app.route('/set_model', methods=['POST'])
def set_model():
    """Set the active model"""
    print("🔧 Set model request received")
    data = request.get_json()
    model_size = data.get('model_size', 'base')
    print(f"📝 Requested model size: {model_size}")
    
    if model_size not in AVAILABLE_MODELS:
        error_msg = f"Invalid model size. Available: {list(AVAILABLE_MODELS.keys())}"
        print(f"❌ {error_msg}")
        return jsonify({"error": error_msg}), 400
    
    print(f"🔄 Loading model: {model_size}")
    success = load_model(model_size)
    if success:
        response = {
            "message": f"Model set to {model_size}",
            "current_model": current_model_size
        }
        print(f"✅ Model set successfully: {response}")
        return jsonify(response)
    else:
        error_msg = f"Failed to load model {model_size}"
        print(f"❌ {error_msg}")
        return jsonify({"error": error_msg}), 500

@app.route('/transcribe', methods=['POST'])
def transcribe_audio():
    """Transcribe audio file to text"""
    try:
        # Check if model is loaded
        if current_model is None:
            if not load_model(current_model_size):
                return jsonify({"error": "Failed to load Whisper model"}), 500
        
        # Double-check model is not None after loading
        if current_model is None:
            return jsonify({"error": "Whisper model not available"}), 500
        
        # Check if file is in request
        if 'audio' not in request.files:
            return jsonify({"error": "No audio file provided"}), 400
        
        audio_file = request.files['audio']
        if audio_file.filename == '':
            return jsonify({"error": "No audio file selected"}), 400
        
        # Save uploaded file temporarily
        with tempfile.NamedTemporaryFile(delete=False, suffix='.wav') as temp_file:
            audio_file.save(temp_file.name)
            temp_path = temp_file.name
        
        try:
            # Transcribe with Whisper
            with model_lock:
                if current_model is not None:
                    result = current_model.transcribe(temp_path)
                else:
                    raise Exception("Model became None during transcription")
            
            # Clean up temp file
            os.unlink(temp_path)
            
            return jsonify({
                "text": result["text"].strip(),
                "language": result.get("language", "unknown"),
                "model_used": current_model_size
            })
            
        except Exception as e:
            # Clean up temp file on error
            if os.path.exists(temp_path):
                os.unlink(temp_path)
            raise e
            
    except Exception as e:
        return jsonify({"error": str(e)}), 500

@app.route('/transcribe_raw', methods=['POST'])
def transcribe_raw_audio():
    """Transcribe raw audio bytes"""
    print("=" * 60)
    print("🎤 RAW AUDIO TRANSCRIPTION REQUEST RECEIVED")
    print(f"🕐 Time: {time.strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"🌐 Request from: {request.remote_addr}")
    print(f"📋 Request headers: {dict(request.headers)}")
    print(f"📋 Request method: {request.method}")
    print(f"📋 Request content type: {request.content_type}")
    print(f"📋 Request content length: {request.content_length}")
    print("=" * 60)
    
    try:
        # Check if model is loaded
        print(f"🔍 Checking if model is loaded. Current model: {current_model}")
        print(f"🔍 Current model size: {current_model_size}")
        
        if current_model is None:
            print("⚠️  Model not loaded, attempting to load...")
            if not load_model(current_model_size):
                error_msg = "Failed to load Whisper model"
                print(f"❌ {error_msg}")
                return jsonify({"error": error_msg}), 500
        
        # Double-check model is not None after loading
        if current_model is None:
            error_msg = "Whisper model not available after loading attempt"
            print(f"❌ {error_msg}")
            return jsonify({"error": error_msg}), 500
        
        print("✅ Whisper model is available")
        print(f"📊 Model type: {type(current_model)}")
        
        # Get raw audio data
        try:
            audio_data = request.get_data()
            print(f"📊 Received audio data: {len(audio_data)} bytes")
        except Exception as e:
            error_msg = f"Failed to get audio data from request: {str(e)}"
            print(f"❌ {error_msg}")
            import traceback
            print(f"🔍 Stack trace: {traceback.format_exc()}")
            return jsonify({"error": error_msg}), 400
        
        if not audio_data:
            error_msg = "No audio data provided in request body"
            print(f"❌ {error_msg}")
            return jsonify({"error": error_msg}), 400
        
        # Analyze the first few bytes to see what we got
        if len(audio_data) >= 12:
            header = audio_data[:12]
            print(f"📋 Audio data header (first 12 bytes): {header}")
            print(f"📋 Header as hex: {header.hex()}")
            print(f"📋 Header as string (ignore errors): {header.decode('ascii', errors='ignore')}")
        
        # Check for dummy/invalid audio data (all zeros or very small)
        if len(audio_data) < 1000:
            print(f"⚠️  Audio data too small: {len(audio_data)} bytes")
            if all(b == 0 for b in audio_data[:min(100, len(audio_data))]):
                print("⚠️  Audio data appears to be all zeros, returning test message")
                response = {
                    "text": "🧪 Test transcription - received zero audio data",
                    "language": "en", 
                    "model_used": current_model_size
                }
                print(f"🎉 Returning test response: {response}")
                return jsonify(response)
        
        # Save raw data to temporary file
        print("💾 Saving audio data to temporary file...")
        try:
            with tempfile.NamedTemporaryFile(delete=False, suffix='.wav') as temp_file:
                temp_file.write(audio_data)
                temp_path = temp_file.name
                print(f"📁 Temporary file created: {temp_path}")
                
            # Verify the file was written correctly
            import os
            file_size = os.path.getsize(temp_path)
            print(f"📊 Temporary file size: {file_size} bytes")
            
            if file_size != len(audio_data):
                print(f"⚠️  File size mismatch! Expected {len(audio_data)}, got {file_size}")
            
        except Exception as e:
            error_msg = f"Failed to save audio data to temporary file: {str(e)}"
            print(f"❌ {error_msg}")
            import traceback
            print(f"🔍 Stack trace: {traceback.format_exc()}")
            return jsonify({"error": error_msg}), 500
        
        try:
            # Transcribe with Whisper
            print("🧠 Starting Whisper transcription...")
            print(f"🔍 Using model: {current_model_size}")
            print(f"🔍 Model object: {current_model}")
            print(f"🔍 Temp file path: {temp_path}")
            
            with model_lock:
                print("🔒 Acquired model lock")
                if current_model is not None:
                    print(f"🔄 Calling current_model.transcribe('{temp_path}')")
                    
                    # Call Whisper transcription with more detailed error handling
                    try:
                        result = current_model.transcribe(temp_path)
                        print(f"✅ Whisper transcription raw result: {result}")
                        print(f"📝 Result type: {type(result)}")
                        if isinstance(result, dict):
                            print(f"📝 Result keys: {list(result.keys())}")
                    except Exception as whisper_error:
                        error_msg = f"Whisper transcription failed: {str(whisper_error)}"
                        print(f"❌ {error_msg}")
                        import traceback
                        print(f"🔍 Whisper error stack trace: {traceback.format_exc()}")
                        raise whisper_error
                        
                else:
                    error_msg = "Model became None during transcription"
                    print(f"❌ {error_msg}")
                    raise Exception(error_msg)
            
            print("🔓 Released model lock")
            
            # Clean up temp file
            print(f"🗑️  Cleaning up temporary file: {temp_path}")
            try:
                os.unlink(temp_path)
                print("✅ Temporary file cleaned up")
            except Exception as cleanup_error:
                print(f"⚠️  Failed to clean up temp file: {cleanup_error}")
            
            # Process the result
            try:
                response = {
                    "text": result["text"].strip() if "text" in result else "No text found",
                    "language": result.get("language", "unknown"),
                    "model_used": current_model_size
                }
                print(f"🎉 Transcription successful: {response}")
                return jsonify(response)
            except Exception as response_error:
                error_msg = f"Failed to process transcription result: {str(response_error)}"
                print(f"❌ {error_msg}")
                print(f"🔍 Raw result was: {result}")
                import traceback
                print(f"🔍 Response processing stack trace: {traceback.format_exc()}")
                return jsonify({"error": error_msg}), 500
            
        except Exception as transcription_error:
            # Clean up temp file on error
            if os.path.exists(temp_path):
                print(f"🗑️  Cleaning up temporary file after error: {temp_path}")
                try:
                    os.unlink(temp_path)
                except:
                    pass
            
            error_msg = f"Transcription process error: {str(transcription_error)}"
            print(f"❌ {error_msg}")
            import traceback
            print(f"🔍 Transcription error stack trace: {traceback.format_exc()}")
            raise transcription_error
            
    except Exception as e:
        error_msg = f"Raw transcription endpoint failed: {str(e)}"
        print(f"❌ {error_msg}")
        print(f"❌ Error type: {type(e).__name__}")
        import traceback
        print(f"🔍 Full stack trace:")
        print(traceback.format_exc())
        return jsonify({
            "error": error_msg,
            "error_type": type(e).__name__,
            "traceback": traceback.format_exc()
        }), 500

def main():
    parser = argparse.ArgumentParser(description='Cursper Voice to Text Backend')
    parser.add_argument('--port', type=int, default=8788, help='Port to run the server on')
    parser.add_argument('--host', default='127.0.0.1', help='Host to bind to')
    parser.add_argument('--model', default='base', choices=list(AVAILABLE_MODELS.keys()),
                       help='Initial Whisper model to load')
    parser.add_argument('--debug', action='store_true', help='Run in debug mode')
    
    args = parser.parse_args()
    
    # Load initial model
    print("=" * 60)
    print("🎤 CURSPER BACKEND STARTING UP")
    print("=" * 60)
    print(f"🌐 Starting Cursper backend on {args.host}:{args.port}")
    print(f"🧠 Loading initial model: {args.model}")
    print(f"🐛 Debug mode: {args.debug}")
    print(f"📋 Available models: {list(AVAILABLE_MODELS.keys())}")
    
    success = load_model(args.model)
    if not success:
        print("⚠️  Failed to load initial model, but continuing...")
    else:
        print("✅ Initial model loaded successfully")
    
    # Start Flask server
    print("=" * 60)
    print("🚀 STARTING FLASK SERVER")
    print("=" * 60)
    print(f"🌐 Server URL: http://{args.host}:{args.port}")
    print(f"🔗 Health check: http://{args.host}:{args.port}/health")
    print(f"🎤 Transcribe endpoint: http://{args.host}:{args.port}/transcribe_raw")
    print("=" * 60)
    
    app.run(
        host=args.host,
        port=args.port,
        debug=args.debug,
        threaded=True
    )

if __name__ == '__main__':
    main()
