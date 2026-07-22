import { getVersion } from "@tauri-apps/api/app";

import { db } from "$lib/services/db";
import {
  getReleaseHistory,
  getUnseenReleaseNotes,
  type ReleaseNote,
} from "$lib/services/release-notes";

type WhatsNewMode = "unseen" | "history";

let open = $state(false);
let mode = $state<WhatsNewMode>("unseen");
let currentVersion = $state("");
let visibleNotes = $state<ReleaseNote[]>([]);
let error = $state<string | null>(null);
let initialized = false;

export const whatsNewStore = {
  get open() {
    return open;
  },

  get mode() {
    return mode;
  },

  get currentVersion() {
    return currentVersion;
  },

  get visibleNotes() {
    return visibleNotes;
  },

  get error() {
    return error;
  },

  async init(): Promise<void> {
    if (initialized) return;
    initialized = true;

    try {
      error = null;
      const [installedVersion, lastSeenVersion] = await Promise.all([
        getVersion(),
        db.whatsNew.getLastSeenVersion(),
      ]);
      currentVersion = installedVersion;
      const unseenNotes = getUnseenReleaseNotes(
        installedVersion,
        lastSeenVersion,
      );
      if (unseenNotes.length > 0) {
        mode = "unseen";
        visibleNotes = unseenNotes;
        open = true;
      }
    } catch (initError) {
      initialized = false;
      error =
        initError instanceof Error
          ? initError.message
          : "Failed to load What's New";
      console.error("Failed to initialize What's New:", initError);
    }
  },

  openHistory(): void {
    mode = "history";
    visibleNotes = getReleaseHistory();
    open = true;
  },

  async close(): Promise<void> {
    const shouldMarkSeen = mode === "unseen" && currentVersion.length > 0;
    open = false;

    if (!shouldMarkSeen) return;
    try {
      await db.whatsNew.setLastSeenVersion(currentVersion);
    } catch (saveError) {
      error =
        saveError instanceof Error
          ? saveError.message
          : "Failed to save What's New state";
      console.error("Failed to mark What's New as seen:", saveError);
    }
  },
};
