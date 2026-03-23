import { beforeEach, describe, expect, it, vi } from "vitest";

import type { ActiveTimer } from "$lib/services/db";
import {
  calculateContinuousElapsedSeconds,
  calculateDisplayElapsedSeconds,
} from "./timer-runtime-utils";

describe("timer-runtime-utils", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-03-23T09:00:00.000Z"));
  });

  it("keeps the paused elapsed value until the timer resumes", () => {
    const pausedTimer: ActiveTimer = {
      task_id: 1,
      started_at: Math.floor(Date.now() / 1000) - 125,
      elapsed_seconds: 0,
      is_running: false,
    };

    expect(calculateDisplayElapsedSeconds(pausedTimer, 125)).toBe(125);
    expect(calculateContinuousElapsedSeconds(pausedTimer)).toBe(0);
  });

  it("adds resumed runtime on top of the preserved paused elapsed value", () => {
    const resumedTimer: ActiveTimer = {
      task_id: 1,
      started_at: Math.floor(Date.now() / 1000) - 3,
      elapsed_seconds: 0,
      is_running: true,
    };

    expect(Math.floor(calculateDisplayElapsedSeconds(resumedTimer, 125))).toBe(128);
    expect(Math.floor(calculateContinuousElapsedSeconds(resumedTimer))).toBe(3);
  });
});
