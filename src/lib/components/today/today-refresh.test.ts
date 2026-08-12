import { afterEach, describe, expect, it, vi } from "vitest";

import {
  millisecondsUntilNextLocalMidnight,
  scheduleLocalMidnightRefresh,
} from "$lib/components/today/today-refresh";

afterEach(() => vi.useRealTimers());

describe("Today refresh scheduling", () => {
  it("computes the next local midnight across a month boundary", () => {
    expect(millisecondsUntilNextLocalMidnight(new Date(2026, 7, 31, 23, 59, 30))).toBe(30_000);
  });

  it("refreshes at midnight and can be cancelled", () => {
    vi.useFakeTimers();
    const refresh = vi.fn();
    const cancel = scheduleLocalMidnightRefresh(
      refresh,
      () => new Date(2026, 7, 12, 23, 59, 59),
    );

    vi.advanceTimersByTime(1_000);
    expect(refresh).toHaveBeenCalledOnce();
    cancel();
    vi.advanceTimersByTime(1_000);
    expect(refresh).toHaveBeenCalledOnce();
  });
});
