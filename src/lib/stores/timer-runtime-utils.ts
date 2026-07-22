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
  if (
    activeTimer?.timer_limit_seconds !== undefined &&
    activeTimer.timer_remaining_seconds !== undefined
  ) {
    return Math.max(
      0,
      activeTimer.timer_limit_seconds - calculateTimerRemainingSeconds(activeTimer),
    );
  }

  if (!activeTimer?.is_running) {
    return elapsedOffset;
  }

  return elapsedOffset + (Date.now() / 1000 - activeTimer.started_at);
}

export function calculateTimerRemainingSeconds(
  activeTimer: ActiveTimer | null,
): number {
  if (
    activeTimer?.timer_limit_seconds === undefined ||
    activeTimer.timer_remaining_seconds === undefined
  ) {
    return 0;
  }

  if (!activeTimer.is_running || activeTimer.timer_expires_at === undefined) {
    return Math.max(0, activeTimer.timer_remaining_seconds);
  }

  return Math.max(0, activeTimer.timer_expires_at - Date.now() / 1000);
}
