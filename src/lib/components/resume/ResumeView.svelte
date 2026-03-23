<script lang="ts">
    import { onMount } from "svelte";
    import { fade, scale } from "svelte/transition";

    let taskId = $state<number | null>(null);
    let taskTitle = $state("");
    let awayTimeSeconds = $state(0);
    let sending = $state(false);
    let mounted = $state(false);

    onMount(() => {
        const theme = window.__RESUME_DATA__?.theme || localStorage.getItem("theme") || "light";
        document.documentElement.setAttribute("data-theme", theme);

        if (window.__RESUME_DATA__) {
            taskId = window.__RESUME_DATA__.taskId;
            taskTitle = window.__RESUME_DATA__.taskTitle;
            awayTimeSeconds = window.__RESUME_DATA__.awayTimeSeconds;
        }

        requestAnimationFrame(() => {
            mounted = true;
        });
    });

    function formatTime(seconds: number): string {
        if (seconds < 60) {
            return `${seconds} sec`;
        }
        const m = Math.floor(seconds / 60);
        const s = seconds % 60;
        if (m < 60) {
            return `${m}m ${s}s`;
        }
        const h = Math.floor(m / 60);
        const remainingM = m % 60;
        return `${h}h ${remainingM}m`;
    }

    async function resumeTask() {
        if (sending || !taskId) return;
        sending = true;
        try {
            const { db } = await import("$lib/services/db");
            const { emit } = await import("@tauri-apps/api/event");
            
            // Log the time away as a break
            if (awayTimeSeconds > 0) {
                try {
                    await db.timeEntries.logBreakTime(awayTimeSeconds);
                } catch (logErr) {
                    console.error("[resume] Error logging break time:", logErr);
                }
            }
            
            // Emitting to main window to cleanly handle local store update + DB update
            await emit("break:action", { action: "resume" });
            
            await db.window.focusMain();
            await db.window.closeResume();
        } catch (e) {
            console.error("[resume] Error resuming task:", e);
            sending = false;
        }
    }

    async function switchTask() {
        if (sending) return;
        sending = true;
        try {
            const { db } = await import("$lib/services/db");
            
            // Log the time away as a break
            if (awayTimeSeconds > 0) {
                await db.timeEntries.logBreakTime(awayTimeSeconds);
            }
            
            await db.window.focusMain();
            await db.window.closeResume();
        } catch (e) {
            console.error("[resume] Error switching task:", e);
            sending = false;
        }
    }

    async function dragWindow() {
        try {
            const { db } = await import("$lib/services/db");
            await db.window.startDragging();
        } catch {
            // ignore
        }
    }
</script>

<div class="shell" class:mounted>
    <!-- Drag region -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="drag-bar" onmousedown={dragWindow} role="presentation">
        <div class="drag-dots">
            <span></span><span></span><span></span>
        </div>
        <button
            class="close-btn"
            onclick={switchTask}
            title="Close"
            aria-label="Close resume window"
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
        <div class="icon-ring" in:scale={{ duration: 400, delay: 100 }}>
            <span class="icon">👋</span>
        </div>

        <div class="text-block">
            <h1 class="title">Welcome Back</h1>
            <p class="subtitle">
                You have been away for <strong>{formatTime(awayTimeSeconds)}</strong>.
            </p>
        </div>

        {#if taskId && taskTitle}
            <div class="task-info" in:fade={{ duration: 200, delay: 150 }}>
                <span class="task-label">Current Task</span>
                <span class="task-title-text">{taskTitle}</span>
            </div>
        {/if}

        <div class="actions" in:fade={{ duration: 200, delay: 200 }}>
            {#if taskId}
                <button
                    class="btn btn-primary"
                    onclick={resumeTask}
                    disabled={sending}
                >
                    <svg
                        width="14"
                        height="14"
                        viewBox="0 0 24 24"
                        fill="currentColor"><path d="M8 5v14l11-7z" /></svg
                    >
                    Resume Task
                </button>
            {/if}
            <button
                class="btn btn-ghost"
                onclick={switchTask}
                disabled={sending}
            >
                Switch to different task
            </button>
        </div>
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
        transition: background-color 0.15s, color 0.15s;
        flex-shrink: 0;
    }

    .close-btn:hover {
        background-color: #ef4444;
        color: #fff;
    }

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

    .icon-ring {
        width: 64px;
        height: 64px;
        border-radius: 50%;
        background: var(--accent-light, color-mix(in srgb, var(--accent) 12%, transparent));
        display: flex;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;
    }

    .icon {
        font-size: 30px;
        line-height: 1;
    }

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
        font-size: 13px;
        color: var(--text-secondary);
        line-height: 1.6;
        opacity: 0.85;
    }
    
    .subtitle strong {
        color: var(--text-primary);
        font-weight: 600;
    }

    .task-info {
        background: var(--bg-secondary);
        border: 1px solid var(--border);
        border-radius: 8px;
        padding: 10px 16px;
        display: flex;
        flex-direction: column;
        gap: 4px;
        width: 100%;
        text-align: left;
    }

    .task-label {
        font-size: 11px;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--text-tertiary);
        font-weight: 600;
    }

    .task-title-text {
        font-size: 14px;
        font-weight: 500;
        color: var(--text-primary);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .actions {
        display: flex;
        flex-direction: column;
        gap: 8px;
        width: 100%;
        margin-top: 4px;
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
        transition: background-color 0.15s, opacity 0.15s, transform 0.1s;
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
        color: #fff;
    }

    .btn-primary:hover:not(:disabled) {
        background-color: var(--accent-hover, var(--accent));
        filter: brightness(1.08);
    }

    .btn-ghost {
        background-color: transparent;
        color: var(--text-tertiary);
        font-weight: 500;
        border: 1px solid var(--border);
    }

    .btn-ghost:hover:not(:disabled) {
        background-color: var(--bg-hover);
        color: var(--text-primary);
    }
</style>
