# 🎤 Cursper - Voice to Text Cursor Assistant

Cursper is a cross-platform application that replaces your system's dictation feature with OpenAI's Whisper speech recognition. It allows you to speak text directly at your cursor position using global keyboard shortcuts.

## Features

- 🎯 **Cursor Integration**: Types text directly where your cursor is positioned
- 🎙️ **OpenAI Whisper**: Multiple model sizes (tiny, base, small, medium, large)
- ⌨️ **Global Shortcuts**: Works system-wide with `Ctrl+Shift+Space`
- 🎨 **Visual Feedback**: Microphone overlay appears near cursor during recording
- 🔧 **Cross-Platform**: Supports macOS, Windows, and Linux
- 🚀 **Fast & Lightweight**: Built with Tauri and Svelte

## Quick Start

### Prerequisites

- **Node.js** (v18 or higher)
- **Rust** (latest stable)
- **Python 3.8+** 
- **Bun** (for package management)

### 1. Install Dependencies

```bash
# Install frontend dependencies
bun install

# Install Python dependencies
cd python
pip install -r requirements.txt
cd ..
```

### 2. Start the Application

```bash
# Start in development mode
bun run tauri dev
```

The app will:
1. Launch the settings window
2. Automatically start the Python backend
3. Load the default Whisper model (base)

### 3. Usage

1. **Setup**: Choose your preferred Whisper model in the settings
2. **Position**: Place your cursor where you want text to appear
3. **Record**: Press `Ctrl+Shift+Space` to start recording (microphone icon appears)
4. **Speak**: Say your text clearly
5. **Insert**: Press `Ctrl+Shift+Space` again to stop and insert the transcribed text

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Tauri App     │    │  Python Backend │    │  OpenAI Whisper │
│  (Rust + JS)    │◄──►│    (Flask)      │◄──►│     Models      │
│                 │    │                 │    │                 │
│ • UI            │    │ • Speech-to-text│    │ • tiny (39MB)   │
│ • Shortcuts     │    │ • Model mgmt    │    │ • base (74MB)   │
│ • Cursor pos    │    │ • Audio proc    │    │ • small (244MB) │
│ • Text typing   │    │                 │    │ • medium (769MB)│
│                 │    │                 │    │ • large (1.5GB) │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Components

### Frontend (Tauri + Svelte)
- **Main Window**: Settings and model selection
- **Overlay Window**: Microphone indicator near cursor
- **Global Shortcuts**: System-wide keyboard capture
- **Cursor Integration**: Position detection and text insertion

### Backend (Python + Flask)
- **Whisper Integration**: Model loading and transcription
- **REST API**: Communication with frontend
- **Audio Processing**: Handle audio files and raw data
- **Model Management**: Dynamic model switching

## API Endpoints

The Python backend exposes these endpoints:

- `GET /health` - Health check and status
- `GET /models` - Available Whisper models
- `POST /set_model` - Change active model
- `POST /transcribe` - Transcribe audio file
- `POST /transcribe_raw` - Transcribe raw audio bytes

## Configuration

### Keyboard Shortcuts
- `Ctrl+Shift+Space` - Toggle recording (default)
- Customizable in future versions

### Model Selection
- **tiny**: Fastest, lowest quality (39 MB)
- **base**: Good balance of speed and quality (74 MB) - Default
- **small**: Better quality, slower (244 MB)
- **medium**: High quality (769 MB)
- **large**: Best quality, slowest (1.5 GB)

## Building for Production

```bash
# Build the application
bun run tauri build
```

This creates:
- **macOS**: `.app` bundle in `src-tauri/target/release/bundle/macos/`
- **Windows**: `.exe` installer in `src-tauri/target/release/bundle/msi/`
- **Linux**: AppImage in `src-tauri/target/release/bundle/appimage/`

## Development

### Project Structure
```
cursper/
├── src/                    # Frontend (Svelte)
│   ├── routes/
│   │   ├── +page.svelte   # Main settings UI
│   │   └── overlay/       # Microphone overlay
│   └── app.html
├── src-tauri/             # Backend (Rust)
│   ├── src/
│   │   ├── lib.rs        # Main Tauri logic
│   │   └── main.rs
│   └── Cargo.toml
├── python/                # Python backend
│   ├── app.py           # Flask server with Whisper
│   └── requirements.txt
└── package.json
```

### Adding Features

1. **New Shortcuts**: Modify `setup_shortcuts()` in `src-tauri/src/lib.rs`
2. **UI Changes**: Edit Svelte components in `src/routes/`
3. **Backend Logic**: Update Flask routes in `python/app.py`
4. **Cross-platform Code**: Use conditional compilation in Rust

## Troubleshooting

### Backend Not Starting
```bash
# Check Python installation
python3 --version

# Install dependencies manually
cd python
pip install flask flask-cors openai-whisper torch

# Start backend manually
python3 app.py --port 8787
```

### Global Shortcuts Not Working
- **macOS**: Grant accessibility permissions in System Preferences
- **Windows**: Run as administrator if needed
- **Linux**: Install required system packages

### Model Loading Issues
- Ensure sufficient disk space for models
- Check internet connection for initial downloads
- Models are cached in `~/.cache/whisper/`

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test across platforms
5. Submit a pull request

## License

MIT License - See [LICENSE](LICENSE) for details.

## Acknowledgments

- [OpenAI Whisper](https://github.com/openai/whisper) for speech recognition
- [Tauri](https://tauri.app/) for the cross-platform framework
- [Svelte](https://svelte.dev/) for the reactive UI
- [Flask](https://flask.palletsprojects.com/) for the Python backend
