<script lang="ts">
  import { uiStore } from "$lib/stores/ui.svelte";

  let isPortrait = $derived(
    uiStore.windowOrientation === "left" ||
      uiStore.windowOrientation === "right",
  );

  // Generate a generic grid (5 weeks, 7 days)
  const weeks = Array(5).fill(0);
  const days = Array(7).fill(0);
</script>

<div class="calendar-skeleton" class:portrait={isPortrait}>
  <div class="skeleton-header">
    {#each days as _}
      <div class="pulse-block weekday-block"></div>
    {/each}
  </div>

  <div class="skeleton-grid">
    {#each weeks as _}
      {#each days as _}
        <div class="skeleton-cell">
          <div class="pulse-block day-number-block"></div>
          <div class="pulse-block task-chip-block w-80"></div>
          <div class="pulse-block task-chip-block w-60"></div>
        </div>
      {/each}
    {/each}
  </div>
</div>

<style>
  .calendar-skeleton {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: var(--spacing-md);
    background: var(--bg-primary);
  }

  .skeleton-header {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-md);
    padding: 0 var(--spacing-sm);
  }

  .skeleton-grid {
    flex: 1;
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    grid-auto-rows: minmax(100px, 1fr);
    gap: 1px;
    background: var(--border-light);
    border: 1px solid var(--border-light);
  }

  .portrait .skeleton-grid {
    grid-auto-rows: minmax(80px, 1fr);
  }

  .skeleton-cell {
    background: var(--bg-primary);
    padding: var(--spacing-sm);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  /* Pulse animation */
  @keyframes pulse {
    0% { opacity: 0.6; }
    50% { opacity: 0.2; }
    100% { opacity: 0.6; }
  }

  .pulse-block {
    background: var(--text-tertiary);
    border-radius: var(--radius-sm);
    animation: pulse 1.5s ease-in-out infinite;
  }

  .weekday-block {
    height: 14px;
    width: 60%;
    margin: 0 auto;
  }

  .day-number-block {
    height: 24px;
    width: 24px;
    border-radius: 50%;
    margin-bottom: var(--spacing-xs);
  }

  .task-chip-block {
    height: 14px;
  }

  .w-80 { width: 80%; }
  .w-60 { width: 60%; }
</style>
