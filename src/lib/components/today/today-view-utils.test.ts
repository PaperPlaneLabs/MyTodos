import { describe, expect, it } from "vitest";

import {
  buildTodayAgenda,
  formatEventTime,
  formatTodayDeadline,
  getTodayProgress,
  groupUpcomingTasks,
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

  it("formats all-day and timed events", () => {
    expect(formatEventTime({ id: 2, title: "Holiday", description: null, date: "2026-08-12", is_all_day: true, color: "" })).toBe("All day");
    expect(formatEventTime({ id: 3, title: "Earlier", description: null, date: "2026-08-12T09:00:00", is_all_day: false, color: "" })).toBe("09:00");
  });

  it("builds all-day, anytime, and deterministically interleaved agenda groups", () => {
    const task = (id: number, title: string, deadline: string) => ({ id, title, deadline, position: 0, total_time_seconds: 0 });
    const agenda = buildTodayAgenda(
      [task(3, "Zulu task", "2026-08-12T09:00:00"), task(2, "Any task", "2026-08-12")],
      [
        { id: 4, title: "Zulu event", description: null, date: "2026-08-12T09:00:00", is_all_day: false, color: "" },
        { id: 1, title: "Holiday", description: null, date: "2026-08-12", is_all_day: true, color: "" },
      ],
    );

    expect(agenda.allDay.map((item) => item.title)).toEqual(["Holiday"]);
    expect(agenda.anytime.map((item) => item.title)).toEqual(["Any task"]);
    expect(agenda.timeline.map((item) => `${item.kind}:${item.title}`)).toEqual([
      "event:Zulu event", "task:Zulu task",
    ]);
  });

  it("groups upcoming tasks by local deadline date with rolling-day labels", () => {
    const task = (id: number, title: string, deadline: string) => ({ id, title, deadline, position: 0, total_time_seconds: 0 });

    expect(groupUpcomingTasks([
      task(1, "Plan", "2026-08-13"),
      task(2, "Review", "2026-08-13T15:00:00"),
      task(3, "Ship", "2026-08-16"),
    ], "2026-08-12")).toEqual([
      { date: "2026-08-13", label: "Tomorrow", tasks: [task(1, "Plan", "2026-08-13"), task(2, "Review", "2026-08-13T15:00:00")] },
      { date: "2026-08-16", label: "Sunday, Aug 16", tasks: [task(3, "Ship", "2026-08-16")] },
    ]);
  });
});
