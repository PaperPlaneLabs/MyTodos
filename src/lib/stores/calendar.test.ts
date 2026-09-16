import { beforeEach, describe, expect, it, vi } from "vitest";

const mockedDb = vi.hoisted(() => ({
  getByDeadlineRange: vi.fn(),
  getInRange: vi.fn(),
  getWithTasks: vi.fn(),
  getEventsInRange: vi.fn(),
}));

vi.mock("$lib/services/db", () => ({
  db: {
    tasks: { getByDeadlineRange: mockedDb.getByDeadlineRange },
    calendarEvents: { getInRange: mockedDb.getInRange },
    timeEntries: { getWithTasks: mockedDb.getWithTasks },
    googleCalendar: { getEventsInRange: mockedDb.getEventsInRange },
  },
}));

vi.mock("$lib/stores/projects.svelte", () => ({
  projectStore: {
    projects: [],
  },
}));

import { calendarStore } from "$lib/stores/calendar.svelte";

beforeEach(() => {
  mockedDb.getByDeadlineRange.mockResolvedValue([]);
  mockedDb.getInRange.mockResolvedValue([]);
  mockedDb.getWithTasks.mockResolvedValue([]);
  mockedDb.getEventsInRange.mockResolvedValue({ events: [], stale: false, error: null });
});

describe("calendarStore multi-week buffer logic", () => {
  it("computes multi-week buffer range in week view mode", () => {
    calendarStore.setViewMode("week");
    calendarStore.setCurrentDate(new Date("2026-09-15T12:00:00"), false);

    const range = calendarStore.getVisibleRange();
    expect(range.startDate <= "2026-09-01").toBe(true);
    expect(range.endDate >= "2026-10-15").toBe(true);
    expect(range.rangeKey).toContain("week:monday");
  });

  it("expands future and past week buffers dynamically", async () => {
    calendarStore.setViewMode("week");
    calendarStore.setCurrentDate(new Date("2026-09-15T12:00:00"), false);

    const initialStart = calendarStore.weekBufferStart.getTime();
    const initialEnd = calendarStore.weekBufferEnd.getTime();

    await calendarStore.loadMoreFutureWeeks(4);
    expect(calendarStore.weekBufferEnd.getTime()).toBeGreaterThan(initialEnd);

    await calendarStore.loadMorePastWeeks(2);
    expect(calendarStore.weekBufferStart.getTime()).toBeLessThan(initialStart);
  });

  it("updates scrollTarget when scrollToDate is called", () => {
    const target = new Date("2026-09-20T10:00:00");
    calendarStore.scrollToDate(target, "smooth");

    expect(calendarStore.scrollTarget?.date.getDate()).toBe(20);
    expect(calendarStore.scrollTarget?.behavior).toBe("smooth");

    calendarStore.clearScrollTarget();
    expect(calendarStore.scrollTarget).toBeNull();
  });

  it("generates consecutive multi-week groups with 7 days per week", () => {
    calendarStore.setViewMode("week");
    calendarStore.setCurrentDate(new Date("2026-09-15T12:00:00"), false);

    const groups = calendarStore.generateMultiWeekGroups();
    expect(groups.length).toBeGreaterThanOrEqual(8);

    for (const week of groups) {
      expect(week.days).toHaveLength(7);
      expect(week.weekKey).toMatch(/^\d{4}-\d{2}-\d{2}$/);
    }
  });

  it("setCurrentDateSilent updates currentDate without resetting buffer when within range", () => {
    calendarStore.setViewMode("week");
    calendarStore.setCurrentDate(new Date("2026-09-15T12:00:00"), false);

    const prevBufferStart = calendarStore.weekBufferStart.getTime();
    calendarStore.setCurrentDateSilent(new Date("2026-09-22T12:00:00"));

    expect(calendarStore.currentDate.getDate()).toBe(22);
    expect(calendarStore.weekBufferStart.getTime()).toBe(prevBufferStart);
  });
});

