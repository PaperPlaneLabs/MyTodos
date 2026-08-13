import { describe, expect, it } from "vitest";
import type { CalendarItem } from "$lib/types/calendar";
import {
  addDays,
  getWeekStart,
  positionTimedItems,
  taskToCalendarItem,
} from "$lib/components/calendar/calendar-utils";

describe("calendar utilities", () => {
  it("supports configurable week boundaries", () => {
    const date = new Date("2026-08-13T12:00:00");
    expect(getWeekStart(date, "monday").getDay()).toBe(1);
    expect(getWeekStart(date, "sunday").getDay()).toBe(0);
  });

  it("moves across month boundaries using local dates", () => {
    expect(addDays("2026-01-31", 1)).toBe("2026-02-01");
  });

  it("normalizes timed tasks with their planned duration", () => {
    const item = taskToCalendarItem({
      id: 4,
      title: "Plan",
      deadline: "2026-08-13T09:30:00",
      planned_duration_minutes: 45,
      completed: false,
      position: 0,
      total_time_seconds: 0,
      created_at: 0,
      updated_at: 0,
    }, "#123456");
    expect(item?.startTime).toBe("09:30");
    expect(item?.endTime).toBe("10:15");
  });

  it("assigns overlapping items to separate columns", () => {
    const make = (key: string, startTime: string, endTime: string): CalendarItem => ({
      key,
      kind: "google_event",
      source: "google",
      title: key,
      description: null,
      color: "blue",
      isAllDay: false,
      startDate: "2026-08-13",
      endDate: "2026-08-14",
      startTime,
      endTime,
      startAt: null,
      endAt: null,
      readOnly: true,
      event: {
        external_id: key,
        title: key,
        description: null,
        is_all_day: false,
        start_date: "2026-08-13",
        end_date: "2026-08-14",
        start_at: null,
        end_at: null,
        timezone: null,
        html_link: null,
        color: "blue",
      },
    });
    const positioned = positionTimedItems([
      make("a", "09:00", "10:00"),
      make("b", "09:30", "10:30"),
      make("c", "11:00", "12:00"),
    ], 1);
    expect(positioned[0].columnCount).toBe(2);
    expect(positioned[1].columnCount).toBe(2);
    expect(positioned[2].columnCount).toBe(1);
  });
});
