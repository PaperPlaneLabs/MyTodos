import { db, type BackupSettings } from "$lib/services/db";
import { calendarStore } from "./calendar.svelte";
import { projectStore } from "./projects.svelte";
import { taskStore } from "./tasks.svelte";
import { timerStore } from "./timer.svelte";
import { windowTrackingStore } from "./window-tracking.svelte";

const defaultSettings: BackupSettings = {
  enabled: false,
  folder: "",
  interval_minutes: 15,
  last_backup_at: null,
};

let settings = $state<BackupSettings>(defaultSettings);
let cloudFolders = $state<string[]>([]);
let busy = $state(false);
let error = $state<string | null>(null);

async function refreshVisibleData(): Promise<void> {
  const selectedProjectId = projectStore.selectedId;
  await Promise.all([
    projectStore.loadAll(),
    taskStore.loadByProject(selectedProjectId),
    timerStore.loadActive(),
    windowTrackingStore.refresh(),
    calendarStore.refreshCurrentRange(),
  ]);
}

export const backupStore = {
  get settings() {
    return settings;
  },

  get cloudFolders() {
    return cloudFolders;
  },

  get busy() {
    return busy;
  },

  get error() {
    return error;
  },

  async init() {
    try {
      error = null;
      const [nextSettings, nextCloudFolders] = await Promise.all([
        db.backup.getSettings(),
        db.backup.checkCloudFolders(),
      ]);
      settings = nextSettings;
      cloudFolders = nextCloudFolders;
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load backup settings";
      console.error("Failed to load backup settings:", e);
    }
  },

  async save(patch: Partial<BackupSettings>) {
    try {
      error = null;
      const next = { ...settings, ...patch };
      settings = await db.backup.setSettings({
        enabled: next.enabled,
        folder: next.folder,
        interval_minutes: next.interval_minutes,
      });
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to save backup settings";
      console.error("Failed to save backup settings:", e);
      throw e;
    }
  },

  async backupNow() {
    if (busy) return;
    busy = true;
    try {
      error = null;
      settings = await db.backup.backupNow();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to create backup";
      console.error("Failed to create backup:", e);
      throw e;
    } finally {
      busy = false;
    }
  },

  async restore(path: string) {
    if (busy) return;
    busy = true;
    try {
      error = null;
      await db.backup.restore(path);
      await refreshVisibleData();
      settings = await db.backup.getSettings();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to restore backup";
      console.error("Failed to restore backup:", e);
      throw e;
    } finally {
      busy = false;
    }
  },
};
