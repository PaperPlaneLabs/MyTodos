import { db, type Task, type Section } from "$lib/services/db";

let tasks = $state<Task[]>([]);
let sections = $state<Section[]>([]);
let currentProjectId = $state<number | null>(null);
let loading = $state(false);
let error = $state<string | null>(null);

let sortedTasks = $derived([...tasks].sort((a, b) => {
  if (a.completed !== b.completed) return a.completed ? 1 : -1;
  return a.position - b.position;
}));

export const taskStore = {
  get tasks() {
    return sortedTasks;
  },

  get activeTasks() {
    return sortedTasks.filter(t => !t.completed);
  },

  get completedTasks() {
    return sortedTasks.filter(t => t.completed);
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
      currentProjectId = projectId;
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
      if (completed !== undefined) {
        await db.tasks.reorder(sortedTasks.map(t => t.id));
      }
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
      
      // Persist the new order because completed tasks should move to the bottom
      const sortedIds = sortedTasks.map(t => t.id);
      await db.tasks.reorder(sortedIds);
      
      return newCompleted;
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to toggle task";
      console.error("Failed to toggle task:", e);
      throw e;
    }
  },

  async reorder(taskIds: number[]) {
    try {
      error = null;
      // Optimistically update local state if the ids match the current tasks
      // But reordering is complex to do optimistically if we just have IDs. 
      // The UI will likely have already updated the visual order.
      await db.tasks.reorder(taskIds);
      // Reload to ensure consistency
      await this.loadByProject(currentProjectId);
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to reorder tasks";
      console.error("Failed to reorder tasks:", e);
      throw e;
    }
  },

  async move(fromIndex: number, toIndex: number) {
    if (fromIndex === toIndex) return;

    this.reorderLocal(fromIndex, toIndex);

    try {
      error = null;
      const ids = tasks.map(t => t.id);
      await db.tasks.reorder(ids);
    } catch (e) {
      console.error("Failed to persist task order:", e);
      error = e instanceof Error ? e.message : "Failed to save order";
      await this.loadByProject(currentProjectId);
    }
  },

  reorderLocal(fromIndex: number, toIndex: number) {
    if (fromIndex === toIndex) return;
    const currentTasks = sortedTasks;
    const newTasks = [...currentTasks];
    const [moved] = newTasks.splice(fromIndex, 1);
    newTasks.splice(toIndex, 0, moved);
    
    // Update the position property to match the new order
    tasks = newTasks.map((t, i) => ({ ...t, position: i }));
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

  updateTaskTime(taskId: number, timeSeconds: number) {
    tasks = tasks.map((t) =>
      t.id === taskId ? { ...t, total_time_seconds: timeSeconds } : t
    );
  },

  clear() {
    tasks = [];
    sections = [];
  },
};
