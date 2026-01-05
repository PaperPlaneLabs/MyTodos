<script lang="ts">
  import { uiStore } from "$lib/stores/ui.svelte";
  import { timerStore } from "$lib/stores/timer.svelte";
  import TimeDisplay from "$lib/components/common/TimeDisplay.svelte";

  let elapsed = $derived(Math.floor(timerStore.elapsed));
</script>

<header class="app-header">
  <div class="header-left">
    <h1>MyTodos</h1>
  </div>

  <div class="header-right">
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
