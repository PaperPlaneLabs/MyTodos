<script lang="ts">
  import { formatSecondsToTime, formatSecondsToHHMMSS } from "$lib/services/time-parser";

  let { seconds = 0, format = "short" }: { seconds: number; format?: "short" | "hms" } = $props();
</script>

<span class="time-display">
  {format === "short" ? formatSecondsToTime(seconds) : formatSecondsToHHMMSS(seconds)}
</span>

<style>
  .time-display {
    font-variant-numeric: tabular-nums;
    font-family: var(--font-mono);
    color: var(--text-secondary);
  }
</style>
