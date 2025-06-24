<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let currentModel = $state("base");
  let availableModels = $state<string[]>([]);
  let isRecording = $state(false);
  let backendStatus = $state("checking");
  let statusMessage = $state("Initializing...");
  let currentPlatform = $state("unknown");
  let currentShortcut = $state("Option+Space");
  let isEditingShortcut = $state(false);
  let tempShortcut = $state("");

  // Platform-specific shortcut display
  function getShortcutDisplay(shortcut: string): string {
    if (currentPlatform === "macos") {
      return shortcut
        .replace("CmdOrCtrl", "⌘")
        .replace("Cmd", "⌘")
        .replace("Ctrl", "⌃")
        .replace("Shift", "⇧")
        .replace("Alt", "⌥")
        .replace("Space", "Space");
    } else {
      return shortcut
        .replace("CmdOrCtrl", "Ctrl")
        .replace("Cmd", "Ctrl")
        .replace("+", " + ");
    }
  }

  async function checkBackendStatus() {
    try {
      const models = await invoke<string[]>("get_available_models");
      availableModels = models;
      backendStatus = "connected";
      statusMessage = "Backend connected ✅";
    } catch (error) {
      backendStatus = "disconnected";
      statusMessage = "Backend disconnected ❌";
      console.error("Backend check failed:", error);
    }
  }

  async function setModel(model: string) {
    try {
      await invoke("set_whisper_model", { model });
      currentModel = model;
      statusMessage = `Model set to ${model} ✅`;
    } catch (error) {
      statusMessage = `Failed to set model: ${error}`;
      console.error("Model set failed:", error);
    }
  }

  async function startBackend() {
    try {
      await invoke("start_backend");
      statusMessage = "Starting backend...";
      // Wait a bit then check status
      setTimeout(checkBackendStatus, 3000);
    } catch (error) {
      statusMessage = `Failed to start backend: ${error}`;
      console.error("Backend start failed:", error);
    }
  }

  async function toggleRecording() {
    try {
      await invoke("toggle_recording");
      isRecording = !isRecording;
      statusMessage = isRecording ? "Recording..." : "Recording stopped";
    } catch (error) {
      statusMessage = `Recording error: ${error}`;
      console.error("Recording toggle failed:", error);
    }
  }

  async function updateShortcut(newShortcut: string) {
    try {
      await invoke("update_global_shortcut", { shortcut: newShortcut });
      currentShortcut = newShortcut;
      statusMessage = `Shortcut updated to ${getShortcutDisplay(newShortcut)} ✅`;
      isEditingShortcut = false;
    } catch (error) {
      statusMessage = `Failed to update shortcut: ${error}`;
      console.error("Shortcut update failed:", error);
    }
  }

  function startEditingShortcut() {
    isEditingShortcut = true;
    tempShortcut = currentShortcut;
  }

  function cancelEditingShortcut() {
    isEditingShortcut = false;
    tempShortcut = "";
  }

  function saveShortcut() {
    if (tempShortcut.trim()) {
      updateShortcut(tempShortcut);
    }
  }

  onMount(() => {
    // Detect platform using navigator
    if (typeof navigator !== 'undefined') {
      const userAgent = navigator.userAgent.toLowerCase();
      if (userAgent.includes('mac')) {
        currentPlatform = "macos";
        currentShortcut = "Option+Space";
      } else if (userAgent.includes('win')) {
        currentPlatform = "windows";
        currentShortcut = "Alt+Space";
      } else {
        currentPlatform = "linux";
        currentShortcut = "Alt+Space";
      }
    }

    console.log("Platform:", currentPlatform);
    checkBackendStatus();

    // Check backend status every 10 seconds
    const interval = setInterval(checkBackendStatus, 10000);
    return () => clearInterval(interval);
  });
</script>

<main class="container">
  <div class="header">
    <h1>🎤 Cursper</h1>
    <p class="subtitle">Voice to Text Cursor Assistant</p>
  </div>

  <div class="message-card">
    <div class="icon">👻</div>
    <h2>Background App Mode</h2>
    <p>Cursper is running invisibly in the background. The app won't appear in your dock - it's purely a system tray application.</p>
    
    <div class="features">
      <div class="feature">
        <span class="feature-icon">🔍</span>
        <span>Look for the microphone icon in your system tray/menu bar</span>
      </div>
      <div class="feature">
        <span class="feature-icon">⌨️</span>
        <span>Use <kbd>Option+Space</kbd> to start/stop recording globally</span>
      </div>
      <div class="feature">
        <span class="feature-icon">🖱️</span>
        <span>Click the tray icon to open settings</span>
      </div>
      <div class="feature">
        <span class="feature-icon">❌</span>
        <span>Use tray menu → Quit to exit the app</span>
      </div>
    </div>
    
    <div class="actions">
      <button class="settings-button" onclick={() => window.location.href = '/tray'}>
        ⚙️ Open Settings
      </button>
      <button class="hide-button" onclick={() => window.close()}>
        👻 Hide Window
      </button>
    </div>
  </div>
</main>

<style>
  :global(body) {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    margin: 0;
    padding: 0;
    background: #0f0f0f;
    min-height: 100vh;
    color: #ffffff;
  }

  .container {
    max-width: 500px;
    margin: 0 auto;
    padding: 40px 20px;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    gap: 30px;
  }

  .header {
    text-align: center;
    color: white;
  }

  .header h1 {
    font-size: 2.5rem;
    margin: 0;
    font-weight: 700;
    background: linear-gradient(45deg, #667eea, #764ba2);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .subtitle {
    margin: 8px 0 0 0;
    opacity: 0.7;
    font-size: 1rem;
    color: #a1a1aa;
  }

  .message-card {
    background: #1a1a1a;
    border-radius: 16px;
    padding: 40px 30px;
    text-align: center;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    border: 1px solid #333333;
    max-width: 450px;
    width: 100%;
  }

  .icon {
    font-size: 3rem;
    margin-bottom: 20px;
  }

  .message-card h2 {
    margin: 0 0 16px 0;
    color: #ffffff;
    font-size: 1.5rem;
    font-weight: 600;
  }

  .message-card p {
    margin: 0 0 24px 0;
    color: #a1a1aa;
    line-height: 1.5;
    font-size: 1rem;
  }

  .features {
    display: flex;
    flex-direction: column;
    gap: 16px;
    margin: 24px 0;
  }

  .feature {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    background: #262626;
    border-radius: 8px;
    text-align: left;
  }

  .feature-icon {
    font-size: 1.2rem;
    width: 24px;
    text-align: center;
  }

  .feature span:last-child {
    color: #e4e4e7;
    font-size: 0.9rem;
  }

  kbd {
    background: #404040;
    border: 1px solid #525252;
    border-radius: 4px;
    padding: 2px 6px;
    font-size: 0.8rem;
    font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', sans-serif;
    font-weight: 500;
    color: #fbbf24;
  }

  .actions {
    margin-top: 24px;
    display: flex;
    justify-content: center;
    gap: 12px;
    flex-wrap: wrap;
  }

  .settings-button, .hide-button {
    border: none;
    border-radius: 8px;
    padding: 12px 20px;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    font-family: inherit;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .settings-button {
    background: #3b82f6;
    color: white;
  }

  .settings-button:hover {
    background: #2563eb;
    transform: translateY(-1px);
  }

  .hide-button {
    background: #6b7280;
    color: white;
  }

  .hide-button:hover {
    background: #4b5563;
    transform: translateY(-1px);
  }
</style>
