<script lang="ts">
    import { onMount } from "svelte";
    import { fade, scale } from "svelte/transition";
    import { emit } from "@tauri-apps/api/event";
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

    let reasonOptions = $derived(afkCategoryStore.buildOptions(taskId !== null));
    let selectedReason = $derived(
        reasonOptions.find((option) => option.id === selectedCategoryId) ??
            reasonOptions[0] ??
            null,
    );
    let hasReasonOptions = $derived(reasonOptions.length > 0);
    let isCurrentTaskRelated = $derived(
        selectedReason?.id === CURRENT_TASK_RELATED_CATEGORY_ID,
    );
    let primaryActionLabel = $derived(
        isCurrentTaskRelated
            ? "Add to task and resume"
            : "Log reason and resume task",
    );
    let openAppLabel = $derived(
        hasReasonOptions
            ? taskId !== null
                ? "Save and open app"
                : "Save and continue"
            : "Open app",
    );
    let reasonHint = $derived.by(() => {
        if (!selectedReason) {
            return "No AFK categories are configured yet.";
        }

        if (selectedReason.id === CURRENT_TASK_RELATED_CATEGORY_ID) {
            return "Use this when the away time still belongs to the task you were working on.";
        }

        return `This time will be tracked under ${selectedReason.label}.`;
    });

    onMount(() => {
        const theme =
            window.__RESUME_DATA__?.theme ||
            localStorage.getItem("theme") ||
            "light";
        document.documentElement.setAttribute("data-theme", theme);

        afkCategoryStore.init();

        if (window.__RESUME_DATA__) {
            taskId = window.__RESUME_DATA__.taskId;
            taskTitle = window.__RESUME_DATA__.taskTitle;
            awayTimeSeconds = window.__RESUME_DATA__.awayTimeSeconds;
        }

        selectedCategoryId = afkCategoryStore.getDefaultSelection(
            taskId !== null,
        );

        requestAnimationFrame(() => {
            mounted = true;
        });
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

    async function logSelectedAwayTime() {
        if (awayTimeSeconds <= 0 || !selectedReason) {
            return;
        }

        if (
            selectedReason.id === CURRENT_TASK_RELATED_CATEGORY_ID &&
            taskId !== null
        ) {
            await db.timeEntries.createManualEntry(taskId, awayTimeSeconds);
            await emit("timer:away-time-logged", {
                affectsVisibleTaskTotals: true,
            });
            return;
        }

        await db.timeEntries.logAfkTime(selectedReason.label, awayTimeSeconds);
        await emit("timer:away-time-logged", {
            affectsVisibleTaskTotals: false,
        });
    }

    async function completeAndResume() {
        if (sending) return;
        sending = true;

        try {
            await logSelectedAwayTime();

            if (taskId !== null) {
                await db.timer.resume();
                await emit("break:action", { action: "resume" });
            }

            await db.window.focusMain();
            await db.window.closeResume();
        } catch (error) {
            console.error("[resume] Error saving away time and resuming:", error);
            sending = false;
        }
    }

    async function openApp() {
        if (sending) return;
        sending = true;

        try {
            await logSelectedAwayTime();
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
            onclick={openApp}
            title="Open app"
            aria-label="Open app"
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
        <div class="icon-ring" in:scale={{ duration: 400, delay: 100 }}>
            <span class="icon">👋</span>
        </div>

        <div class="text-block">
            <h1 class="title">Welcome Back</h1>
            <p class="subtitle">
                You were away for <strong>{formatTime(awayTimeSeconds)}</strong>.
                Pick the best match so we can log the time correctly.
            </p>
        </div>

        {#if taskId !== null && taskTitle}
            <div class="task-info" in:fade={{ duration: 200, delay: 150 }}>
                <span class="task-label">Current Task</span>
                <span class="task-title-text">{taskTitle}</span>
            </div>
        {/if}

        {#if hasReasonOptions}
            <div class="reason-panel" in:fade={{ duration: 200, delay: 175 }}>
                <label class="reason-label" for="afk-reason-select">
                    Why were you away?
                </label>
                <select
                    id="afk-reason-select"
                    class="reason-select"
                    bind:value={selectedCategoryId}
                    disabled={sending}
                >
                    {#each reasonOptions as option}
                        <option value={option.id}>{option.label}</option>
                    {/each}
                </select>
                <p class="reason-hint">{reasonHint}</p>
            </div>
        {:else}
            <div class="reason-empty" in:fade={{ duration: 200, delay: 175 }}>
                Add AFK categories in Settings to track this time here.
            </div>
        {/if}

        <div class="actions" in:fade={{ duration: 200, delay: 220 }}>
            {#if taskId !== null}
                <button
                    class="btn btn-primary"
                    onclick={completeAndResume}
                    disabled={sending}
                >
                    <svg
                        width="14"
                        height="14"
                        viewBox="0 0 24 24"
                        fill="currentColor"><path d="M8 5v14l11-7z" /></svg
                    >
                    {primaryActionLabel}
                </button>
            {/if}

            <button
                class="btn"
                class:btn-primary={taskId === null}
                class:btn-ghost={taskId !== null}
                onclick={openApp}
                disabled={sending}
            >
                {openAppLabel}
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
        justify-content: center;
        padding: 24px 28px 28px;
        gap: 14px;
        text-align: center;
    }

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

    .task-info,
    .reason-panel {
        background: var(--bg-secondary);
        border: 1px solid var(--border);
        border-radius: 10px;
        padding: 12px 14px;
        display: flex;
        flex-direction: column;
        gap: 6px;
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

    .reason-select {
        width: 100%;
        border-radius: 8px;
        border: 1px solid var(--border);
        background: var(--bg-primary);
        color: var(--text-primary);
        padding: 10px 12px;
        font-size: 13px;
        font-family: inherit;
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
        margin-top: 2px;
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
