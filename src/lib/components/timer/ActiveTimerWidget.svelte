<script lang="ts">
  import { fly } from "svelte/transition";
  import TimeDisplay from "$lib/components/common/TimeDisplay.svelte";
  import { timerStore } from "$lib/stores/timer.svelte";

  let { onStop }: { onStop: () => void | Promise<void> } = $props();

  function getAutoPauseLabel(): string {
    if (timerStore.autoPausedReason === "SystemSleep") {
      return "system sleep";
    }
    if (timerStore.autoPausedReason === "ScreenLock") {
      return "screen lock";
    }
    return "shutdown";
  }
</script>

{#if timerStore.active}
  <div class="timer-widget" transition:fly={{ y: 50, duration: 300 }}>
    {#if timerStore.isAutoPaused}
      <div class="auto-pause-banner">
        <span class="icon">⏸️</span>
        <span>Timer auto-paused due to {getAutoPauseLabel()}</span>
      </div>
    {/if}
    <div class="timer-content">
      <div class="timer-info">
        <div class="timer-task-name">
          {timerStore.active.task_title || "Task"}
        </div>
        <div class="timer-elapsed">
          <TimeDisplay seconds={Math.floor(timerStore.elapsed)} format="hms" />
        </div>
      </div>
      <div class="timer-controls">
        {#if timerStore.isRunning}
          <button class="btn btn-sm btn-secondary" onclick={() => timerStore.pause()}>
            Pause
          </button>
        {:else}
          <button class="btn btn-sm btn-primary" onclick={() => timerStore.resume()}>
            Resume
          </button>
        {/if}
        <button class="btn btn-sm btn-danger" onclick={onStop}>Stop</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .timer-widget {
    flex-shrink: 0;
    padding: var(--spacing-md);
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-top: 3px solid var(--success);
    border-radius: var(--radius-lg) var(--radius-lg) 0 0;
    display: flex;
    flex-direction: column;
    gap: 0;
    box-shadow: 0 -4px 20px rgba(0, 0, 0, 0.08);
    margin: 0 var(--spacing-sm);
    z-index: 100;
  }

  :global([data-theme="dark"]) .timer-widget {
    background: var(--bg-secondary);
    box-shadow: 0 -4px 20px rgba(0, 0, 0, 0.4);
  }

  .auto-pause-banner {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-xs) var(--spacing-sm);
    background-color: var(--warning-light);
    border: 1px solid var(--warning);
    border-radius: var(--radius-md);
    font-size: 12px;
    color: var(--text-primary);
    margin-bottom: var(--spacing-sm);
  }

  .auto-pause-banner .icon {
    font-size: 14px;
    flex-shrink: 0;
  }

  :global([data-theme="dark"]) .auto-pause-banner {
    background-color: rgba(251, 191, 36, 0.15);
    border-color: var(--warning);
  }

  .timer-content {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--spacing-md);
  }

  .timer-info {
    flex: 1;
    min-width: 0;
  }

  .timer-task-name {
    font-weight: 600;
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--text-primary);
    margin-bottom: 2px;
  }

  .timer-elapsed {
    font-family: var(--font-mono);
    font-size: 20px;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: 0.5px;
  }

  .timer-controls {
    display: flex;
    gap: var(--spacing-sm);
  }
</style>
