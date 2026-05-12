<script lang="ts">
    import { onMount } from "svelte";
    import { fade, scale } from "svelte/transition";
    import { emit, listen } from "@tauri-apps/api/event";
    import { db } from "$lib/services/db";
    import {
        afkCategoryStore,
        CURRENT_TASK_RELATED_CATEGORY_ID,
    } from "$lib/stores/afk-categories.svelte";

    let taskId = $state<number | null>(null);
    let taskTitle = $state("");
    let awayTimeSeconds = $state(0);
    let selectedCategoryId = $state("");
    let sending = $state(false);
    let mounted = $state(false);

    let hasSelectableReasons = $derived(
        taskId !== null || afkCategoryStore.customCategories.length > 0,
    );
    let isCurrentTaskRelated = $derived(
        selectedCategoryId === CURRENT_TASK_RELATED_CATEGORY_ID,
    );
    let reasonHint = $derived.by(() => {
        if (isCurrentTaskRelated) {
            return taskTitle
                ? `Away time will be added to ${taskTitle} before you continue.`
                : "Away time will be added to the current task before you continue.";
        }

        if (selectedCategoryId) {
            return `Away time will be tracked under ${selectedCategoryId}.`;
        }

        return "If you leave this unselected, the away time will be logged as Break.";
    });

    onMount(() => {
        const theme =
            (typeof window.__RESUME_DATA__?.theme === "string" && window.__RESUME_DATA__.theme) ||
            localStorage.getItem("theme") ||
            "light";
        document.documentElement.setAttribute("data-theme", theme);

        afkCategoryStore.init();

        if (window.__RESUME_DATA__) {
            taskId = window.__RESUME_DATA__.taskId;
            taskTitle = window.__RESUME_DATA__.taskTitle;
            awayTimeSeconds = window.__RESUME_DATA__.awayTimeSeconds;
        }

        const unlistenPromise = listen<{
            taskId: number | null;
            taskTitle: string;
            awayTimeSeconds: number;
        }>("resume:update", (event) => {
            if (!sending) {
                taskId = event.payload.taskId;
                taskTitle = event.payload.taskTitle;
                awayTimeSeconds = event.payload.awayTimeSeconds;
                selectedCategoryId = "";
            }
        });

        requestAnimationFrame(() => {
            mounted = true;
        });

        return () => {
            unlistenPromise.then((unlisten) => unlisten());
        };
    });

    function formatTime(seconds: number): string {
        if (seconds < 60) {
            return `${seconds} sec`;
        }
        const minutes = Math.floor(seconds / 60);
        const remainingSeconds = seconds % 60;
        if (minutes < 60) {
            return `${minutes}m ${remainingSeconds}s`;
        }
        const hours = Math.floor(minutes / 60);
        const remainingMinutes = minutes % 60;
        return `${hours}h ${remainingMinutes}m`;
    }

    function toggleCategory(categoryId: string) {
        if (sending) return;
        selectedCategoryId =
            selectedCategoryId === categoryId ? "" : categoryId;
    }

    async function logSelectedAwayTime() {
        if (awayTimeSeconds <= 0) {
            return;
        }

        if (isCurrentTaskRelated && taskId !== null) {
            await db.timeEntries.createManualEntry(taskId, awayTimeSeconds);
            await emit("timer:away-time-logged", {
                affectsVisibleTaskTotals: true,
            });
            return;
        }

        if (selectedCategoryId) {
            await db.timeEntries.logAfkTime(selectedCategoryId, awayTimeSeconds);
            await emit("timer:away-time-logged", {
                affectsVisibleTaskTotals: false,
            });
            return;
        }

        await db.timeEntries.logBreakTime(awayTimeSeconds);
        await emit("timer:away-time-logged", {
            affectsVisibleTaskTotals: false,
        });
    }

    async function resumeTask() {
        if (sending) return;
        sending = true;

        try {
            await logSelectedAwayTime();

            if (taskId !== null) {
                await db.timer.resume();
                await emit("break:action", { action: "resume" });
            }
            await db.windowTracking.setPaused(false);

            await db.window.focusMain();
            await db.window.closeResume();
        } catch (error) {
            console.error("[resume] Error saving away time and resuming:", error);
            sending = false;
        }
    }

    async function switchTask() {
        if (sending) return;
        sending = true;

        try {
            await logSelectedAwayTime();
            await db.windowTracking.setPaused(false);
            await db.window.focusMain();
            await db.window.closeResume();
        } catch (error) {
            console.error("[resume] Error saving away time:", error);
            sending = false;
        }
    }

    async function dragWindow() {
        try {
            await db.window.startDragging();
        } catch {
            // ignore
        }
    }
</script>

<div class="shell" class:mounted>
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

    <div class="content">
        <div class="content-main">
            <div class="icon-ring" in:scale={{ duration: 400, delay: 100 }}>
                <span class="icon">👋</span>
            </div>

            <div class="text-block">
                <h1 class="title">Welcome Back</h1>
                <p class="subtitle">
                    You were away for
                    <strong>{formatTime(awayTimeSeconds)}</strong>.
                </p>
            </div>

            {#if taskId !== null && taskTitle}
                <div class="task-info" in:fade={{ duration: 200, delay: 150 }}>
                    <span class="task-label">Current Task</span>
                    <span class="task-title-text">{taskTitle}</span>
                </div>
            {/if}

            {#if hasSelectableReasons}
                <div class="reason-panel" in:fade={{ duration: 200, delay: 175 }}>
                    <span class="reason-label">AFK Category</span>
                    <div class="break-default-pill">Default: Break</div>
                    <div class="reason-chip-list">
                        {#if taskId !== null}
                            <button
                                type="button"
                                class="reason-chip reason-chip-current"
                                class:active={isCurrentTaskRelated}
                                onclick={() =>
                                    toggleCategory(
                                        CURRENT_TASK_RELATED_CATEGORY_ID,
                                    )}
                                disabled={sending}
                            >
                                Current task related
                            </button>
                        {/if}

                        {#each afkCategoryStore.customCategories as category}
                            <button
                                type="button"
                                class="reason-chip"
                                class:active={selectedCategoryId === category}
                                onclick={() => toggleCategory(category)}
                                disabled={sending}
                            >
                                {category}
                            </button>
                        {/each}
                    </div>
                    <p class="reason-hint">{reasonHint}</p>
                </div>
            {:else}
                <div class="reason-empty" in:fade={{ duration: 200, delay: 175 }}>
                    No AFK categories yet. Away time will be logged as Break.
                </div>
            {/if}
        </div>

        <div class="actions" in:fade={{ duration: 200, delay: 220 }}>
            {#if taskId !== null}
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
                class="btn"
                class:btn-primary={taskId === null}
                class:btn-ghost={taskId !== null}
                onclick={switchTask}
                disabled={sending}
            >
                {taskId !== null ? "Switch to different task" : "Open app"}
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
        background-color: var(--danger);
        color: var(--danger-contrast);
    }

    .content {
        flex: 1;
        display: flex;
        flex-direction: column;
        align-items: center;
        padding: 18px 22px 22px;
        gap: 12px;
        text-align: center;
        overflow: hidden;
        min-height: 0;
    }

    .content-main {
        width: 100%;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 12px;
        min-height: 0;
        overflow-y: auto;
        padding: 2px 4px 0 0;
    }

    .icon-ring {
        width: 56px;
        height: 56px;
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
        font-size: 28px;
        line-height: 1;
    }

    .text-block {
        display: flex;
        flex-direction: column;
        gap: 4px;
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

    .task-info,
    .reason-panel {
        background: var(--bg-secondary);
        border: 1px solid var(--border);
        border-radius: 10px;
        padding: 11px 13px;
        display: flex;
        flex-direction: column;
        gap: 8px;
        width: 100%;
        text-align: left;
    }

    .task-label,
    .reason-label {
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

    .break-default-pill {
        align-self: flex-start;
        padding: 6px 10px;
        border-radius: 999px;
        border: 1px solid color-mix(in srgb, var(--accent) 22%, var(--border));
        background: color-mix(in srgb, var(--accent) 10%, var(--bg-primary));
        color: var(--text-secondary);
        font-size: 11px;
        font-weight: 600;
    }

    .reason-chip-list {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
        max-height: 110px;
        overflow-y: auto;
        padding-right: 4px;
    }

    .reason-chip {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        padding: 6px 12px;
        border-radius: 999px;
        border: 1px solid var(--border);
        background: var(--bg-primary);
        color: var(--text-secondary);
        font-size: 12px;
        font-weight: 500;
        cursor: pointer;
        transition:
            background-color 0.15s,
            border-color 0.15s,
            color 0.15s,
            transform 0.1s;
    }

    .reason-chip:hover:not(:disabled) {
        border-color: var(--accent);
        color: var(--text-primary);
    }

    .reason-chip:active:not(:disabled) {
        transform: scale(0.98);
    }

    .reason-chip:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .reason-chip.active {
        border-color: var(--accent);
        background: color-mix(in srgb, var(--accent) 15%, var(--bg-primary));
        color: var(--text-primary);
    }

    .reason-chip-current {
        font-weight: 600;
    }

    .reason-hint,
    .reason-empty {
        font-size: 12px;
        color: var(--text-secondary);
        line-height: 1.5;
    }

    .reason-empty {
        width: 100%;
        padding: 12px 14px;
        border-radius: 10px;
        border: 1px dashed var(--border);
        background: color-mix(in srgb, var(--bg-secondary) 60%, transparent);
    }

    .actions {
        display: flex;
        flex-direction: column;
        gap: 8px;
        width: 100%;
        margin-top: auto;
        padding-top: 12px;
        border-top: 1px solid var(--border-light, var(--border));
        flex-shrink: 0;
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
        border: 1px solid var(--border);
    }

    .btn-ghost:hover:not(:disabled) {
        background-color: var(--bg-hover);
        color: var(--text-primary);
    }
</style>
