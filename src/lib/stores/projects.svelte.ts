import { db, type Project } from "$lib/services/db";

let projects = $state<Project[]>([]);
let selectedProjectId = $state<number | null>(null);
let loading = $state(false);
let error = $state<string | null>(null);

export const projectStore = {
  get projects() {
    return projects;
  },

  get selected() {
    return projects.find((p) => p.id === selectedProjectId) ?? null;
  },

  get selectedId() {
    return selectedProjectId;
  },

  get loading() {
    return loading;
  },

  get error() {
    return error;
  },

  setSelected(id: number | null) {
    selectedProjectId = id;
  },

  async loadAll() {
    try {
      loading = true;
      error = null;
      projects = await db.projects.getAll();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load projects";
      console.error("Failed to load projects:", e);
    } finally {
      loading = false;
    }
  },

  async create(name: string, description?: string, color?: string) {
    try {
      error = null;
      const project = await db.projects.create(name, description, color);
      projects = [...projects, project];
      return project;
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to create project";
      console.error("Failed to create project:", e);
      throw e;
    }
  },

  async update(id: number, name?: string, description?: string, color?: string) {
    try {
      error = null;
      await db.projects.update(id, name, description, color);
      await this.loadAll();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to update project";
      console.error("Failed to update project:", e);
      throw e;
    }
  },

  async delete(id: number) {
    try {
      error = null;
      await db.projects.delete(id);
      projects = projects.filter((p) => p.id !== id);
      if (selectedProjectId === id) {
        selectedProjectId = null;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to delete project";
      console.error("Failed to delete project:", e);
      throw e;
    }
  },

  async reorder(projectIds: number[]) {
    try {
      error = null;
      await db.projects.reorder(projectIds);
      await this.loadAll();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to reorder projects";
      console.error("Failed to reorder projects:", e);
      throw e;
    }
  },

  async move(fromIndex: number, toIndex: number) {
    if (fromIndex === toIndex) return;
    
    this.reorderLocal(fromIndex, toIndex);

    try {
      error = null;
      const ids = projects.map(p => p.id);
      await db.projects.reorder(ids);
    } catch (e) {
      // Revert on error? For now just log and reload
      console.error("Failed to persist project order:", e);
      error = e instanceof Error ? e.message : "Failed to save order";
      await this.loadAll();
    }
  },

  reorderLocal(fromIndex: number, toIndex: number) {
    if (fromIndex === toIndex) return;
    const newProjects = [...projects];
    const [moved] = newProjects.splice(fromIndex, 1);
    newProjects.splice(toIndex, 0, moved);
    projects = newProjects;
  },
};
