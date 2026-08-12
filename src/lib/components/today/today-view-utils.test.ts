import { describe, expect, it } from "vitest";

import {
  formatEventTime,
  formatTodayDeadline,
  getTodayProgress,
  sortTodayEvents,
} from "$lib/components/today/today-view-utils";

describe("Today view utilities", () => {
  it("calculates bounded completion progress", () => {
    expect(getTodayProgress(3, 4)).toBe(75);
    expect(getTodayProgress(0, 0)).toBe(0);
    expect(getTodayProgress(5, 4)).toBe(100);
  });

  it("formats only deadlines with a usable time", () => {
    expect(formatTodayDeadline("2026-08-12T09:30:00")).toBe("09:30");
    expect(formatTodayDeadline("2026-08-12")).toBeNull();
  });

  it("places all-day events first and then orders timed events", () => {
    const events = sortTodayEvents([
      { id: 1, title: "Later", description: null, date: "2026-08-12T15:00:00", is_all_day: false, color: "" },
      { id: 2, title: "Holiday", description: null, date: "2026-08-12", is_all_day: true, color: "" },
      { id: 3, title: "Earlier", description: null, date: "2026-08-12T09:00:00", is_all_day: false, color: "" },
    ]);

    expect(events.map((event) => event.title)).toEqual(["Holiday", "Earlier", "Later"]);
    expect(formatEventTime(events[0])).toBe("All day");
    expect(formatEventTime(events[1])).toBe("09:00");
  });
});
