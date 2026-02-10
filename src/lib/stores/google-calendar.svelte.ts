import { db } from "$lib/services/db";

let connected = $state(false);
let connecting = $state(false);
let syncing = $state(false);
let error = $state<string | null>(null);
let lastSyncResult = $state<{
  synced: number;
  failed: number;
} | null>(null);

let unlisten: (() => void) | null = null;

export const googleCalendarStore = {
  get connected() {
    return connected;
  },
  get connecting() {
    return connecting;
  },
  get syncing() {
    return syncing;
  },
  get error() {
    return error;
  },
  get lastSyncResult() {
    return lastSyncResult;
  },

  async checkStatus() {
    try {
      const status = await db.googleCalendar.authStatus();
      connected = status.connected;
    } catch (e) {
      console.error("Failed to check Google Calendar status:", e);
    }
  },

  async connect() {
    if (connecting) return;
    connecting = true;
    error = null;

    try {
      const authUrl = await db.googleCalendar.authStart();

      // Open the auth URL in the default browser
      const { openUrl } = await import("@tauri-apps/plugin-opener");
      await openUrl(authUrl);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      connecting = false;
    }
  },

  async disconnect() {
    try {
      await db.googleCalendar.disconnect();
      connected = false;
      lastSyncResult = null;
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  },

  async syncAll() {
    if (syncing) return;
    syncing = true;
    error = null;

    try {
      const result = await db.googleCalendar.syncAll();
      lastSyncResult = { synced: result.synced, failed: result.failed };
      if (result.failed > 0) {
        error = `${result.failed} task(s) failed to sync`;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      syncing = false;
    }
  },

  async init() {
    await this.checkStatus();

    // Listen for auth completion events from the backend
    try {
      const { listen } = await import("@tauri-apps/api/event");
      if (unlisten) unlisten();
      unlisten = await listen<boolean>("google-auth-complete", (event) => {
        connecting = false;
        if (event.payload) {
          connected = true;
          error = null;
        } else {
          error = "Authentication failed. Please try again.";
        }
      });
    } catch (e) {
      console.error("Failed to set up auth listener:", e);
    }
  },
};
