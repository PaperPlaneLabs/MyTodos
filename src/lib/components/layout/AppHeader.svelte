<script lang="ts">
  import { uiStore } from "$lib/stores/ui.svelte";
  import { timerStore } from "$lib/stores/timer.svelte";
  import TimeDisplay from "$lib/components/common/TimeDisplay.svelte";
  import { fly } from "svelte/transition";

  let elapsed = $derived(Math.floor(timerStore.elapsed));
</script>

<header class="app-header">
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

    {#if timerStore.active}
      <div class="timer-badge" class:running={timerStore.isRunning}>
        <span class="timer-icon">⏱</span>
        <TimeDisplay seconds={elapsed} format="hms" />
      </div>
    {/if}

    <button class="icon-btn" onclick={() => uiStore.toggleTheme()} title="Toggle theme">
      {uiStore.theme === "dark" ? "☀" : "🌙"}
    </button>
  </div>
</header>

<style>
  .app-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-md);
    border-bottom: 1px solid var(--border);
    background-color: var(--bg-primary);
    flex-shrink: 0;
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
