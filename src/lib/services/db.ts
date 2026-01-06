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

export const db = {
  projects: {
    getAll: () => invoke<Project[]>("get_all_projects"),
    get: (id: number) => invoke<Project>("get_project", { id }),
    create: (name: string, description?: string, color?: string) =>
      invoke<Project>("create_project", { name, description, color }),
    update: (id: number, name?: string, description?: string, color?: string) =>
      invoke<void>("update_project", { id, name, description, color }),
    delete: (id: number) => invoke<void>("delete_project", { id }),
    reorder: (projectIds: number[]) => invoke<void>("reorder_projects", { projectIds }),
    getStats: (projectId: number) => invoke<ProjectStats>("get_project_stats", { projectId }),
  },

  sections: {
    getByProject: (projectId: number) => invoke<Section[]>("get_sections_by_project", { projectId }),
    create: (projectId: number, name: string) =>
      invoke<Section>("create_section", { projectId, name }),
    update: (id: number, name: string) => invoke<void>("update_section", { id, name }),
    delete: (id: number) => invoke<void>("delete_section", { id }),
    reorder: (sectionIds: number[]) => invoke<void>("reorder_sections", { sectionIds }),
  },

  tasks: {
    getByProject: (projectId: number) => invoke<Task[]>("get_tasks_by_project", { projectId }),
    getUnassigned: () => invoke<Task[]>("get_unassigned_tasks"),
    getBySection: (sectionId: number) => invoke<Task[]>("get_tasks_by_section", { sectionId }),
    create: (projectId: number | null, sectionId: number | null, title: string, description?: string) =>
      invoke<Task>("create_task", { projectId, sectionId, title, description }),
    update: (id: number, title?: string, description?: string, completed?: boolean) =>
      invoke<void>("update_task", { id, title, description, completed }),
    delete: (id: number) => invoke<void>("delete_task", { id }),
    toggleCompletion: (id: number) => invoke<boolean>("toggle_task_completion", { id }),
    reorder: (taskIds: number[]) => invoke<void>("reorder_tasks", { taskIds }),
    resetTime: (id: number) => invoke<void>("reset_task_time", { id }),
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
    getByTask: (taskId: number) => invoke<TimeEntry[]>("get_time_entries_by_task", { taskId }),
    createManual: (taskId: number, durationSeconds: number, note?: string) =>
      invoke<TimeEntry>("create_manual_entry", { taskId, durationSeconds, note }),
    update: (id: number, durationSeconds: number, note?: string) =>
      invoke<void>("update_time_entry", { id, durationSeconds, note }),
    delete: (id: number) => invoke<void>("delete_time_entry", { id }),
  },

  window: {
    saveState: (x?: number, y?: number, width?: number, height?: number) =>
      invoke<void>("save_window_state", { x, y, width, height }),
    getState: () => invoke<WindowState | null>("get_window_state"),
    minimize: () => invoke<void>("minimize_window"),
    toggleMaximize: () => invoke<void>("toggle_maximize"),
    close: () => invoke<void>("close_window"),
    dock: (side: "left" | "right") => invoke<void>("dock_window", { side }),
  },

  initialize: () => invoke<void>("initialize_database"),
};
