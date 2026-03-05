import { invoke } from "@tauri-apps/api/core";

export interface Project {
  id: number;
  name: string;
  description?: string;
  color: string;
  position: number;
  total_time_seconds: number;
  created_at: number;
  updated_at: number;
}

export interface Section {
  id: number;
  project_id: number;
  name: string;
  position: number;
  total_time_seconds: number;
  created_at: number;
}

export interface Task {
  id: number;
  project_id?: number;
  section_id?: number;
  title: string;
  description?: string;
  completed: boolean;
  position: number;
  total_time_seconds: number;
  deadline?: string | null;
  google_event_id?: string | null;
  created_at: number;
  updated_at: number;
}

export interface TimeEntry {
  id: number;
  task_id: number;
  entry_type: string;
  duration_seconds: number;
  started_at?: number;
  ended_at?: number;
  note?: string;
  created_at: number;
}

export interface ActiveTimer {
  task_id: number;
  started_at: number;
  elapsed_seconds: number;
  is_running: boolean;
  task_title?: string;
  project_id?: number;
}

export enum AutoPauseReason {
  SystemSleep = "SystemSleep",
  ScreenLock = "ScreenLock",
  Shutdown = "Shutdown"
}

export interface AutoPauseEvent {
  reason: AutoPauseReason;
  timestamp: number;
}

export interface WindowState {
  x?: number;
  y?: number;
  width: number;
  height: number;
  updated_at: number;
}

export interface ProjectStats {
  task_count: number;
  completed_count: number;
  total_time_seconds: number;
}

export interface TaskTimeEntry {
  task_id: number;
  task_title: string;
  project_name: string | null;
  project_color: string | null;
  total_seconds: number;
}

export interface DailyAggregate {
  date: string;
  total_seconds: number;
}

export interface ProjectTime {
  name: string;
  color: string;
  total_seconds: number;
}

export interface TimeStats {
  today_tasks: TaskTimeEntry[];
  week_daily: DailyAggregate[];
  projects: ProjectTime[];
}

export interface CalendarEvent {
  id: number;
  title: string;
  description: string | null;
  date: string;
  is_all_day: boolean;
  color: string;
}

export interface TimeEntryWithTask {
  id: number;
  task_id: number;
  task_title: string;
  project_id: number | null;
  project_name: string | null;
  project_color: string | null;
  duration_seconds: number;
  started_at: number;
  ended_at: number;
  note: string | null;
}

export interface WindowOrientation {
  side: string;
  is_portrait: boolean;
  width: number;
  height: number;
}

export const db = {
  projects: {
    getAll: () => invoke<Project[]>("get_all_projects"),
    create: (name: string, description?: string, color?: string) =>
      invoke<Project>("create_project", { name, description, color }),
    update: (id: number, name?: string, description?: string, color?: string) =>
      invoke<void>("update_project", { id, name, description, color }),
    delete: (id: number) => invoke<void>("delete_project", { id }),
    reorder: (projectIds: number[]) => invoke<void>("reorder_projects", { projectIds }),
  },

  tasks: {
    getByProject: (projectId: number) => invoke<Task[]>("get_tasks_by_project", { projectId }),
    getUnassigned: () => invoke<Task[]>("get_unassigned_tasks"),
    getByDeadlineRange: (startDate: string, endDate: string) =>
      invoke<Task[]>("get_tasks_by_deadline_range", { startDate, endDate }),
    create: (projectId: number | null, sectionId: number | null, title: string, description?: string) =>
      invoke<Task>("create_task", { projectId, sectionId, title, description }),
    update: (id: number, title?: string, description?: string, completed?: boolean) =>
      invoke<void>("update_task", { id, title, description, completed }),
    updateDeadline: (id: number, deadline: string | null) =>
      invoke<void>("update_task_deadline", { taskId: id, deadline }),
    delete: (id: number) => invoke<void>("delete_task", { id }),
    toggleCompletion: (id: number) => invoke<boolean>("toggle_task_completion", { id }),
    reorder: (taskIds: number[]) => invoke<void>("reorder_tasks", { taskIds }),
    resetTime: (id: number) => invoke<void>("reset_task_time", { id }),
  },

  calendarEvents: {
    getInRange: (startDate: string, endDate: string) =>
      invoke<CalendarEvent[]>("get_calendar_events_in_range", { startDate, endDate }),
  },

  timer: {
    getActive: () => invoke<ActiveTimer | null>("get_active_timer"),
    start: (taskId: number) => invoke<ActiveTimer>("start_timer", { taskId }),
    pause: () => invoke<void>("pause_timer"),
    resume: () => invoke<void>("resume_timer"),
    stop: () => invoke<TimeEntry>("stop_timer"),
    reset: () => invoke<void>("reset_timer"),
  },

  timeEntries: {
    getWithTasks: (startDate: string, endDate: string) =>
      invoke<TimeEntryWithTask[]>("get_time_entries_with_tasks", { startDate, endDate }),
    delete: (id: number) => invoke<void>("delete_time_entry", { id }),
    getDailyTotalTime: (startTimestamp: number) =>
      invoke<number>("get_daily_total_time", { startTimestamp }),
    getTimeStats: (includeActiveTimer: boolean = true) =>
      invoke<TimeStats>("get_time_stats", { includeActiveTimer }),
    logBreakTime: (seconds: number) =>
      invoke<void>("log_break_time", { durationSeconds: seconds }),
  },

  window: {
    getOrientation: () => invoke<WindowOrientation>("get_window_orientation"),
    minimize: () => invoke<void>("minimize_window"),
    toggleMaximize: () => invoke<void>("toggle_maximize"),
    close: () => invoke<void>("close_window"),
    dock: (side: "left" | "right") => invoke<void>("dock_window", { side }),
    center: () => invoke<void>("center_window"),
    setCollapsed: (collapsed: boolean, top: number) => invoke<void>("set_collapsed", { collapsed, top }),
    move: (x: number, y: number) => invoke<void>("move_window", { x, y }),
    startDragging: () => invoke<void>("start_window_drag"),
  },

  googleCalendar: {
    authStart: () => invoke<string>("google_auth_start"),
    authStatus: () => invoke<{ connected: boolean }>("google_auth_status"),
    disconnect: () => invoke<void>("google_auth_disconnect"),
    syncAll: () =>
      invoke<{ synced: number; failed: number; errors: string[] }>(
        "google_sync_all_tasks"
      ),
  },
};
