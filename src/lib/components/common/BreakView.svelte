<script lang="ts">
    import { onMount } from "svelte";

    // ── State ─────────────────────────────────────────────────────────────────
    let message = $state("Time for a quick break?");
    let sending = $state(false);

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
    });

    // ── Tauri API helpers ─────────────────────────────────────────────────────
    async function emitAndClose(action: "break" | "skip" | "snooze") {
        if (sending) return;
        sending = true;
        try {
            const { invoke } = await import("@tauri-apps/api/core");
            const { emit } = await import("@tauri-apps/api/event");
            await emit("break:action", { action });
            await invoke("close_break_window");
        } catch (e) {
            console.error("[break] action failed:", e);
            sending = false;
        }
    }
</script>

<div class="break-window">
    <div class="break-icon">☕</div>

    <h1 class="break-title">{message}</h1>
    <p class="break-subtitle">
        Short breaks boost focus. Step away from the screen for a few minutes.
    </p>

    <div class="actions">
        <button
            class="break-btn break-btn-primary"
            onclick={() => emitAndClose("break")}
            disabled={sending}
        >
            Take a break
        </button>

        <button
            class="break-btn break-btn-secondary"
            onclick={() => emitAndClose("skip")}
            disabled={sending}
        >
            Keep going
        </button>

        <button
            class="break-btn break-btn-ghost"
            onclick={() => emitAndClose("snooze")}
            disabled={sending}
        >
            Remind me in 10 min
        </button>
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

    .break-btn-secondary {
        background-color: var(--bg-secondary);
        color: var(--text-primary);
        border: 1px solid var(--border);
    }
    .break-btn-secondary:hover:not(:disabled) {
        background-color: var(--bg-tertiary);
    }

    .break-btn-ghost {
        background-color: transparent;
        color: var(--text-tertiary);
    }
    .break-btn-ghost:hover:not(:disabled) {
        color: var(--text-secondary);
        background-color: var(--bg-hover);
    }
</style>
