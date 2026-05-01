<script lang="ts">
  import { onMount } from "svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import { timerStore } from "$lib/stores/timer.svelte";
  import { windowTrackingStore } from "$lib/stores/window-tracking.svelte";
  import { db } from "$lib/services/db";
  import TimeDisplay from "$lib/components/common/TimeDisplay.svelte";
  import { fly } from "svelte/transition";

  let isMobile = $state(true); // Default to true (safe for mobile/web) until confirmed
  const quickAddMenuId = `quick-add-menu-${Math.random().toString(36).slice(2, 10)}`;

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
    <h1>MyTodos</h1>
  </div>

  <div class="header-right">
    <div class="quick-add-container">
      <button
        type="button"
        class="add-btn"
        class:active={uiStore.showQuickAdd}
        aria-label={uiStore.showQuickAdd ? "Close quick add menu" : "Open quick add menu"}
        aria-controls={quickAddMenuId}
        aria-expanded={uiStore.showQuickAdd}
        aria-haspopup="menu"
        onclick={() => uiStore.toggleQuickAdd()}
        title="Add new..."
      >
        <span class="plus-icon" aria-hidden="true">＋</span>
      </button>

      {#if uiStore.showQuickAdd}
        <div
          id={quickAddMenuId}
          class="quick-add-menu"
          role="menu"
          aria-label="Quick add actions"
          transition:fly={{ y: 10, duration: 200 }}
        >
          <button
            type="button"
            role="menuitem"
            onclick={() => {
              uiStore.openTaskModal();
              uiStore.closeQuickAdd();
            }}
            class="menu-item"
          >
            <span class="menu-icon" aria-hidden="true">✓</span>
            <div class="menu-text">
              <span class="menu-title">New Task</span>
              <span class="menu-desc">Add to current list</span>
            </div>
          </button>
          <button
            type="button"
            role="menuitem"
            onclick={() => {
              uiStore.openProjectModal();
              uiStore.closeQuickAdd();
            }}
            class="menu-item"
          >
            <span class="menu-icon" aria-hidden="true">📁</span>
            <div class="menu-text">
              <span class="menu-title">New Project</span>
              <span class="menu-desc">Create a container</span>
            </div>
          </button>
        </div>
      {/if}
    </div>

    <div
      class="timer-badge"
      class:running={timerStore.isRunning || windowTrackingStore.isWorkActive}
      class:window-tracking={windowTrackingStore.enabled}
      aria-live="polite"
      title={windowTrackingStore.enabled
        ? `Today's active-window time${windowTrackingStore.active ? `: ${windowTrackingStore.active.app_name}` : ""}`
        : "Today's work time"}
    >
      <span class="timer-icon">{windowTrackingStore.enabled ? "▣" : ""}</span>
      <TimeDisplay
        seconds={Math.floor(
          windowTrackingStore.enabled
            ? windowTrackingStore.dailyTotal
            : timerStore.dailyTotal,
        )}
        format="hm"
      />
    </div>

    <button
      type="button"
      class="icon-btn"
      class:active={uiStore.showCalendarView}
      aria-label="Open calendar view"
      onclick={() => uiStore.openCalendarView()}
      title="Calendar"
    >
      📅
    </button>

    <button
      type="button"
      class="icon-btn"
      aria-label="Open statistics view"
      onclick={() => uiStore.openStatsView()}
      title="View statistics"
    >
      📊
    </button>

    <button
      type="button"
      class="icon-btn"
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

  .header-left h1 {
    font-size: 18px;
    font-weight: 700;
    color: var(--accent);
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .quick-add-container {
    position: relative;
  }

  .add-btn {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background-color: var(--accent);
    color: var(--accent-contrast);
    transition: all var(--transition-fast);
    font-size: 18px;
    box-shadow: var(--shadow-sm);
  }

  .add-btn:hover {
    background-color: var(--accent-hover);
    transform: scale(1.1);
  }

  .add-btn.active {
    transform: rotate(45deg);
    background-color: var(--danger);
    color: var(--danger-contrast);
  }

  .quick-add-menu {
    position: absolute;
    top: calc(100% + 10px);
    right: 0;
    width: 180px;
    background-color: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    z-index: 1000;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    padding: var(--spacing-xs);
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm);
    border-radius: var(--radius-md);
    transition: all var(--transition-fast);
    text-align: left;
    width: 100%;
  }

  .menu-item:hover {
    background-color: var(--bg-hover);
  }

  .menu-icon {
    font-size: 16px;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--bg-secondary);
    border-radius: var(--radius-sm);
    color: var(--accent);
  }

  .menu-text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .menu-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .menu-desc {
    font-size: 10px;
    color: var(--text-tertiary);
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

  .timer-badge.window-tracking {
    border-color: color-mix(in srgb, var(--accent) 45%, var(--border));
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
