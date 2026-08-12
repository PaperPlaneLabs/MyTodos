<script lang="ts">
  import TimeDisplay from "$lib/components/common/TimeDisplay.svelte";
  import { timerStore } from "$lib/stores/timer.svelte";

  import { getTodayProgress } from "./today-view-utils";

  let { completed, total }: { completed: number; total: number } = $props();
  const progress = $derived(getTodayProgress(completed, total));
</script>

<section class="progress-card" aria-labelledby="progress-title">
  <div class="section-heading">
    <div>
      <p>Daily progress</p>
      <h3 id="progress-title">{completed} of {total} tasks</h3>
    </div>
    <strong>{Math.min(100, progress)}%</strong>
  </div>
  <div class="progress-track" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={Math.min(100, progress)}>
    <span style:width={`${Math.min(100, progress)}%`}></span>
  </div>
  <div class="focus-total">
    <span>Focused today</span>
    <TimeDisplay seconds={Math.floor(timerStore.dailyTotal)} format="hm" />
  </div>
</section>

<style>
  .progress-card { padding: var(--spacing-md); border: 1px solid var(--border); border-radius: var(--radius-lg); background: var(--bg-secondary); }
  .section-heading, .focus-total { display: flex; align-items: center; justify-content: space-between; gap: var(--spacing-md); }
  p, h3 { margin: 0; }
  p { color: var(--text-tertiary); font-size: var(--text-xs); text-transform: uppercase; letter-spacing: .06em; }
  h3 { margin-top: 2px; font-size: var(--text-base); }
  .section-heading > strong { color: var(--accent); font-size: 22px; }
  .progress-track { height: 7px; margin: var(--spacing-md) 0; overflow: hidden; border-radius: 999px; background: var(--bg-tertiary); }
  .progress-track span { display: block; height: 100%; border-radius: inherit; background: var(--accent); transition: width var(--transition-normal); }
  .focus-total { padding-top: var(--spacing-sm); border-top: 1px solid var(--border-light); color: var(--text-secondary); font-size: var(--text-sm); }
</style>
