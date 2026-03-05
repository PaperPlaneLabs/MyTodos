<script lang="ts">
  import {
    formatSecondsToTime,
    formatSecondsToHHMMSS,
    formatSecondsToHM,
  } from "$lib/services/time-parser";

  let {
    seconds = 0,
    format = "short",
  }: { seconds: number; format?: "short" | "hms" | "hm" } = $props();
</script>

<span class="time-display">
  {#if format === "short"}
    {formatSecondsToTime(seconds)}
  {:else if format === "hms"}
    {formatSecondsToHHMMSS(seconds)}
  {:else if format === "hm"}
    {formatSecondsToHM(seconds)}
  {/if}
</span>

<style>
  .time-display {
    font-variant-numeric: tabular-nums;
    font-family: var(--font-mono);
    color: var(--text-secondary);
  }
</style>
