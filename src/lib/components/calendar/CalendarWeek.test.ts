import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import CalendarWeek from "$lib/components/calendar/CalendarWeek.svelte";
import type { CalendarDay, CalendarItem } from "$lib/types/calendar";

const {
  mockToggleTask,
  mockSelectItem,
  mockOpenNewEvent,
  mockScrollToDate,
  mockLoadMoreFutureWeeks,
  mockLoadMorePastWeeks,
  mockSetCurrentDateSilent,
  mockSetCurrentDate,
  mockClearScrollTarget,
  testTaskItem,
  testGoogleItem,
  mockWeekGroups,
} = vi.hoisted(() => {
  const toggle = vi.fn();
  const select = vi.fn();
  const openNew = vi.fn();
  const scrollTo = vi.fn();
  const loadFuture = vi.fn();
  const loadPast = vi.fn();
  const setCurrentSilent = vi.fn();
  const setCurrent = vi.fn();
  const clearTarget = vi.fn();

  const taskItem: CalendarItem = {
    key: "task:42",
    kind: "task",
    source: "tasks",
    title: "Complete quarterly report",
    description: "Draft finances and review with team",
    color: "#3b82f6",
    isAllDay: false,
    startDate: "2026-09-15",
    endDate: "2026-09-16",
    startTime: "14:00",
    endTime: "15:30",
    startAt: 1789470000,
    endAt: 1789475400,
    readOnly: false,
    task: {
      id: 42,
      title: "Complete quarterly report",
      description: "Draft finances and review with team",
      completed: false,
      project_id: 1,
      position: 0,
      total_time_seconds: 0,
      planned_duration_minutes: 90,
      created_at: 0,
      updated_at: 0,
      deadline: "2026-09-15T14:00:00",
    },
  };

  const googleItem: CalendarItem = {
    key: "google:g1",
    kind: "google_event",
    source: "google",
    title: "Team Sync",
    description: "Weekly sync meeting",
    color: "#4285f4",
    isAllDay: false,
    startDate: "2026-09-16",
    endDate: "2026-09-17",
    startTime: "10:00",
    endTime: "11:00",
    startAt: 1789542000,
    endAt: 1789545600,
    readOnly: true,
    event: {
      external_id: "g1",
      title: "Team Sync",
      description: "Weekly sync meeting",
      is_all_day: false,
      start_date: "2026-09-16",
      end_date: "2026-09-17",
      start_at: 1789542000,
      end_at: 1789545600,
      timezone: "UTC",
      html_link: null,
      color: "#4285f4",
    },
  };

  const weekGroups = [
    {
      weekStart: new Date(2026, 8, 14),
      weekKey: "2026-09-14",
      days: [
        {
          date: new Date(2026, 8, 14),
          dateKey: "2026-09-14",
          isCurrentMonth: true,
          isToday: false,
          isSelected: false,
          items: [],
        },
        {
          date: new Date(2026, 8, 15),
          dateKey: "2026-09-15",
          isCurrentMonth: true,
          isToday: true,
          isSelected: false,
          items: [taskItem],
        },
        {
          date: new Date(2026, 8, 16),
          dateKey: "2026-09-16",
          isCurrentMonth: true,
          isToday: false,
          isSelected: false,
          items: [googleItem],
        },
        {
          date: new Date(2026, 8, 17),
          dateKey: "2026-09-17",
          isCurrentMonth: true,
          isToday: false,
          isSelected: false,
          items: [],
        },
      ] as CalendarDay[],
    },
  ];

  return {
    mockToggleTask: toggle,
    mockSelectItem: select,
    mockOpenNewEvent: openNew,
    mockScrollToDate: scrollTo,
    mockLoadMoreFutureWeeks: loadFuture,
    mockLoadMorePastWeeks: loadPast,
    mockSetCurrentDateSilent: setCurrentSilent,
    mockSetCurrentDate: setCurrent,
    mockClearScrollTarget: clearTarget,
    testTaskItem: taskItem,
    testGoogleItem: googleItem,
    mockWeekGroups: weekGroups,
  };
});

vi.mock("$lib/stores/calendar.svelte", () => ({
  calendarStore: {
    get currentDate() { return new Date(2026, 8, 15); },
    get selectedDate() { return null; },
    get selectedItemKey() { return null; },
    get scrollTarget() { return null; },
    generateMultiWeekGroups: () => mockWeekGroups,
    generateWeekDays: () => [
      { date: new Date(2026, 8, 14), dayName: "Mon" },
      { date: new Date(2026, 8, 15), dayName: "Tue" },
      { date: new Date(2026, 8, 16), dayName: "Wed" },
      { date: new Date(2026, 8, 17), dayName: "Thu" },
      { date: new Date(2026, 8, 18), dayName: "Fri" },
      { date: new Date(2026, 8, 19), dayName: "Sat" },
      { date: new Date(2026, 8, 20), dayName: "Sun" },
    ],
    getItemsForDate: (dateKey: string) => {
      if (dateKey === "2026-09-15") return [testTaskItem];
      if (dateKey === "2026-09-16") return [testGoogleItem];
      return [];
    },
    toggleTask: (...args: unknown[]) => mockToggleTask(...args),
    selectItem: (...args: unknown[]) => mockSelectItem(...args),
    openNewEvent: (...args: unknown[]) => mockOpenNewEvent(...args),
    scrollToDate: (...args: unknown[]) => mockScrollToDate(...args),
    clearScrollTarget: () => mockClearScrollTarget(),
    loadMoreFutureWeeks: (...args: unknown[]) => mockLoadMoreFutureWeeks(...args),
    loadMorePastWeeks: (...args: unknown[]) => mockLoadMorePastWeeks(...args),
    setCurrentDateSilent: (...args: unknown[]) => mockSetCurrentDateSilent(...args),
    setCurrentDate: (...args: unknown[]) => mockSetCurrentDate(...args),
    allItems: [testTaskItem, testGoogleItem],
  },
}));

vi.mock("$lib/stores/projects.svelte", () => ({
  projectStore: {
    projects: [
      { id: 1, name: "Finance", color: "#3b82f6" },
    ],
  },
}));

vi.mock("$lib/stores/ui.svelte", () => ({
  uiStore: {
    windowOrientation: "left",
  },
}));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("CalendarWeek component (Google Calendar widget style)", () => {
  it("renders the sticky week strip with weekdays and numbers", () => {
    render(CalendarWeek);

    expect(screen.getByRole("button", { name: /Mon 14/i })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Tue 15/i })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Wed 16/i })).toBeTruthy();
  });

  it("renders task cards with title, project badge, and checkbox", () => {
    render(CalendarWeek);

    expect(screen.getByText("Complete quarterly report")).toBeTruthy();
    expect(screen.getByText("Finance")).toBeTruthy();
    expect(screen.getByText("2:00 PM – 3:30 PM")).toBeTruthy();
    expect(screen.getByText("90m")).toBeTruthy();
  });

  it("clicking the task checkbox toggles task completion", async () => {
    render(CalendarWeek);

    const checkBtn = screen.getByRole("button", { name: /Mark active|Mark complete/i });
    expect(checkBtn).toBeTruthy();
    await fireEvent.click(checkBtn);

    expect(mockToggleTask).toHaveBeenCalledWith(42);
  });

  it("renders Google Calendar event cards with Google badge", () => {
    render(CalendarWeek);

    expect(screen.getByText("Team Sync")).toBeTruthy();
    expect(screen.getByText("Google")).toBeTruthy();
    expect(screen.getByText("10:00 AM – 11:00 AM")).toBeTruthy();
  });

  it("renders empty day placeholders with + Add button", async () => {
    render(CalendarWeek);

    const emptyCards = screen.getAllByText("No tasks or events");
    expect(emptyCards.length).toBeGreaterThan(0);

    const addButtons = screen.getAllByRole("button", { name: /\+ Add/i });
    await fireEvent.click(addButtons[0]);

    expect(mockOpenNewEvent).toHaveBeenCalledWith("2026-09-14", null);
  });

  it("clicking a card selects it in the calendar inspector", async () => {
    render(CalendarWeek);

    const taskTitle = screen.getByText("Complete quarterly report");
    await fireEvent.click(taskTitle);

    expect(mockSelectItem).toHaveBeenCalledWith(testTaskItem);
  });
});
