<script lang="ts">
  import TimeDisplay from "$lib/components/common/TimeDisplay.svelte";
  import { timerStore } from "$lib/stores/timer.svelte";

  const progress = $derived(
    timerStore.timerLimit
      ? Math.min(100, Math.max(0, ((timerStore.timerLimit - timerStore.remaining) / timerStore.timerLimit) * 100))
      : 0,
  );

  async function toggleTimer() {
    if (timerStore.isRunning) {
      await timerStore.pause();
    } else {
      await timerStore.resume();
    }
  }
</script>

{#if timerStore.active}
  <section class="focus-card" aria-labelledby="focus-card-title">
    <div class="focus-copy">
      <div class="focus-label">
        <span class:running={timerStore.isRunning} class="status-dot"></span>
        {timerStore.isRunning ? "Focusing now" : "Session paused"}
      </div>
      <h3 id="focus-card-title">{timerStore.active.task_title ?? "Current task"}</h3>
      <div class="focus-clock">
        <TimeDisplay
          seconds={Math.floor(timerStore.isTimed ? timerStore.remaining : timerStore.elapsed)}
          format="hms"
        />
        {#if timerStore.isTimed}<small>remaining</small>{/if}
      </div>
      {#if timerStore.isTimed}
        <div class="focus-progress" aria-label={`${Math.round(progress)}% complete`}>
          <span style:width={`${progress}%`}></span>
        </div>
      {/if}
    </div>
    <div class="focus-actions">
      <button type="button" class="btn btn-sm btn-secondary" onclick={toggleTimer}>
        {timerStore.isRunning ? "Pause" : "Resume"}
      </button>
      <button type="button" class="btn btn-sm btn-danger" onclick={() => timerStore.stop()}>
        Stop
      </button>
    </div>
  </section>
{/if}

<style>
  .focus-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-lg);
    padding: var(--spacing-lg);
    border: 1px solid var(--success);
    border-radius: var(--radius-lg);
    background: linear-gradient(135deg, var(--success-light), var(--bg-secondary));
    box-shadow: 0 8px 24px var(--success-glow);
  }
  .focus-copy { min-width: 0; flex: 1; }
  .focus-label { display: flex; align-items: center; gap: var(--spacing-xs); color: var(--text-secondary); font-size: var(--text-xs); font-weight: 700; text-transform: uppercase; letter-spacing: .06em; }
  .status-dot { width: 8px; height: 8px; border-radius: 50%; background: var(--warning); }
  .status-dot.running { background: var(--success); box-shadow: 0 0 0 4px var(--success-glow); }
  h3 { margin: var(--spacing-xs) 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 18px; }
  .focus-clock { display: flex; align-items: baseline; gap: var(--spacing-xs); font-family: var(--font-mono); font-size: 26px; font-weight: 700; }
  .focus-clock small { color: var(--text-tertiary); font-family: var(--font-sans); font-size: 10px; text-transform: uppercase; }
  .focus-progress { max-width: 320px; height: 5px; margin-top: var(--spacing-sm); overflow: hidden; border-radius: 999px; background: var(--bg-tertiary); }
  .focus-progress span { display: block; height: 100%; border-radius: inherit; background: var(--success); transition: width .3s linear; }
  .focus-actions { display: flex; gap: var(--spacing-sm); flex: none; }
  @media (max-width: 520px) { .focus-card { align-items: stretch; flex-direction: column; } .focus-actions button { flex: 1; } }
</style>
