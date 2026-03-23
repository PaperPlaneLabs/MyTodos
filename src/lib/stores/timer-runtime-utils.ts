import type { ActiveTimer } from "$lib/services/db";

export function calculateContinuousElapsedSeconds(
  activeTimer: ActiveTimer | null,
): number {
  if (!activeTimer?.is_running) {
    return 0;
  }

  return activeTimer.elapsed_seconds + (Date.now() / 1000 - activeTimer.started_at);
}

export function calculateDisplayElapsedSeconds(
  activeTimer: ActiveTimer | null,
  elapsedOffset: number,
): number {
  if (!activeTimer?.is_running) {
    return elapsedOffset;
  }

  return elapsedOffset + (Date.now() / 1000 - activeTimer.started_at);
}
