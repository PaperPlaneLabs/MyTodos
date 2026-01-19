<script lang="ts">
  import { onMount } from "svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import { timerStore } from "$lib/stores/timer.svelte";
  import { db } from "$lib/services/db";
  import TimeDisplay from "$lib/components/common/TimeDisplay.svelte";
  import { fly } from "svelte/transition";

  let elapsed = $derived(Math.floor(timerStore.elapsed));
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

  async function dock(side: "left" | "right") {
    await db.window.dock(side);
  }

  async function handleDrag(e: MouseEvent) {
    if (isMobile) return;
    // Only drag on left click and not on buttons
    if (e.button === 0 && !(e.target as HTMLElement).closest('button')) {
      await db.window.startDragging();
    }
  }
</script>

{#if !isMobile}
  <div class="title-bar" onmousedown={handleDrag} data-tauri-drag-region>
    <div class="window-controls">
      <button class="win-btn minimize" onclick={minimize} title="Minimize">
        <svg width="12" height="12" viewBox="0 0 12 12"><rect fill="currentColor" x="1" y="5" width="10" height="1"/></svg>
      </button>
      <button class="win-btn dock-left" onclick={() => dock('left')} title="Dock Left">
        <svg width="12" height="12" viewBox="0 0 12 12"><path fill="currentColor" d="M1 1h4v10H1V1zm1 1v8h2V2H2z"/></svg>
      </button>
      <button class="win-btn maximize" onclick={toggleMaximize} title="Maximize">
        <svg width="12" height="12" viewBox="0 0 12 12"><rect fill="none" stroke="currentColor" stroke-width="1" x="1.5" y="1.5" width="9" height="9"/></svg>
      </button>
      <button class="win-btn dock-right" onclick={() => dock('right')} title="Dock Right">
        <svg width="12" height="12" viewBox="0 0 12 12"><path fill="currentColor" d="M7 1h4v10H7V1zm1 1v8h2V2H8z"/></svg>
      </button>
      <button class="win-btn close" onclick={close} title="Close">
        <svg width="12" height="12" viewBox="0 0 12 12"><path fill="currentColor" d="M1.5 1.5l9 9m-9 0l9-9" stroke="currentColor" stroke-width="1.2"/></svg>
      </button>
    </div>
  </div>
{/if}

<header class="app-header" onmousedown={handleDrag} data-tauri-drag-region>
  <div class="header-left">
    <h1>MyTodos</h1>
  </div>

  <div class="header-right">
    <div class="quick-add-container">
      <button 
        class="add-btn" 
        class:active={uiStore.showQuickAdd}
        onclick={() => uiStore.toggleQuickAdd()} 
        title="Add new..."
      >
        <span class="plus-icon">＋</span>
      </button>

      {#if uiStore.showQuickAdd}
        <div class="quick-add-menu" transition:fly={{ y: 10, duration: 200 }}>
          <button 
            onclick={() => { uiStore.openTaskModal(); uiStore.closeQuickAdd(); }}
            class="menu-item"
          >
            <span class="menu-icon">✓</span>
            <div class="menu-text">
              <span class="menu-title">New Task</span>
              <span class="menu-desc">Add to current list</span>
            </div>
          </button>
          <button 
            onclick={() => { uiStore.openProjectModal(); uiStore.closeQuickAdd(); }}
            class="menu-item"
          >
            <span class="menu-icon">📁</span>
            <div class="menu-text">
              <span class="menu-title">New Project</span>
              <span class="menu-desc">Create a container</span>
            </div>
          </button>
        </div>
      {/if}
    </div>

    <div class="timer-badge" class:running={timerStore.isRunning} title="Total time today">
      <span class="timer-icon">📅</span>
      <TimeDisplay seconds={Math.floor(timerStore.dailyTotal)} format="hms" />
    </div>

    <button class="icon-btn" onclick={() => uiStore.toggleTheme()} title="Toggle theme">
      {uiStore.theme === "dark" ? "☀" : "🌙"}
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
    color: white;
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
  }

  .timer-badge.running {
    background-color: var(--success);
    color: white;
    border-color: var(--success);
    box-shadow: 0 0 10px var(--success-glow);
    animation: header-pulse 2s ease-in-out infinite;
  }

  @keyframes header-pulse {
    0%, 100% {
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
