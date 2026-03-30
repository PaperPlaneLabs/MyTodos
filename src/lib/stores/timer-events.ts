import type { AutoPauseEvent, AutoPauseReason } from "$lib/services/db";

type BreakAction = "take_break" | "dismiss" | "snooze" | "resume";
type AwayTimeLoggedEvent = {
  affectsVisibleTaskTotals: boolean;
};

interface RegisterTimerEventHandlersOptions {
  onAutoPaused: (reason: AutoPauseReason) => void;
  onTakeBreak: () => Promise<void> | void;
  onDismiss: () => void;
  onSnooze: () => void;
  onResume: () => Promise<void> | void;
  onAwayTimeLogged: (event: AwayTimeLoggedEvent) => Promise<void> | void;
}

let listenersRegistered = false;

export function registerTimerEventHandlers({
  onAutoPaused,
  onTakeBreak,
  onDismiss,
  onSnooze,
  onResume,
  onAwayTimeLogged,
}: RegisterTimerEventHandlersOptions): void {
  if (typeof window === "undefined" || listenersRegistered) {
    return;
  }

  listenersRegistered = true;

  void (async () => {
    const { listen } = await import("@tauri-apps/api/event");

    await listen<AutoPauseEvent>("timer:auto-paused", (event) => {
      console.log("Timer auto-paused:", event.payload.reason);
      onAutoPaused(event.payload.reason);
    });

    await listen<AwayTimeLoggedEvent>("timer:away-time-logged", (event) => {
      Promise.resolve(onAwayTimeLogged(event.payload)).catch((error) =>
        console.error("Failed to sync away-time logging:", error),
      );
    });

    await listen<{ action: BreakAction }>("break:action", (event) => {
      const { action } = event.payload;

      if (action === "take_break") {
        Promise.resolve(onTakeBreak()).catch((error) =>
          console.error("Failed to pause for break:", error),
        );
      } else if (action === "dismiss") {
        onDismiss();
      } else if (action === "snooze") {
        onSnooze();
      } else if (action === "resume") {
        Promise.resolve(onResume()).catch((error) =>
          console.error("Failed to resume after break:", error),
        );
      }
    });
  })();
}
