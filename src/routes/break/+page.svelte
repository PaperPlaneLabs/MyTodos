<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { emit } from "@tauri-apps/api/event";
  import "$lib/styles/global.css";

  let message = $state("Quick break time. Stand up, stretch, and reset.");
  let isDragging = $state(false);

  onMount(() => {
    // Read theme from localStorage so the break window matches the main app
    const savedTheme = localStorage.getItem("theme") ?? "light";
    document.documentElement.setAttribute("data-theme", savedTheme);

    // Read message from URL query param
    const params = new URLSearchParams(window.location.search);
    const raw = params.get("message");
    if (raw) {
      message = raw.replace(/\+/g, " ");
    }
  });

  async function sendAction(action: "take_break" | "dismiss" | "snooze") {
    try {
      await emit("break:action", { action });
      await invoke("close_break_window");
    } catch (e) {
      console.error("Break action failed:", e);
    }
  }

  function handleDragStart(e: MouseEvent) {
    if ((e.target as HTMLElement).closest(".break-actions")) return;
    invoke("start_window_drag").catch(() => {});
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="break-window"
  onmousedown={handleDragStart}
  role="dialog"
  tabindex="-1"
  aria-modal="true"
  aria-label="Break reminder"
>
  <div class="break-icon">&#9786;</div>

  <h2 class="break-title">Time for a quick break?</h2>

  <p class="break-message">{message}</p>

  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="break-actions" onmousedown={(e) => e.stopPropagation()}>
    <button
      type="button"
      class="btn btn-warning break-btn"
      onclick={() => sendAction("take_break")}
    >
      Take a break
    </button>
    <button
      type="button"
      class="btn btn-primary break-btn"
      onclick={() => sendAction("dismiss")}
    >
      Keep going
    </button>
    <button
      type="button"
      class="btn btn-secondary break-btn"
      onclick={() => sendAction("snooze")}
    >
      Remind me in 10 min
    </button>
  </div>
</div>

<style>
  :global(html, body) {
    margin: 0;
    padding: 0;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background: transparent;
  }

  .break-window {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    width: 100%;
    height: 100vh;
    padding: 28px 32px 24px;
    box-sizing: border-box;
    background-color: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 14px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.35), 0 4px 16px rgba(0, 0, 0, 0.2);
    cursor: default;
    user-select: none;
  }

  .break-icon {
    font-size: 36px;
    line-height: 1;
    margin-bottom: 2px;
  }

  .break-title {
    font-size: 16px;
    font-weight: 700;
    color: var(--text-primary);
    text-align: center;
    margin: 0;
    line-height: 1.3;
  }

  .break-message {
    font-size: 13px;
    line-height: 1.55;
    color: var(--text-secondary);
    text-align: center;
    margin: 0;
    max-width: 320px;
  }

  .break-actions {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
    max-width: 260px;
    margin-top: 4px;
  }

  .break-btn {
    width: 100%;
    justify-content: center;
    font-size: 13px;
    padding: 9px 16px;
    border-radius: 8px;
    font-weight: 500;
    transition: all 150ms ease;
  }

  .break-btn:hover {
    transform: translateY(-1px);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  }

  .break-btn:active {
    transform: translateY(0);
  }
</style>
