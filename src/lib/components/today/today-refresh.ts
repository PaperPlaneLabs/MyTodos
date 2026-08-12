export function millisecondsUntilNextLocalMidnight(now: Date = new Date()): number {
  const next = new Date(now);
  next.setHours(24, 0, 0, 0);
  return Math.max(1, next.getTime() - now.getTime());
}

export function scheduleLocalMidnightRefresh(
  refresh: () => void,
  now: () => Date = () => new Date(),
): () => void {
  let timeoutId: ReturnType<typeof setTimeout> | null = null;
  let cancelled = false;

  const schedule = () => {
    timeoutId = setTimeout(() => {
      if (cancelled) return;
      refresh();
      schedule();
    }, millisecondsUntilNextLocalMidnight(now()));
  };

  schedule();
  return () => {
    cancelled = true;
    if (timeoutId !== null) clearTimeout(timeoutId);
  };
}
