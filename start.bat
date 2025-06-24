@echo off
echo 🎤 Starting Cursper...

REM Check if Python dependencies are installed
echo Checking Python dependencies...
python -c "import flask, whisper" 2>nul
if errorlevel 1 (
    echo Installing Python dependencies...
    cd python
    pip install -r requirements.txt
    cd ..
)

REM Check if Node dependencies are installed
if not exist "node_modules" (
    echo Installing Node dependencies...
    bun install
)

REM Start the application
echo Starting Cursper application...
bun run tauri dev
pause 