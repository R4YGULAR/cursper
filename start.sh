#!/bin/bash

# Cursper Startup Script
echo "🎤 Starting Cursper..."

# Check if Python dependencies are installed
echo "Checking Python dependencies..."
if ! python3 -c "import flask, whisper" 2>/dev/null; then
    echo "Installing Python dependencies..."
    cd python
    pip install -r requirements.txt
    cd ..
fi

# Check if Node dependencies are installed
if [ ! -d "node_modules" ]; then
    echo "Installing Node dependencies..."
    bun install
fi

# Start the application
echo "Starting Cursper application..."
bun run tauri dev 