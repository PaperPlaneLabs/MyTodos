<script lang="ts">
  import { onMount } from "svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import { timerStore } from "$lib/stores/timer.svelte";
  import { db } from "$lib/services/db";
  import TimeDisplay from "$lib/components/common/TimeDisplay.svelte";

  let isMobile = $state(true); // Default to true (safe for mobile/web) until confirmed

  onMount(async () => {
    try {
      // Cast to any because the language server might not see the exact types for dynamic import
      const core = (await import("@tauri-apps/api/core")) as any;
      if (core && core.type) {
        const platformName = await core.type();
        isMobile = platformName === "android" || platformName === "ios";
      } else {
        isMobile = false; // Probably web browser
      }
    } catch (e) {
      // If we're not in a tauri context (e.g. browser), we can assume not mobile for window buttons
      isMobile = false;
      console.warn("Failed to detect platform, assuming non-mobile:", e);
    }
  });

  async function minimize() {
    await db.window.minimize();
  }

  async function toggleMaximize() {
    await db.window.toggleMaximize();
  }

  async function close() {
    await db.window.close();
  }

  async function applyWindowOrientation(orientation: "left" | "right" | "center") {
    if (orientation === "center") {
      await db.window.center();
    } else {
      await db.window.dock(orientation);
    }

    await db.window.setDockPreference(orientation);
    uiStore.setWindowOrientation(orientation);
  }

  async function dock(side: "left" | "right") {
    await applyWindowOrientation(side);
  }

  async function handleDrag(e: MouseEvent) {
    if (isMobile) return;
    // Only drag on left click and not on buttons
    if (e.button === 0 && !(e.target as HTMLElement).closest("button")) {
      await db.window.startDragging();
    }
  }

  function windowDrag(node: HTMLElement) {
    const onMouseDown = (event: MouseEvent) => {
      void handleDrag(event);
    };

    node.addEventListener("mousedown", onMouseDown);
    return {
      destroy() {
        node.removeEventListener("mousedown", onMouseDown);
      },
    };
  }
</script>

{#if !isMobile}
  <div
    class="title-bar"
    use:windowDrag
    data-tauri-drag-region
    role="presentation"
  >
    <div class="window-controls">
      <button
        type="button"
        class="win-btn minimize"
        aria-label="Minimize window"
        onclick={minimize}
        title="Minimize"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true"
          ><rect fill="currentColor" x="1" y="5" width="10" height="1" /></svg
        >
      </button>
      <button
        type="button"
        class="win-btn dock-left"
        aria-label="Dock window to the left"
        onclick={() => dock("left")}
        title="Dock Left"
      >
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <rect width="18" height="18" x="3" y="3" rx="2" />
          <path d="M9 3v18" />
        </svg>
      </button>
      <button
        type="button"
        class="win-btn maximize"
        aria-label="Toggle window maximized state"
        onclick={toggleMaximize}
        title="Maximize"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true"
          ><rect
            fill="none"
            stroke="currentColor"
            stroke-width="1"
            x="1.5"
            y="1.5"
            width="9"
            height="9"
          /></svg
        >
      </button>
      <button
        type="button"
        class="win-btn dock-right"
        aria-label="Dock window to the right"
        onclick={() => dock("right")}
        title="Dock Right"
      >
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <rect width="18" height="18" x="3" y="3" rx="2" />
          <path d="M15 3v18" />
        </svg>
      </button>
      <button
        type="button"
        class="win-btn close"
        aria-label="Close window"
        onclick={close}
        title="Close"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true"
          ><path
            fill="currentColor"
            d="M1.5 1.5l9 9m-9 0l9-9"
            stroke="currentColor"
            stroke-width="1.2"
          /></svg
        >
      </button>
    </div>
  </div>
{/if}

<header
  class="app-header"
  use:windowDrag
  data-tauri-drag-region
>
  <div class="header-left">
    <button
      type="button"
      class="brand-btn"
      class:active={uiStore.primaryView === "today"}
      onclick={() => uiStore.openTodayView()}
      aria-label="Open Today workspace"
      title="Today"
    >Todoz</button>
    <button
      type="button"
      class="projects-btn"
      class:active={uiStore.primaryView === "projects"}
      onclick={() => uiStore.openProjectsView()}
    >Projects</button>
  </div>

  <div class="header-right">
    <div
      class="timer-badge"
      class:running={timerStore.isRunning}
      aria-live="polite"
      title="Today's task time"
    >
      <span class="timer-icon"></span>
      <TimeDisplay
        seconds={Math.floor(timerStore.dailyTotal)}
        format="hm"
      />
    </div>

    <button
      type="button"
      class="icon-btn"
      class:active={uiStore.primaryView === "calendar"}
      aria-label="Open calendar view"
      onclick={() => uiStore.openCalendarView()}
      title="Calendar"
    >
      📅
    </button>

    <button
      type="button"
      class="icon-btn"
      class:active={uiStore.primaryView === "stats"}
      aria-label="Open statistics view"
      onclick={() => uiStore.openStatsView()}
      title="View statistics"
    >
      📊
    </button>

    <button
      type="button"
      class="icon-btn"
      class:active={uiStore.primaryView === "settings"}
      aria-label="Open settings view"
      onclick={() => uiStore.openSettingsView()}
      title="Settings"
    >
      ⚙️
    </button>
  </div>
</header>

<style>
  .title-bar {
    height: 32px;
    background-color: var(--bg-primary);
    display: flex;
    justify-content: flex-end;
    align-items: center;
    user-select: none;
    border-bottom: 1px solid var(--border-light);
    flex-shrink: 0;
  }

  .window-controls {
    display: flex;
    height: 100%;
  }

  .win-btn {
    width: 44px;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    background: transparent;
    border: none;
    transition: all 0.1s;
    border-radius: 0;
  }

  .win-btn:hover {
    background-color: var(--bg-hover);
    color: var(--text-primary);
  }

  .win-btn.close:hover {
    background-color: #e81123;
    color: white;
  }

  .app-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-sm) var(--spacing-md);
    border-bottom: 1px solid var(--border);
    background-color: var(--bg-primary);
    flex-shrink: 0;
    user-select: none;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .brand-btn,
  .projects-btn {
    border: 0;
    background: transparent;
    cursor: pointer;
  }

  .brand-btn {
    font-size: 18px;
    font-weight: 700;
    color: var(--accent);
  }

  .projects-btn {
    padding: var(--spacing-xs) var(--spacing-sm);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    font-size: var(--text-sm);
  }

  .projects-btn:hover,
  .projects-btn.active {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .timer-badge {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: 4px 10px;
    border-radius: 20px;
    background-color: var(--bg-secondary);
    font-size: 12px;
    font-weight: 600;
    font-family: var(--font-mono);
    border: 1px solid var(--border);
    transition: all var(--transition-normal);
    cursor: default;
    color: var(--text-secondary);
  }

  .timer-badge:hover {
    background-color: var(--bg-hover);
    border-color: var(--accent);
  }

  .timer-badge.running {
    background-color: var(--success);
    color: var(--success-contrast);
    border-color: var(--success);
    box-shadow: 0 0 10px var(--success-glow);
    animation: header-pulse 2s ease-in-out infinite;
  }

  @keyframes header-pulse {
    0%,
    100% {
      transform: scale(1);
    }
    50% {
      transform: scale(1.03);
    }
  }

  .timer-icon {
    font-size: 14px;
  }

  .icon-btn {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    transition: all var(--transition-fast);
    font-size: 16px;
  }

  .icon-btn:hover {
    background-color: var(--bg-hover);
    color: var(--text-primary);
  }
</style>
