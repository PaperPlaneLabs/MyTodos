<script lang="ts">
  import { calendarStore } from '$lib/stores/calendar.svelte';
  import CalendarMonth from './CalendarMonth.svelte';
  import CalendarWeek from './CalendarWeek.svelte';
  import CalendarDay from './CalendarDay.svelte';
  import { fade, scale } from 'svelte/transition';
</script>

{#if calendarStore.isOpen}
  <div 
    class="calendar-overlay" 
    onclick={() => calendarStore.close()}
    transition:fade={{ duration: 200 }}
    role="button"
    tabindex="-1"
  >
    <div 
      class="calendar-panel"
      onclick={(e) => e.stopPropagation()}
      transition:scale={{ duration: 200, start: 0.95 }}
      role="dialog"
      aria-modal="true"
    >
      {#if calendarStore.viewMode === 'month'}
        <CalendarMonth />
      {:else if calendarStore.viewMode === 'week'}
        <CalendarWeek />
      {:else}
        <CalendarDay />
      {/if}
    </div>
  </div>
{/if}

<style>
  .calendar-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: var(--spacing-md);
  }
  
  .calendar-panel {
    background: var(--bg-primary);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-xl);
    width: 100%;
    max-width: 800px;
    max-height: 90vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  
  @media (min-width: 768px) {
    .calendar-panel {
      max-width: 900px;
    }
  }
</style>
