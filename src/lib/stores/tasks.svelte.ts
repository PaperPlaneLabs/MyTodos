import { db, type Task, type Section } from "$lib/services/db";

let tasks = $state<Task[]>([]);
let sections = $state<Section[]>([]);
let loading = $state(false);
let error = $state<string | null>(null);

export const taskStore = {
  get tasks() {
    return tasks;
  },

  get sections() {
    return sections;
  },

  get loading() {
    return loading;
  },

  get error() {
    return error;
  },

  async loadByProject(projectId: number | null) {
    try {
      loading = true;
      error = null;
      if (projectId === null) {
        tasks = await db.tasks.getUnassigned();
        sections = [];
      } else {
        const [loadedTasks, loadedSections] = await Promise.all([
          db.tasks.getByProject(projectId),
          db.sections.getByProject(projectId),
        ]);
        tasks = loadedTasks;
        sections = loadedSections;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load tasks";
      console.error("Failed to load tasks:", e);
    } finally {
      loading = false;
    }
  },

  async createTask(projectId: number | null, sectionId: number | null, title: string, description?: string) {
    try {
      error = null;
      const task = await db.tasks.create(projectId, sectionId, title, description);
      tasks = [...tasks, task];
      return task;
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to create task";
      console.error("Failed to create task:", e);
      throw e;
    }
  },

  async updateTask(id: number, title?: string, description?: string, completed?: boolean) {
    try {
      error = null;
      await db.tasks.update(id, title, description, completed);
      tasks = tasks.map((t) =>
        t.id === id
          ? { ...t, title: title ?? t.title, description: description ?? t.description, completed: completed ?? t.completed }
          : t
      );
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to update task";
      console.error("Failed to update task:", e);
      throw e;
    }
  },

  async deleteTask(id: number) {
    try {
      error = null;
      await db.tasks.delete(id);
      tasks = tasks.filter((t) => t.id !== id);
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to delete task";
      console.error("Failed to delete task:", e);
      throw e;
    }
  },

  async toggleCompletion(id: number) {
    try {
      error = null;
      const newCompleted = await db.tasks.toggleCompletion(id);
      tasks = tasks.map((t) => (t.id === id ? { ...t, completed: newCompleted } : t));
      return newCompleted;
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to toggle task";
      console.error("Failed to toggle task:", e);
      throw e;
    }
  },

  async resetTaskTime(id: number) {
    try {
      error = null;
      await db.tasks.resetTime(id);
      tasks = tasks.map((t) => (t.id === id ? { ...t, total_time_seconds: 0 } : t));
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to reset task time";
      console.error("Failed to reset task time:", e);
      throw e;
    }
  },

  async createSection(projectId: number, name: string) {
    try {
      error = null;
      const section = await db.sections.create(projectId, name);
      sections = [...sections, section];
      return section;
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to create section";
      console.error("Failed to create section:", e);
      throw e;
    }
  },

  async updateSection(id: number, name: string) {
    try {
      error = null;
      await db.sections.update(id, name);
      sections = sections.map((s) => (s.id === id ? { ...s, name } : s));
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to update section";
      console.error("Failed to update section:", e);
      throw e;
    }
  },

  async deleteSection(id: number) {
    try {
      error = null;
      await db.sections.delete(id);
      sections = sections.filter((s) => s.id !== id);
      tasks = tasks.filter((t) => t.section_id !== id);
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to delete section";
      console.error("Failed to delete section:", e);
      throw e;
    }
  },

  getTasksBySection(sectionId: number | null) {
    return tasks.filter((t) => t.section_id === sectionId);
  },

  clear() {
    tasks = [];
    sections = [];
  },
};
