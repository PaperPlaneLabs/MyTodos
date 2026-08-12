import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import TodayWorkspace from "$lib/components/today/TodayWorkspace.svelte";

const states = vi.hoisted(() => ({
  todayState: {
    taskSummary: {
      overdue: [] as Array<Record<string, unknown>>,
      today: [] as Array<Record<string, unknown>>,
      completed_today: 0,
      total_today: 0,
    },
    events: [] as Array<Record<string, unknown>>,
    stats: null,
    date: "2026-08-12",
    loading: false,
    error: null as string | null,
    refresh: vi.fn().mockResolvedValue(undefined),
  },
  uiState: {
    windowOrientation: "center",
    showTaskModal: false,
    openTaskModal: vi.fn(),
  },
  timerState: {
    active: null as Record<string, unknown> | null,
    isRunning: false,
    isTimed: false,
    timerLimit: null as number | null,
    remaining: 0,
    elapsed: 0,
    dailyTotal: 0,
    changeSignal: 0,
    pause: vi.fn(),
    resume: vi.fn(),
    stop: vi.fn(),
  },
  googleState: { connected: false },
}));

vi.mock("$lib/stores/today.svelte", () => ({ todayStore: states.todayState }));
vi.mock("$lib/stores/ui.svelte", () => ({ uiStore: states.uiState }));
vi.mock("$lib/stores/timer.svelte", () => ({ timerStore: states.timerState }));
vi.mock("$lib/stores/google-calendar.svelte", () => ({ googleCalendarStore: states.googleState }));

const props = {
  onCompleteTask: vi.fn(),
  onToggleTimer: vi.fn(),
  onTaskContextMenu: vi.fn(),
};

afterEach(() => {
  cleanup();
  document.body.classList.remove("compact-mode");
});

beforeEach(() => {
  states.todayState.taskSummary = {
    overdue: [],
    today: [],
    completed_today: 0,
    total_today: 0,
  };
  states.todayState.events = [];
  states.todayState.date = "2026-08-12";
  states.todayState.loading = false;
  states.todayState.error = null;
  states.todayState.refresh.mockClear();
  states.uiState.windowOrientation = "center";
  states.timerState.active = null;
  states.timerState.isRunning = false;
  states.timerState.dailyTotal = 0;
  states.googleState.connected = false;
  props.onCompleteTask.mockClear();
  props.onToggleTimer.mockClear();
  props.onTaskContextMenu.mockClear();
});

describe("TodayWorkspace", () => {
  it("renders the disconnected empty state in docked portrait mode", () => {
    states.uiState.windowOrientation = "left";

    const { container } = render(TodayWorkspace, { props });

    expect(container.querySelector("main")?.classList.contains("portrait")).toBe(true);
    expect(screen.getByText("Nothing due today")).toBeTruthy();
    expect(screen.getByText("Calendar not connected")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Add task" })).toBeTruthy();
  });

  it("renders overdue work, an active session, and progress", () => {
    states.todayState.taskSummary = {
      overdue: [{
        id: 7,
        project_id: 2,
        title: "Ship release",
        position: 0,
        total_time_seconds: 900,
        deadline: "2026-08-11T17:00:00",
        project_name: "Todoz",
        project_color: "#6366f1",
      }],
      today: [],
      completed_today: 2,
      total_today: 4,
    };
    states.timerState.active = { task_id: 7, task_title: "Ship release" };
    states.timerState.isRunning = true;
    states.timerState.elapsed = 120;
    states.timerState.dailyTotal = 3600;
    states.googleState.connected = true;
    states.todayState.events = [
      { id: 1, title: "Planning", description: null, date: "2026-08-12T09:00:00", is_all_day: false, color: "#6366f1" },
      { id: 2, title: "Holiday", description: null, date: "2026-08-12", is_all_day: true, color: "#22c55e" },
    ];

    render(TodayWorkspace, { props });

    expect(screen.getByText("Focusing now")).toBeTruthy();
    expect(screen.getAllByText("Ship release").length).toBeGreaterThan(0);
    expect(screen.getByText("Overdue")).toBeTruthy();
    expect(screen.getByText("2 of 4 tasks")).toBeTruthy();
    expect(screen.getByText("50%")).toBeTruthy();
    expect(screen.getAllByText("All day").length).toBeGreaterThan(0);
    expect(screen.getByText("Timeline")).toBeTruthy();
  });

  it("renders loading and retained-snapshot error states with retry", async () => {
    states.todayState.loading = true;
    states.todayState.date = "";
    const loading = render(TodayWorkspace, { props });
    expect(loading.container.querySelector('[aria-busy="true"]')).toBeTruthy();
    loading.unmount();

    states.todayState.loading = false;
    states.todayState.date = "2026-08-12";
    states.todayState.error = "Calendar unavailable";
    render(TodayWorkspace, { props });
    expect(screen.getByRole("alert").textContent).toContain("Showing the last available snapshot");
    await fireEvent.click(screen.getByRole("button", { name: "Try again" }));
    expect(states.todayState.refresh).toHaveBeenCalled();
  });

  it("keeps task actions keyboard-addressable in compact mode", async () => {
    document.body.classList.add("compact-mode");
    states.todayState.taskSummary = {
      overdue: [],
      today: [{
        id: 8,
        project_id: 2,
        title: "Review roadmap",
        position: 0,
        total_time_seconds: 0,
        deadline: "2026-08-12T10:00:00",
        project_name: "Todoz",
        project_color: "#6366f1",
      }],
      completed_today: 0,
      total_today: 1,
    };

    render(TodayWorkspace, { props });
    const complete = screen.getByRole("button", { name: "Mark Review roadmap complete" });
    const timer = screen.getByRole("button", { name: "Start timer for Review roadmap" });
    expect(complete.getAttribute("tabindex")).not.toBe("-1");
    expect(timer.getAttribute("tabindex")).not.toBe("-1");
    await fireEvent.click(complete);
    await fireEvent.click(timer);
    expect(props.onCompleteTask).toHaveBeenCalledWith(8);
    expect(props.onToggleTimer).toHaveBeenCalledWith(8);
  });
});
