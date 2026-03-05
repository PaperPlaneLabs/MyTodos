<script lang="ts">
    import { onMount } from "svelte";

    // ── State ─────────────────────────────────────────────────────────────────
    let message = $state("Time for a quick break?");
    let sending = $state(false);

    let isBreaking = $state(false);
    let breakElapsedSeconds = $state(0);
    let breakIntervalId: number | null = null;
    let breakStartTime = 0;

    onMount(() => {
        // Apply theme: prefer injected value > localStorage > default
        const theme =
            typeof window.__BREAK_THEME__ === "string" && window.__BREAK_THEME__
                ? window.__BREAK_THEME__
                : (localStorage.getItem("theme") ?? "light");
        document.documentElement.setAttribute("data-theme", theme);

        // Read the injected message
        if (
            typeof window.__BREAK_MESSAGE__ === "string" &&
            window.__BREAK_MESSAGE__
        ) {
            message = window.__BREAK_MESSAGE__;
        }

        return () => {
            if (breakIntervalId !== null) {
                clearInterval(breakIntervalId);
            }
        };
    });

    // ── Tauri API helpers ─────────────────────────────────────────────────────
    async function emitAndClose(
        action: "take_break" | "dismiss" | "snooze" | "resume",
    ) {
        if (sending) return;
        sending = true;
        try {
            const { invoke } = await import("@tauri-apps/api/core");
            const { emit } = await import("@tauri-apps/api/event");

            // If we are resuming, log the break time first
            if (action === "resume" && breakElapsedSeconds > 0) {
                const { db } = await import("$lib/services/db");
                await db.timeEntries.logBreakTime(breakElapsedSeconds);
            }

            await emit("break:action", { action });
            await invoke("close_break_window");
        } catch (e) {
            console.error("[break] action failed:", e);
            sending = false;
        }
    }

    async function startBreak() {
        if (sending) return;
        sending = true;
        try {
            const { emit } = await import("@tauri-apps/api/event");
            // Tell main window to pause the timer
            await emit("break:action", { action: "take_break" });

            isBreaking = true;
            breakStartTime = Date.now();
            breakElapsedSeconds = 0;

            breakIntervalId = window.setInterval(() => {
                breakElapsedSeconds = Math.floor(
                    (Date.now() - breakStartTime) / 1000,
                );
            }, 1000);

            sending = false;
        } catch (e) {
            console.error("[break] start break failed:", e);
            sending = false;
        }
    }

    function formatTime(seconds: number): string {
        const m = Math.floor(seconds / 60);
        const s = seconds % 60;
        return `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
    }
</script>

<div class="break-window">
    <div class="break-icon">☕</div>

    <h1 class="break-title">{message}</h1>
    <p class="break-subtitle">
        Short breaks boost focus. Step away from the screen for a few minutes.
    </p>

    <div class="actions">
        {#if !isBreaking}
            <button
                class="break-btn break-btn-primary"
                onclick={startBreak}
                disabled={sending}
            >
                Take a break
            </button>

            <button
                class="break-btn break-btn-ghost"
                onclick={() => emitAndClose("snooze")}
                disabled={sending}
            >
                Remind me in 10 min
            </button>
        {:else}
            <div class="break-timer">
                {formatTime(breakElapsedSeconds)}
            </div>
            <button
                class="break-btn break-btn-primary"
                onclick={() => emitAndClose("resume")}
                disabled={sending}
            >
                Resume work
            </button>
        {/if}
    </div>
</div>

<style>
    .break-window {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100vh;
        padding: 32px 24px;
        gap: 12px;
        background-color: var(--bg-primary);
        color: var(--text-primary);
        font-family: var(--font-sans);
        text-align: center;
        box-sizing: border-box;
    }

    .break-icon {
        font-size: 40px;
        line-height: 1;
        margin-bottom: 4px;
    }

    .break-title {
        font-size: 18px;
        font-weight: 700;
        color: var(--text-primary);
        margin: 0;
        line-height: 1.3;
    }

    .break-subtitle {
        font-size: 13px;
        color: var(--text-secondary);
        margin: 0 0 8px;
        max-width: 320px;
        line-height: 1.5;
    }

    .actions {
        display: flex;
        flex-direction: column;
        gap: 8px;
        width: 100%;
        max-width: 240px;
    }

    .break-btn {
        width: 100%;
        padding: 9px 16px;
        font-size: 13px;
        font-weight: 500;
        border-radius: 8px;
        border: none;
        cursor: pointer;
        transition:
            background-color 150ms ease,
            opacity 150ms ease;
        font-family: inherit;
    }

    .break-btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }

    .break-btn-primary {
        background-color: var(--accent);
        color: #fff;
    }
    .break-btn-primary:hover:not(:disabled) {
        background-color: var(--accent-hover);
    }

    .break-btn-ghost {
        background-color: transparent;
        color: var(--text-tertiary);
    }
    .break-btn-ghost:hover:not(:disabled) {
        color: var(--text-secondary);
        background-color: var(--bg-hover);
    }

    .break-timer {
        font-size: 36px;
        font-weight: 700;
        color: var(--accent);
        font-variant-numeric: tabular-nums;
        margin: 12px 0 16px;
        letter-spacing: -0.5px;
    }
</style>
