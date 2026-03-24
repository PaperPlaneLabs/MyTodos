<script lang="ts">
    import { onMount } from "svelte";
    import { fade, scale } from "svelte/transition";

    // ── State ─────────────────────────────────────────────────────────────────
    let message = $state("Time for a quick break?");
    let sending = $state(false);

    let isBreaking = $state(false);
    let breakElapsedSeconds = $state(0);
    let breakIntervalId: number | null = null;
    let breakStartTime = 0;

    let mounted = $state(false);

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

        // Trigger entrance animation
        requestAnimationFrame(() => {
            mounted = true;
        });

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

    async function dragWindow() {
        try {
            const { invoke } = await import("@tauri-apps/api/core");
            await invoke("start_window_drag");
        } catch {
            // ignore
        }
    }

    function formatTime(seconds: number): string {
        const m = Math.floor(seconds / 60);
        const s = seconds % 60;
        return `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
    }
</script>

<div class="shell" class:mounted>
    <!-- Drag region / custom title bar -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="drag-bar" onmousedown={dragWindow} role="presentation">
        <div class="drag-dots">
            <span></span><span></span><span></span>
        </div>
        <button
            class="close-btn"
            onclick={() => emitAndClose("dismiss")}
            title="Dismiss"
            aria-label="Dismiss break reminder"
        >
            <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
                <path
                    d="M1 1l8 8M9 1l-8 8"
                    stroke="currentColor"
                    stroke-width="1.5"
                    stroke-linecap="round"
                />
            </svg>
        </button>
    </div>

    <!-- Content -->
    <div class="content">
        {#if !isBreaking}
            <div class="icon-ring" in:scale={{ duration: 400, delay: 100 }}>
                <span class="icon">☕</span>
            </div>

            <div class="text-block">
                <h1 class="title">{message}</h1>
                <p class="subtitle">
                    Short breaks keep your mind sharp.<br />
                    Step away for a few minutes.
                </p>
            </div>

            <div class="actions" in:fade={{ duration: 200, delay: 200 }}>
                <button
                    class="btn btn-primary"
                    onclick={startBreak}
                    disabled={sending}
                >
                    <svg
                        width="14"
                        height="14"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2.5"
                        ><path
                            d="M12 2v6m0 8v6M4.93 4.93l4.24 4.24m5.66 5.66 4.24 4.24M2 12h6m8 0h6M4.93 19.07l4.24-4.24m5.66-5.66 4.24-4.24"
                        /></svg
                    >
                    Take a break
                </button>
                <button
                    class="btn btn-ghost"
                    onclick={() => emitAndClose("snooze")}
                    disabled={sending}
                >
                    Remind me in 10 min
                </button>
            </div>
        {:else}
            <div class="breaking-state" in:fade={{ duration: 300 }}>
                <div class="pulse-ring">
                    <div class="pulse-dot"></div>
                </div>
                <div class="timer-display">
                    {formatTime(breakElapsedSeconds)}
                </div>
                <p class="timer-label">Break in progress</p>
                <button
                    class="btn btn-primary"
                    onclick={() => emitAndClose("resume")}
                    disabled={sending}
                >
                    <svg
                        width="14"
                        height="14"
                        viewBox="0 0 24 24"
                        fill="currentColor"><path d="M8 5v14l11-7z" /></svg
                    >
                    Resume work
                </button>
            </div>
        {/if}
    </div>
</div>

<style>
    :global(*) {
        box-sizing: border-box;
        margin: 0;
        padding: 0;
    }

    :global(body) {
        background: transparent;
        overflow: hidden;
    }

    /* ── Shell ────────────────────────────────────────────────────────────── */
    .shell {
        width: 100vw;
        height: 100vh;
        display: flex;
        flex-direction: column;
        background-color: var(--bg-primary);
        color: var(--text-primary);
        font-family: var(--font-sans, system-ui, sans-serif);
        border-radius: 12px;
        border: 1px solid var(--border);
        overflow: hidden;
        opacity: 0;
        transform: scale(0.96) translateY(6px);
        transition:
            opacity 0.25s ease,
            transform 0.25s cubic-bezier(0.22, 1, 0.36, 1);
    }

    .shell.mounted {
        opacity: 1;
        transform: scale(1) translateY(0);
    }

    /* ── Drag bar ─────────────────────────────────────────────────────────── */
    .drag-bar {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 8px 10px 8px 12px;
        cursor: grab;
        background-color: var(--bg-secondary);
        border-bottom: 1px solid var(--border-light, var(--border));
        flex-shrink: 0;
        user-select: none;
    }

    .drag-bar:active {
        cursor: grabbing;
    }

    .drag-dots {
        display: flex;
        gap: 4px;
        align-items: center;
    }

    .drag-dots span {
        display: block;
        width: 5px;
        height: 5px;
        border-radius: 50%;
        background-color: var(--text-tertiary, #aaa);
        opacity: 0.4;
    }

    /* Close button — styled like macOS traffic lights */
    .close-btn {
        width: 20px;
        height: 20px;
        border-radius: 50%;
        border: none;
        background-color: var(--bg-tertiary);
        color: var(--text-tertiary);
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        transition:
            background-color 0.15s,
            color 0.15s;
        flex-shrink: 0;
    }

    .close-btn:hover {
        background-color: var(--danger);
        color: var(--danger-contrast);
    }

    /* ── Content ──────────────────────────────────────────────────────────── */
    .content {
        flex: 1;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 24px 28px 28px;
        gap: 16px;
        text-align: center;
    }

    /* ── Icon ring ────────────────────────────────────────────────────────── */
    .icon-ring {
        width: 64px;
        height: 64px;
        border-radius: 50%;
        background: var(
            --accent-light,
            color-mix(in srgb, var(--accent) 12%, transparent)
        );
        display: flex;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;
    }

    .icon {
        font-size: 30px;
        line-height: 1;
    }

    /* ── Text ─────────────────────────────────────────────────────────────── */
    .text-block {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .title {
        font-size: 16px;
        font-weight: 700;
        color: var(--text-primary);
        line-height: 1.3;
        letter-spacing: -0.2px;
    }

    .subtitle {
        font-size: 12px;
        color: var(--text-secondary);
        line-height: 1.6;
        opacity: 0.85;
    }

    /* ── Actions ──────────────────────────────────────────────────────────── */
    .actions {
        display: flex;
        flex-direction: column;
        gap: 8px;
        width: 100%;
    }

    .btn {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 7px;
        width: 100%;
        padding: 9px 16px;
        font-size: 13px;
        font-weight: 600;
        border-radius: 8px;
        border: none;
        cursor: pointer;
        font-family: inherit;
        transition:
            background-color 0.15s,
            opacity 0.15s,
            transform 0.1s;
    }

    .btn:active:not(:disabled) {
        transform: scale(0.98);
    }

    .btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .btn-primary {
        background-color: var(--accent);
        color: var(--accent-contrast);
    }

    .btn-primary:hover:not(:disabled) {
        background-color: var(--accent-hover, var(--accent));
        filter: brightness(1.08);
    }

    .btn-ghost {
        background-color: transparent;
        color: var(--text-tertiary);
        font-weight: 500;
    }

    .btn-ghost:hover:not(:disabled) {
        background-color: var(--bg-hover);
        color: var(--text-secondary);
    }

    /* ── Breaking state ───────────────────────────────────────────────────── */
    .breaking-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 12px;
        width: 100%;
    }

    .pulse-ring {
        position: relative;
        width: 48px;
        height: 48px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .pulse-ring::before,
    .pulse-ring::after {
        content: "";
        position: absolute;
        inset: 0;
        border-radius: 50%;
        border: 2px solid var(--accent);
        animation: pulse-ring 2s ease-out infinite;
        opacity: 0;
    }

    .pulse-ring::after {
        animation-delay: 1s;
    }

    .pulse-dot {
        width: 16px;
        height: 16px;
        border-radius: 50%;
        background-color: var(--accent);
        position: relative;
        z-index: 1;
    }

    @keyframes pulse-ring {
        0% {
            transform: scale(0.6);
            opacity: 0.7;
        }
        100% {
            transform: scale(1.6);
            opacity: 0;
        }
    }

    .timer-display {
        font-size: 38px;
        font-weight: 700;
        color: var(--accent);
        font-variant-numeric: tabular-nums;
        letter-spacing: -1px;
        line-height: 1;
    }

    .timer-label {
        font-size: 11px;
        color: var(--text-tertiary);
        text-transform: uppercase;
        letter-spacing: 0.08em;
        font-weight: 600;
    }
</style>
