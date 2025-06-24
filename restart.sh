#!/bin/bash

echo "🔄 Restarting Cursper with fixes..."

# Kill any existing backend processes on port 8788
echo "Stopping existing backend processes..."
lsof -ti:8788 | xargs kill -9 2>/dev/null || true

# Kill any existing Tauri processes
echo "Stopping existing Tauri processes..."
pkill -f cursper 2>/dev/null || true

# Wait a moment
sleep 2

# Start the app
echo "Starting Cursper..."
bun run tauri dev

echo "✅ Cursper started successfully!" 