<script lang="ts">
  import { calendarStore } from '$lib/stores/calendar.svelte';
</script>

<button 
  class="icon-btn" 
  onclick={() => calendarStore.toggle()}
  title="Calendar"
>
  📅
</button>

<style>
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
