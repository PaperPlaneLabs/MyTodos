<script lang="ts">
    import { onMount } from "svelte";
    import "$lib/styles/global.css";

    // ── State ─────────────────────────────────────────────────────────────────
    let message = $state("Time for a quick break?");
    let sending = $state(false);

    // ── Theme bootstrap ───────────────────────────────────────────────────────
    // Runs synchronously before first paint so there is no flash of wrong theme.
    // Priority: injected via initialization_script > localStorage > default
    const theme =
        typeof window !== "undefined" &&
        typeof window.__BREAK_THEME__ === "string" &&
        window.__BREAK_THEME__
            ? window.__BREAK_THEME__
            : typeof window !== "undefined"
              ? (localStorage.getItem("theme") ?? "light")
              : "light";

    onMount(() => {
        // Apply theme to document root
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

<svelte:head>
    <title>Break Reminder</title>
</svelte:head>

<div class="break-window">
    <div class="break-icon">☕</div>

    <h1 class="break-title">{message}</h1>
    <p class="break-subtitle">
        Short breaks boost focus. Step away from the screen for a few minutes.
    </p>

    <div class="actions">
        <button
            class="btn btn-primary"
            onclick={() => emitAndClose("break")}
            disabled={sending}
        >
            Take a break
        </button>

        <button
            class="btn btn-secondary"
            onclick={() => emitAndClose("skip")}
            disabled={sending}
        >
            Keep going
        </button>

        <button
            class="btn btn-ghost"
            onclick={() => emitAndClose("snooze")}
            disabled={sending}
        >
            Remind me in 10 min
        </button>
    </div>
</div>

<style>
    :global(html, body) {
        height: 100%;
        overflow: hidden;
        background: transparent;
    }

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

    .btn {
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

    .btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }

    .btn-primary {
        background-color: var(--accent);
        color: #fff;
    }
    .btn-primary:hover:not(:disabled) {
        background-color: var(--accent-hover);
    }

    .btn-secondary {
        background-color: var(--bg-secondary);
        color: var(--text-primary);
        border: 1px solid var(--border);
    }
    .btn-secondary:hover:not(:disabled) {
        background-color: var(--bg-tertiary);
    }

    .btn-ghost {
        background-color: transparent;
        color: var(--text-tertiary);
    }
    .btn-ghost:hover:not(:disabled) {
        color: var(--text-secondary);
        background-color: var(--bg-hover);
    }
</style>
