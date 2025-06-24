<script lang="ts">
  import { onMount } from "svelte";

  let isRecording = $state(false);
  let animationFrame = $state(0);

  onMount(() => {
    // Listen for recording state changes from the main process
    // This would typically come from Tauri events in a real implementation
    
    // For now, we'll simulate the recording animation
    const animate = () => {
      animationFrame = (animationFrame + 1) % 60;
      requestAnimationFrame(animate);
    };
    animate();
  });
</script>

<div class="overlay-container">
  <div class="microphone-icon {isRecording ? 'recording' : ''}">
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path
        d="M12 1C10.34 1 9 2.34 9 4V12C9 13.66 10.34 15 12 15C13.66 15 15 13.66 15 12V4C15 2.34 13.66 1 12 1Z"
        fill="currentColor"
      />
      <path
        d="M19 10V12C19 16.42 15.42 20 11 20H13C17.42 20 21 16.42 21 12V10H19Z"
        fill="currentColor"
      />
      <path
        d="M7 12V10H5V12C5 16.42 8.58 20 13 20H11C6.58 20 3 16.42 3 12Z"
        fill="currentColor"
      />
      <path d="M11 22H13V24H11V22Z" fill="currentColor" />
    </svg>
  </div>
  
  {#if isRecording}
    <div class="recording-indicator">
      <div class="pulse-ring"></div>
      <div class="pulse-ring pulse-ring-delay"></div>
    </div>
  {/if}
</div>

<style>
  .overlay-container {
    position: fixed;
    top: 0;
    left: 0;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
    z-index: 9999;
  }

  .microphone-icon {
    width: 24px;
    height: 24px;
    background: rgba(59, 130, 246, 0.9);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
    backdrop-filter: blur(10px);
    border: 2px solid rgba(255, 255, 255, 0.3);
    transition: all 0.3s ease;
    position: relative;
  }

  .microphone-icon.recording {
    background: rgba(239, 68, 68, 0.9);
    animation: recordingPulse 1s ease-in-out infinite alternate;
  }

  .recording-indicator {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    pointer-events: none;
  }

  .pulse-ring {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 40px;
    height: 40px;
    border: 2px solid rgba(239, 68, 68, 0.4);
    border-radius: 50%;
    animation: pulseRing 2s ease-out infinite;
  }

  .pulse-ring-delay {
    animation-delay: 1s;
  }

  @keyframes recordingPulse {
    0% {
      transform: scale(1);
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
    }
    100% {
      transform: scale(1.1);
      box-shadow: 0 4px 16px rgba(239, 68, 68, 0.4);
    }
  }

  @keyframes pulseRing {
    0% {
      opacity: 1;
      transform: translate(-50%, -50%) scale(0.8);
    }
    100% {
      opacity: 0;
      transform: translate(-50%, -50%) scale(2);
    }
  }

  /* Make the overlay transparent to mouse events */
  :global(body) {
    -webkit-app-region: no-drag;
  }
</style> 