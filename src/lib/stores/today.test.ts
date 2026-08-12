import { describe, expect, it, vi } from "vitest";

import type { CalendarEvent, TimeStats, TodayTaskSummary } from "$lib/services/db";
import {
  getTodayDateBoundaries,
  TodayLoader,
  type TodayDataSource,
} from "$lib/stores/today-loader";

const emptySummary: TodayTaskSummary = {
  overdue: [],
  today: [],
  completed_today: 0,
  total_today: 0,
};
const emptyStats: TimeStats = { today_tasks: [], week_daily: [], projects: [] };

function createSource(overrides: Partial<TodayDataSource> = {}): TodayDataSource {
  return {
    getTaskSummary: vi.fn().mockResolvedValue(emptySummary),
    getEvents: vi.fn().mockResolvedValue([] as CalendarEvent[]),
    getStats: vi.fn().mockResolvedValue(emptyStats),
    ...overrides,
  };
}

describe("getTodayDateBoundaries", () => {
  it("uses local calendar dates and handles month rollover", () => {
    expect(getTodayDateBoundaries(new Date(2026, 7, 31, 23, 45))).toEqual({
      todayStart: "2026-08-31",
      tomorrowStart: "2026-09-01",
    });
  });
});

describe("TodayLoader", () => {
  it("loads tasks, events, and stats concurrently with explicit boundaries", async () => {
    const source = createSource();
    const loader = new TodayLoader(source);

    const snapshot = await loader.load(new Date(2026, 7, 12, 14, 0));

    expect(source.getTaskSummary).toHaveBeenCalledWith("2026-08-12", "2026-08-13");
    expect(source.getEvents).toHaveBeenCalledWith("2026-08-12", "2026-08-12");
    expect(source.getStats).toHaveBeenCalledOnce();
    expect(snapshot?.date).toBe("2026-08-12");
  });

  it("ignores a stale response that finishes after a newer refresh", async () => {
    let resolveFirst!: (value: TodayTaskSummary) => void;
    const first = new Promise<TodayTaskSummary>((resolve) => {
      resolveFirst = resolve;
    });
    const newer: TodayTaskSummary = {
      overdue: [],
      completed_today: 0,
      total_today: 1,
      today: [{
        id: 2,
        title: "Newer",
        position: 0,
        total_time_seconds: 0,
        deadline: "2026-08-13",
      }],
    };
    const getTaskSummary = vi
      .fn()
      .mockReturnValueOnce(first)
      .mockResolvedValueOnce(newer);
    const loader = new TodayLoader(createSource({ getTaskSummary }));

    const oldRefresh = loader.load(new Date(2026, 7, 12));
    const latest = await loader.load(new Date(2026, 7, 13));
    resolveFirst(emptySummary);
    const stale = await oldRefresh;

    expect(stale).toBeNull();
    expect(latest?.date).toBe("2026-08-13");
    expect(latest?.taskSummary.today[0]?.title).toBe("Newer");
  });

  it("surfaces a source failure without manufacturing partial data", async () => {
    const source = createSource();
    const loader = new TodayLoader(source);
    vi.mocked(source.getStats).mockRejectedValueOnce(new Error("stats unavailable"));

    await expect(loader.load(new Date(2026, 7, 13))).rejects.toThrow("stats unavailable");
  });

  it("ignores a stale failure after a newer load has started", async () => {
    let rejectFirst!: (reason: Error) => void;
    const first = new Promise<TodayTaskSummary>((_resolve, reject) => {
      rejectFirst = reject;
    });
    const getTaskSummary = vi
      .fn()
      .mockReturnValueOnce(first)
      .mockResolvedValueOnce(emptySummary);
    const loader = new TodayLoader(createSource({ getTaskSummary }));

    const staleLoad = loader.load(new Date(2026, 7, 12));
    const latest = loader.load(new Date(2026, 7, 13));
    rejectFirst(new Error("stale failure"));

    await expect(staleLoad).resolves.toBeNull();
    await expect(latest).resolves.toMatchObject({ date: "2026-08-13" });
  });
});
