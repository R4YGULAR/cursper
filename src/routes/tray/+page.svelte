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

  <div class="status-card">
    <div class="status-indicator {backendStatus}"></div>
    <div class="status-text">
      <span class="status-label">Status:</span>
      <span class="status-message">{statusMessage}</span>
    </div>
  </div>

  <div class="card">
    <h3>Keyboard Shortcut</h3>
    <p class="description">Customize your activation shortcut</p>
    
    <div class="shortcut-settings">
      {#if !isEditingShortcut}
        <div class="current-shortcut">
          <span class="shortcut-display">{getShortcutDisplay(currentShortcut)}</span>
          <button class="edit-button" onclick={startEditingShortcut}>
            ✏️ Edit
          </button>
        </div>
      {:else}
        <div class="shortcut-editor">
          <input 
            class="shortcut-input"
            bind:value={tempShortcut}
            placeholder="e.g., {currentPlatform === 'macos' ? 'Option+Space' : 'Alt+Space'}, F5, or Cmd+Shift+V"
          />
          <div class="shortcut-buttons">
            <button class="save-button" onclick={saveShortcut}>Save</button>
            <button class="cancel-button" onclick={cancelEditingShortcut}>Cancel</button>
          </div>
        </div>
      {/if}
      
      <div class="shortcut-help">
        <p><strong>Available modifiers:</strong></p>
        <ul>
          <li><code>{currentPlatform === 'macos' ? 'Cmd' : 'Ctrl'}</code> - Command/Control key</li>
          <li><code>Shift</code> - Shift key</li>
          <li><code>Alt</code> - Alt/Option key</li>
          <li><code>Space</code>, <code>A-Z</code>, <code>F1-F12</code> - Keys</li>
        </ul>
        <p><em>Example: {currentPlatform === 'macos' ? 'Cmd+Shift+V' : 'Ctrl+Alt+V'}</em></p>
      </div>
    </div>
  </div>

  <div class="card">
    <h3>Whisper Model</h3>
    <p class="description">Choose the Whisper model size (larger = better quality, slower)</p>
    
    <div class="model-grid">
      {#each availableModels as model}
        <button 
          class="model-button {currentModel === model ? 'active' : ''}"
          onclick={() => setModel(model)}
          disabled={backendStatus !== 'connected'}
        >
          <span class="model-name">{model}</span>
          <span class="model-size">
            {model === 'tiny' ? '39MB' : 
             model === 'base' ? '74MB' :
             model === 'small' ? '244MB' :
             model === 'medium' ? '769MB' : '1.5GB'}
          </span>
        </button>
      {/each}
    </div>
  </div>

  <div class="card">
    <h3>Recording Control</h3>
    <p class="description">Test the voice recording functionality</p>
    
    <button 
      class="record-button {isRecording ? 'recording' : ''}"
      onclick={toggleRecording}
      disabled={backendStatus !== 'connected'}
    >
      {isRecording ? '🔴 Stop Recording' : '🎤 Start Recording'}
    </button>
  </div>

  <div class="card">
    <h3>Usage</h3>
    <div class="usage-instructions">
      <div class="shortcut">
        <span class="keys">{getShortcutDisplay(currentShortcut)}</span>
        <span class="action">Start/Stop Recording</span>
      </div>
      <div class="steps">
        <ol>
          <li>Position your cursor where you want text</li>
          <li>Press <kbd>{getShortcutDisplay(currentShortcut)}</kbd> or click the button above</li>
          <li>Speak your text clearly</li>
          <li>Press <kbd>{getShortcutDisplay(currentShortcut)}</kbd> again to stop and insert text</li>
        </ol>
      </div>
    </div>
  </div>

  {#if backendStatus === 'disconnected'}
    <div class="card">
      <h3>Backend Setup</h3>
      <p>The Python backend is not running. Click below to start it:</p>
      <button class="primary-button" onclick={startBackend}>
        Start Backend
      </button>
      <div class="setup-info">
        <p><strong>First time setup:</strong></p>
        <pre><code>cd python
pip install -r requirements.txt</code></pre>
      </div>
    </div>
  {/if}
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
    max-width: 420px;
    margin: 0 auto;
    padding: 16px;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .header {
    text-align: center;
    color: white;
    margin-bottom: 8px;
  }

  .header h1 {
    font-size: 1.8rem;
    margin: 0;
    font-weight: 700;
    background: linear-gradient(45deg, #667eea, #764ba2);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .subtitle {
    margin: 4px 0 0 0;
    opacity: 0.7;
    font-size: 0.9rem;
    color: #a1a1aa;
  }

  .card {
    background: #1a1a1a;
    border-radius: 12px;
    padding: 20px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    border: 1px solid #333333;
  }

  .status-card {
    background: #1a1a1a;
    border-radius: 10px;
    padding: 14px 16px;
    display: flex;
    align-items: center;
    gap: 12px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
    border: 1px solid #333333;
  }

  .status-indicator {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    animation: pulse 2s infinite;
  }

  .status-indicator.connected {
    background: #10b981;
  }

  .status-indicator.disconnected {
    background: #ef4444;
  }

  .status-indicator.checking {
    background: #f59e0b;
  }

  .status-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .status-label {
    font-size: 0.7rem;
    color: #71717a;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .status-message {
    font-size: 0.85rem;
    font-weight: 500;
    color: #e4e4e7;
  }

  h3 {
    margin: 0 0 6px 0;
    color: #ffffff;
    font-size: 1.1rem;
    font-weight: 600;
  }

  .description {
    margin: 0 0 16px 0;
    color: #a1a1aa;
    font-size: 0.8rem;
    line-height: 1.4;
  }

  .shortcut-settings {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .current-shortcut {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 14px;
    background: #262626;
    border-radius: 8px;
    border: 1px solid #404040;
  }

  .shortcut-display {
    font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', sans-serif;
    font-size: 1rem;
    font-weight: 600;
    color: #ffffff;
  }

  .edit-button {
    background: #3b82f6;
    color: white;
    border: none;
    border-radius: 5px;
    padding: 5px 10px;
    font-size: 0.7rem;
    cursor: pointer;
    transition: background 0.2s ease;
  }

  .edit-button:hover {
    background: #2563eb;
  }

  .shortcut-editor {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .shortcut-input {
    padding: 10px 14px;
    border: 1px solid #404040;
    border-radius: 6px;
    font-size: 0.9rem;
    font-family: inherit;
    background: #262626;
    color: #ffffff;
    transition: border-color 0.2s ease;
  }

  .shortcut-input:focus {
    outline: none;
    border-color: #3b82f6;
  }

  .shortcut-buttons {
    display: flex;
    gap: 6px;
  }

  .save-button {
    background: #10b981;
    color: white;
    border: none;
    border-radius: 5px;
    padding: 6px 12px;
    cursor: pointer;
    transition: background 0.2s ease;
    font-family: inherit;
    font-size: 0.8rem;
  }

  .save-button:hover {
    background: #059669;
  }

  .cancel-button {
    background: #6b7280;
    color: white;
    border: none;
    border-radius: 5px;
    padding: 6px 12px;
    cursor: pointer;
    transition: background 0.2s ease;
    font-family: inherit;
    font-size: 0.8rem;
  }

  .cancel-button:hover {
    background: #4b5563;
  }

  .shortcut-help {
    padding: 12px;
    background: #262626;
    border-radius: 6px;
    border-left: 3px solid #3b82f6;
  }

  .shortcut-help p {
    margin: 0 0 6px 0;
    font-size: 0.8rem;
    color: #e4e4e7;
  }

  .shortcut-help ul {
    margin: 6px 0;
    padding-left: 16px;
    font-size: 0.75rem;
    color: #a1a1aa;
  }

  .shortcut-help li {
    margin-bottom: 3px;
  }

  .shortcut-help code {
    background: #404040;
    padding: 1px 3px;
    border-radius: 2px;
    font-family: 'SF Mono', Monaco, monospace;
    font-size: 0.7rem;
    color: #fbbf24;
  }

  .model-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }

  .model-button {
    background: #262626;
    border: 1px solid #404040;
    border-radius: 8px;
    padding: 12px 8px;
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 3px;
    font-family: inherit;
    color: #ffffff;
  }

  .model-button:hover:not(:disabled) {
    border-color: #3b82f6;
    background: #1e293b;
  }

  .model-button.active {
    border-color: #3b82f6;
    background: #1e3a8a;
    color: #dbeafe;
  }

  .model-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .model-name {
    font-weight: 600;
    font-size: 0.9rem;
    text-transform: capitalize;
  }

  .model-size {
    font-size: 0.7rem;
    color: #a1a1aa;
  }

  .record-button {
    width: 100%;
    padding: 12px 20px;
    font-size: 1rem;
    font-weight: 600;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.3s ease;
    font-family: inherit;
    background: #3b82f6;
    color: white;
  }

  .record-button:hover:not(:disabled) {
    background: #2563eb;
    transform: translateY(-1px);
  }

  .record-button.recording {
    background: #ef4444;
    animation: recordPulse 1s ease-in-out infinite alternate;
  }

  .record-button.recording:hover {
    background: #dc2626;
  }

  .record-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .usage-instructions {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .shortcut {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 12px;
    background: #262626;
    border-radius: 6px;
  }

  .keys {
    font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', sans-serif;
    background: #0f172a;
    color: #e2e8f0;
    padding: 3px 6px;
    border-radius: 3px;
    font-size: 0.8rem;
    font-weight: 500;
    border: 1px solid #334155;
  }

  .action {
    font-weight: 500;
    color: #e4e4e7;
    font-size: 0.85rem;
  }

  .steps ol {
    margin: 0;
    padding-left: 16px;
    color: #a1a1aa;
    line-height: 1.5;
    font-size: 0.85rem;
  }

  .steps li {
    margin-bottom: 6px;
  }

  kbd {
    background: #404040;
    border: 1px solid #525252;
    border-radius: 3px;
    padding: 1px 4px;
    font-size: 0.7rem;
    font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', sans-serif;
    font-weight: 500;
    color: #fbbf24;
  }

  .primary-button {
    background: #3b82f6;
    color: white;
    border: none;
    border-radius: 6px;
    padding: 10px 20px;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.2s ease;
    font-family: inherit;
  }

  .primary-button:hover {
    background: #2563eb;
  }

  .setup-info {
    margin-top: 12px;
    padding: 12px;
    background: #262626;
    border-radius: 6px;
    border-left: 3px solid #3b82f6;
  }

  .setup-info p {
    margin: 0 0 6px 0;
    font-weight: 500;
    color: #ffffff;
    font-size: 0.85rem;
  }

  .setup-info pre {
    margin: 0;
    padding: 8px;
    background: #0f0f0f;
    color: #f9fafb;
    border-radius: 4px;
    font-size: 0.75rem;
    overflow-x: auto;
    border: 1px solid #404040;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  @keyframes recordPulse {
    0% { transform: scale(1); }
    100% { transform: scale(1.02); }
  }
</style> 