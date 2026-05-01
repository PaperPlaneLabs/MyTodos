<script lang="ts">
    import { onMount } from "svelte";
    import { fade, fly } from "svelte/transition";
    import {
        uiStore,
        type Theme,
        type WindowOrientation,
    } from "$lib/stores/ui.svelte";
    import { googleCalendarStore } from "$lib/stores/google-calendar.svelte";
    import { projectStore } from "$lib/stores/projects.svelte";
    import { timerStore } from "$lib/stores/timer.svelte";
    import { windowTrackingStore } from "$lib/stores/window-tracking.svelte";
    import { afkCategoryStore } from "$lib/stores/afk-categories.svelte";
    import { db } from "$lib/services/db";
    import { getVersion } from "@tauri-apps/api/app";
    import { check } from "@tauri-apps/plugin-updater";
    import { relaunch } from "@tauri-apps/plugin-process";
    import Modal from "$lib/components/common/Modal.svelte";

    const themes: { id: Theme; name: string; bg: string; accent: string }[] = [
        { id: "light", name: "Light", bg: "#ffffff", accent: "#6366f1" },
        { id: "dark", name: "Dark", bg: "#1a1a1a", accent: "#818cf8" },
        {
            id: "minecraft",
            name: "Minecraft",
            bg: "#3d2b1f",
            accent: "#3c8527",
        },
        { id: "retro", name: "Retro", bg: "#0c0c0c", accent: "#ffb000" },
        { id: "ocean", name: "Ocean", bg: "#001219", accent: "#0a9396" },
        { id: "nord", name: "Nord", bg: "#2e3440", accent: "#88c0d0" },
    ];
    const breakReminderOptions: { value: number; label: string }[] = [
        { value: 0, label: "Not Tracked" },
        { value: 15, label: "15 minutes" },
        { value: 20, label: "20 minutes" },
        { value: 25, label: "25 minutes" },
        { value: 30, label: "30 minutes" },
        { value: 45, label: "45 minutes" },
        { value: 60, label: "60 minutes" },
    ];
    let currentThemePreview = $derived(
        themes.find((t) => t.id === uiStore.theme) ?? themes[0],
    );

    let isAutoStartEnabled = $state(false);
    let togglingWindowTracking = $state(false);
    let loading = $state(true);
    let toggling = $state(false);
    let showResetConfirm = $state(false);
    let showWindowTrackConfirm = $state(false);
    let showBreakIntervalConfirm = $state(false);
    let pendingBreakInterval = $state<number | null>(null);
    let breakIntervalSelectValue = $state("30");
    let newAfkCategory = $state("");
    let afkCategoryError = $state("");

    let appVersion = $state("");
    let updateStatus = $state<
        | "idle"
        | "checking"
        | "up-to-date"
        | "available"
        | "downloading"
        | "error"
    >("idle");
    let updateVersion = $state("");
    let updateError = $state("");
    let updateProgress = $state(0);

    onMount(async () => {
        afkCategoryStore.init();
        await windowTrackingStore.init();
        breakIntervalSelectValue = String(
            timerStore.breakReminderIntervalMinutes,
        );

        try {
            appVersion = await getVersion();
        } catch (e) {
            console.error("Failed to get app version:", e);
            appVersion = "unknown";
        }

        try {
            const { isEnabled } = await import("@tauri-apps/plugin-autostart");
            isAutoStartEnabled = await isEnabled();
        } catch (e) {
            console.error("Failed to check autostart status:", e);
        } finally {
            loading = false;
        }
    });

    async function checkForUpdates() {
        updateStatus = "checking";
        updateError = "";

        try {
            const update = await check();
            if (update) {
                updateVersion = update.version;
                updateStatus = "available";
            } else {
                updateStatus = "up-to-date";
            }
        } catch (e) {
            updateError = e instanceof Error ? e.message : String(e);
            updateStatus = "error";
        }
    }

    async function downloadAndInstallUpdate() {
        updateStatus = "downloading";
        updateProgress = 0;

        try {
            const update = await check();
            if (!update) return;

            let contentLength = 0;
            let totalDownloaded = 0;

            await update.downloadAndInstall((event) => {
                if (event.event === "Started") {
                    contentLength = event.data.contentLength ?? 0;
                    totalDownloaded = 0;
                    updateProgress = 0;
                } else if (event.event === "Progress") {
                    totalDownloaded += event.data.chunkLength;
                    if (contentLength > 0) {
                        updateProgress = Math.min(
                            Math.round((totalDownloaded / contentLength) * 100),
                            99,
                        );
                    } else {
                        updateProgress = Math.min(updateProgress + 3, 95);
                    }
                } else if (event.event === "Finished") {
                    updateProgress = 100;
                }
            });

            await relaunch();
        } catch (e) {
            updateError = e instanceof Error ? e.message : String(e);
            updateStatus = "error";
        }
    }

    async function toggleAutoStart() {
        if (toggling) return;
        toggling = true;

        try {
            const autostart = await import("@tauri-apps/plugin-autostart");

            if (isAutoStartEnabled) {
                await autostart.disable();
                isAutoStartEnabled = false;
            } else {
                await autostart.enable();
                isAutoStartEnabled = true;
            }
        } catch (e) {
            console.error("Failed to toggle autostart:", e);
        } finally {
            toggling = false;
        }
    }

    async function handleClearData() {
        try {
            if (timerStore.active) {
                await timerStore.stop();
            }
            await windowTrackingStore.clearActivity();
            const projects = [...projectStore.projects];
            for (const p of projects) {
                await projectStore.delete(p.id);
            }
            await projectStore.loadAll();
            showResetConfirm = false;
        } catch (e) {
            console.error("Failed to clear data:", e);
        }
    }

    async function toggleWindowTracking() {
        if (togglingWindowTracking) return;

        if (!windowTrackingStore.enabled) {
            showWindowTrackConfirm = true;
            return;
        }

        await setWindowTrackingEnabled(false);
    }

    async function setWindowTrackingEnabled(enabled: boolean) {
        togglingWindowTracking = true;

        try {
            if (enabled && timerStore.active) {
                await timerStore.stop();
            }

            await windowTrackingStore.setEnabled(enabled);
            showWindowTrackConfirm = false;
        } catch (e) {
            console.error("Failed to toggle window tracking:", e);
        } finally {
            togglingWindowTracking = false;
        }
    }

    function cancelWindowTrackingEnable() {
        if (togglingWindowTracking) return;
        showWindowTrackConfirm = false;
    }

    function selectTheme(id: Theme) {
        uiStore.setTheme(id);
    }

    async function handleWindowOrientation(orientation: WindowOrientation) {
        try {
            if (orientation === "left") {
                await db.window.dock("left");
            } else if (orientation === "right") {
                await db.window.dock("right");
            } else {
                await db.window.center();
            }

            await db.window.setDockPreference(orientation);
            uiStore.setWindowOrientation(orientation);
        } catch (e) {
            console.error("Failed to update window orientation:", e);
        }
    }

    function updateBreakReminderInterval(event: Event) {
        const target = event.currentTarget as HTMLSelectElement;
        const nextInterval = Number(target.value);
        if (!Number.isFinite(nextInterval) || nextInterval < 0) {
            breakIntervalSelectValue = String(
                timerStore.breakReminderIntervalMinutes,
            );
            return;
        }

        if (nextInterval === timerStore.breakReminderIntervalMinutes) {
            breakIntervalSelectValue = String(
                timerStore.breakReminderIntervalMinutes,
            );
            return;
        }

        // "Not Tracked" (0) applies immediately — no confirm needed
        if (nextInterval === 0) {
            timerStore.setBreakReminderInterval(0);
            breakIntervalSelectValue = "0";
            return;
        }

        pendingBreakInterval = nextInterval;
        showBreakIntervalConfirm = true;
    }

    function confirmBreakIntervalChange() {
        if (pendingBreakInterval === null) return;
        timerStore.setBreakReminderInterval(pendingBreakInterval);
        breakIntervalSelectValue = String(pendingBreakInterval);
        pendingBreakInterval = null;
        showBreakIntervalConfirm = false;
    }

    function cancelBreakIntervalChange() {
        pendingBreakInterval = null;
        showBreakIntervalConfirm = false;
        breakIntervalSelectValue = String(
            timerStore.breakReminderIntervalMinutes,
        );
    }

    function addAfkCategory() {
        const result = afkCategoryStore.addCategory(newAfkCategory);
        if (!result.added) {
            afkCategoryError = result.error;
            return;
        }

        afkCategoryError = "";
        newAfkCategory = "";
    }

    function removeAfkCategory(category: string) {
        afkCategoryStore.removeCategory(category);
        afkCategoryError = "";
    }

    function handleAfkCategoryKeydown(event: KeyboardEvent) {
        if (event.key !== "Enter") return;
        event.preventDefault();
        addAfkCategory();
    }
</script>

<div class="settings-view" transition:fade={{ duration: 200 }}>
    <header class="settings-header">
        <button
            type="button"
            class="back-btn"
            onclick={() => uiStore.closeSettingsView()}
        >
            <svg
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                aria-hidden="true"
            >
                <path d="m15 18-6-6 6-6" />
            </svg>
            <span>Back</span>
        </button>
        <h2>Settings</h2>
    </header>

    <div class="settings-content">
        <!-- GENERAL SECTION -->
        <section
            class="settings-section"
            transition:fly={{ y: 20, duration: 300, delay: 100 }}
        >
            <h3><span class="section-icon">⚙️</span> General</h3>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label" id="start-at-login-label"
                        >Start at login</span
                    >
                    <span class="setting-desc"
                        >Automatically launch MyTodos when you log in</span
                    >
                </div>
                <button
                    type="button"
                    class="toggle-switch"
                    class:active={isAutoStartEnabled}
                    class:loading={loading || toggling}
                    role="switch"
                    aria-checked={isAutoStartEnabled}
                    aria-labelledby="start-at-login-label"
                    onclick={toggleAutoStart}
                    disabled={loading || toggling}
                    title={isAutoStartEnabled
                        ? "Disable autostart"
                        : "Enable autostart"}
                >
                    <span class="toggle-knob"></span>
                </button>
            </div>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">Window Docking</span>
                    <span class="setting-desc">Position and snap window</span>
                </div>
                <div class="segmented-control">
                    <button
                        type="button"
                        class:active={uiStore.windowOrientation === "left"}
                        aria-pressed={uiStore.windowOrientation === "left"}
                        onclick={() => handleWindowOrientation("left")}
                        >Left</button
                    >
                    <button
                        type="button"
                        class:active={uiStore.windowOrientation === "center"}
                        aria-pressed={uiStore.windowOrientation === "center"}
                        onclick={() => handleWindowOrientation("center")}
                        >FreeForm</button
                    >
                    <button
                        type="button"
                        class:active={uiStore.windowOrientation === "right"}
                        aria-pressed={uiStore.windowOrientation === "right"}
                        onclick={() => handleWindowOrientation("right")}
                        >Right</button
                    >
                </div>
            </div>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label" id="compact-mode-label"
                        >Compact Mode</span
                    >
                    <span class="setting-desc"
                        >Reduce spacing for a denser layout</span
                    >
                </div>
                <button
                    type="button"
                    class="toggle-switch"
                    class:active={uiStore.compactMode}
                    role="switch"
                    aria-checked={uiStore.compactMode}
                    aria-labelledby="compact-mode-label"
                    onclick={() => uiStore.setCompactMode(!uiStore.compactMode)}
                    title={uiStore.compactMode
                        ? "Disable compact mode"
                        : "Enable compact mode"}
                >
                    <span class="toggle-knob"></span>
                </button>
            </div>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label" id="break-reminders-label"
                        >Break Reminders</span
                    >
                    <span class="setting-desc"
                        >Show a break prompt while a timer is running</span
                    >
                </div>
                <button
                    type="button"
                    class="toggle-switch"
                    class:active={timerStore.breakReminderEnabled}
                    role="switch"
                    aria-checked={timerStore.breakReminderEnabled}
                    aria-labelledby="break-reminders-label"
                    onclick={() =>
                        timerStore.setBreakReminderEnabled(
                            !timerStore.breakReminderEnabled,
                        )}
                    title={timerStore.breakReminderEnabled
                        ? "Disable break reminders"
                        : "Enable break reminders"}
                >
                    <span class="toggle-knob"></span>
                </button>
            </div>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label" id="window-track-label"
                        >Window Track</span
                    >
                    <span class="setting-desc">
                        Track active foreground application time. Project/task
                        timers are disabled while this is on.
                    </span>
                </div>
                <button
                    type="button"
                    class="toggle-switch"
                    class:active={windowTrackingStore.enabled}
                    class:loading={togglingWindowTracking}
                    role="switch"
                    aria-checked={windowTrackingStore.enabled}
                    aria-labelledby="window-track-label"
                    onclick={toggleWindowTracking}
                    disabled={togglingWindowTracking}
                    title={windowTrackingStore.enabled
                        ? "Disable window tracking"
                        : "Enable window tracking"}
                >
                    <span class="toggle-knob"></span>
                </button>
            </div>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label" id="break-interval-label"
                        >Break Interval</span
                    >
                    <span class="setting-desc">
                        {#if timerStore.breakReminderIntervalMinutes === 0}
                            Break time is not tracked
                        {:else if timerStore.breakReminderEnabled}
                            Remind every {timerStore.breakReminderIntervalMinutes}
                            minutes
                        {:else}
                            Reminders are currently disabled
                        {/if}
                    </span>
                </div>
                <div class="setting-select">
                    <select
                        id="break-interval-select"
                        class="input setting-native-select"
                        bind:value={breakIntervalSelectValue}
                        aria-labelledby="break-interval-label"
                        onchange={updateBreakReminderInterval}
                    >
                        {#each breakReminderOptions as opt}
                            <option value={String(opt.value)}>{opt.label}</option>
                        {/each}
                    </select>
                </div>
            </div>

            <div class="setting-item setting-item-stack">
                <div class="setting-info">
                    <span class="setting-label" id="afk-categories-label"
                        >AFK Categories</span
                    >
                    <span class="setting-desc">
                        Used by the welcome-back screen to categorize away time.
                        <strong>Current task related</strong> is always available
                        when a timer exists.
                    </span>
                </div>

                <div class="afk-category-manager" aria-labelledby="afk-categories-label">
                    <div class="afk-built-in-pill">
                        Built in: Current task related
                    </div>

                    {#if afkCategoryStore.customCategories.length > 0}
                        <div class="afk-category-list">
                            {#each afkCategoryStore.customCategories as category}
                                <div class="afk-category-pill">
                                    <span>{category}</span>
                                    <button
                                        type="button"
                                        class="afk-remove-btn"
                                        aria-label={`Remove ${category} AFK category`}
                                        title={`Remove ${category}`}
                                        onclick={() => removeAfkCategory(category)}
                                    >
                                        <svg
                                            width="10"
                                            height="10"
                                            viewBox="0 0 10 10"
                                            fill="none"
                                            aria-hidden="true"
                                        >
                                            <path
                                                d="M1 1l8 8M9 1l-8 8"
                                                stroke="currentColor"
                                                stroke-width="1.5"
                                                stroke-linecap="round"
                                            />
                                        </svg>
                                    </button>
                                </div>
                            {/each}
                        </div>
                    {:else}
                        <p class="afk-category-empty">
                            No custom AFK categories yet.
                        </p>
                    {/if}

                    <div class="afk-category-add-row">
                        <input
                            class="input afk-category-input"
                            type="text"
                            placeholder="Add category like Meeting"
                            bind:value={newAfkCategory}
                            onkeydown={handleAfkCategoryKeydown}
                        />
                        <button
                            type="button"
                            class="btn btn-secondary btn-sm"
                            onclick={addAfkCategory}
                        >
                            Add
                        </button>
                    </div>

                    {#if afkCategoryError}
                        <p class="afk-category-error" role="alert">
                            {afkCategoryError}
                        </p>
                    {/if}
                </div>
            </div>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">App Updates</span>
                    <span
                        class="setting-desc"
                        class:update-error={updateStatus === "error"}
                        class:update-success={updateStatus === "up-to-date"}
                        role="status"
                        aria-live="polite"
                    >
                        {#if updateStatus === "checking"}
                            Checking...
                        {:else if updateStatus === "up-to-date"}
                            Up to date
                        {:else if updateStatus === "available"}
                            Update: v{updateVersion}
                        {:else if updateStatus === "downloading"}
                            Downloading... {updateProgress}%
                        {:else if updateStatus === "error"}
                            {updateError}
                        {:else}
                            v{appVersion}
                        {/if}
                    </span>
                </div>
                {#if updateStatus === "available"}
                    <button
                        type="button"
                        class="btn btn-primary btn-sm"
                        onclick={downloadAndInstallUpdate}>Update</button
                    >
                {:else if updateStatus === "downloading"}
                    <div class="update-progress-inline">
                        <div
                            class="progress-bar-sm"
                            role="progressbar"
                            aria-label="Update download progress"
                            aria-valuemin="0"
                            aria-valuemax="100"
                            aria-valuenow={updateProgress}
                        >
                            <div
                                class="progress-fill-sm"
                                style="width: {updateProgress}%"
                            ></div>
                        </div>
                    </div>
                {:else}
                    <button
                        type="button"
                        class="btn btn-secondary btn-sm"
                        onclick={checkForUpdates}
                        disabled={updateStatus === "checking"}
                    >
                        Check
                    </button>
                {/if}
            </div>
        </section>

        <!-- APPEARANCE SECTION -->
        <section
            class="settings-section"
            transition:fly={{ y: 20, duration: 300, delay: 150 }}
        >
            <h3><span class="section-icon">🎨</span> Appearance</h3>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label" id="theme-label">Theme</span>
                    <span class="setting-desc">Customize application look</span>
                </div>

                <div class="setting-select theme-select">
                    <select
                        id="theme-select"
                        class="input setting-native-select"
                        value={uiStore.theme}
                        aria-labelledby="theme-label"
                        onchange={(event) =>
                            selectTheme(
                                (event.currentTarget as HTMLSelectElement)
                                    .value as Theme,
                            )}
                    >
                        {#each themes as t}
                            <option value={t.id}>{t.name}</option>
                        {/each}
                    </select>
                    <div
                        class="preview-dot"
                        aria-hidden="true"
                        style="--preview-bg: {currentThemePreview.bg}; --preview-accent: {currentThemePreview.accent}"
                    ></div>
                </div>
            </div>
        </section>

        <!-- INTEGRATIONS SECTION -->
        <section
            class="settings-section"
            transition:fly={{ y: 20, duration: 300, delay: 200 }}
        >
            <h3><span class="section-icon">🔗</span> Integrations</h3>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">Google Calendar</span>
                    <span class="setting-desc">
                        {#if !googleCalendarStore.connected}
                            Sync tasks as events
                        {:else}
                            <span class="gcal-connected">Connected</span>
                        {/if}
                    </span>
                </div>
                {#if !googleCalendarStore.connected}
                    <button
                        type="button"
                        class="btn btn-primary btn-sm"
                        onclick={() => googleCalendarStore.connect()}
                        disabled={googleCalendarStore.connecting}
                    >
                        {googleCalendarStore.connecting ? "..." : "Connect"}
                    </button>
                {:else}
                    <div style="display:flex; gap: 8px;">
                        <button
                            type="button"
                            class="btn btn-secondary btn-sm"
                            onclick={() => googleCalendarStore.syncAll()}
                            disabled={googleCalendarStore.syncing}
                        >
                            {googleCalendarStore.syncing ? "..." : "Sync"}
                        </button>
                        <button
                            type="button"
                            class="btn btn-secondary btn-sm"
                            onclick={() => googleCalendarStore.disconnect()}
                        >
                            Disconnect
                        </button>
                    </div>
                {/if}
            </div>
            {#if googleCalendarStore.error}
                <div class="gcal-error" role="alert">
                    {googleCalendarStore.error}
                </div>
            {/if}
        </section>

        <!-- DATA MANAGEMENT SECTION -->
        <section
            class="settings-section danger-zone"
            transition:fly={{ y: 20, duration: 300, delay: 250 }}
        >
            <h3><span class="section-icon">⚠️</span> Data Management</h3>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label text-danger"
                        >Reset Application</span
                    >
                    <span class="setting-desc"
                        >Permanently delete all projects and tasks</span
                    >
                </div>
                <button
                    type="button"
                    class="btn btn-danger btn-sm"
                    onclick={() => (showResetConfirm = true)}
                >
                    Reset Data
                </button>
            </div>
        </section>

        <!-- ABOUT SECTION -->
        <section
            class="settings-section about"
            transition:fly={{ y: 20, duration: 300, delay: 300 }}
        >
            <p class="version">MyTodos v{appVersion}</p>
        </section>
    </div>
</div>

<Modal
    open={showResetConfirm}
    title="⚠️ Irreversible Action"
    onClose={() => (showResetConfirm = false)}
>
    {#snippet children()}
        <div class="reset-confirm-content">
            <p
                style="margin-bottom: 20px; color: var(--text-secondary); line-height: 1.5;"
            >
                Are you absolutely sure you want to delete all data? This action
                cannot be undone.
            </p>
            <div style="display: flex; gap: 10px; justify-content: flex-end;">
                <button
                    class="btn btn-secondary"
                    onclick={() => (showResetConfirm = false)}>Cancel</button
                >
                <button class="btn btn-danger" onclick={handleClearData}
                    >Yes, Delete Everything</button
                >
            </div>
        </div>
    {/snippet}
</Modal>

<Modal
    open={showBreakIntervalConfirm}
    title="Set Break Interval?"
    onClose={cancelBreakIntervalChange}
>
    {#snippet children()}
        <div class="reset-confirm-content">
            <p
                style="margin-bottom: 20px; color: var(--text-secondary); line-height: 1.5;"
            >
                Set break reminders to every {pendingBreakInterval ??
                    timerStore.breakReminderIntervalMinutes} minutes?
            </p>
            <div style="display: flex; gap: 10px; justify-content: flex-end;">
                <button
                    class="btn btn-secondary"
                    onclick={cancelBreakIntervalChange}>No</button
                >
                <button
                    class="btn btn-primary"
                    onclick={confirmBreakIntervalChange}>Yes</button
                >
            </div>
        </div>
    {/snippet}
</Modal>

<Modal
    open={showWindowTrackConfirm}
    title="Enable Window Track?"
    onClose={cancelWindowTrackingEnable}
>
    {#snippet children()}
        <div class="reset-confirm-content">
            <p class="window-track-confirm-text">
                Window Track records time spent in the active foreground
                application, such as VS Code, Chrome, or Excel. It stores app
                names only, not window titles.
            </p>
            <div class="window-track-confirm-list">
                <p>When enabled:</p>
                <ul>
                    <li>Project and task timers are disabled.</li>
                    <li>Stats and graphs switch to application time.</li>
                    <li>Break reminders continue from active work time.</li>
                    <li>AFK categories still work when you return.</li>
                </ul>
            </div>
            <div style="display: flex; gap: 10px; justify-content: flex-end;">
                <button
                    class="btn btn-secondary"
                    onclick={cancelWindowTrackingEnable}
                    disabled={togglingWindowTracking}>Cancel</button
                >
                <button
                    class="btn btn-primary"
                    onclick={() => setWindowTrackingEnabled(true)}
                    disabled={togglingWindowTracking}
                >
                    {togglingWindowTracking ? "Enabling..." : "Enable Window Track"}
                </button>
            </div>
        </div>
    {/snippet}
</Modal>

<style>
    .settings-view {
        display: flex;
        flex-direction: column;
        height: 100%;
        overflow: hidden;
        background-color: var(--bg-primary);
    }

    .settings-header {
        display: flex;
        align-items: center;
        gap: var(--spacing-md);
        padding: var(--spacing-md);
        border-bottom: 1px solid var(--border);
        background-color: var(--bg-secondary);
    }

    .settings-header h2 {
        font-size: 16px;
        font-weight: 600;
        color: var(--text-primary);
    }

    .back-btn {
        display: flex;
        align-items: center;
        gap: 4px;
        padding: 6px 12px;
        border-radius: var(--radius-md);
        color: var(--text-secondary);
        background-color: var(--bg-primary);
        border: 1px solid var(--border);
        font-size: 13px;
        font-weight: 500;
        transition: all var(--transition-fast);
    }

    .back-btn:hover {
        background-color: var(--bg-hover);
        color: var(--text-primary);
    }

    .settings-content {
        flex: 1;
        overflow-y: auto;
        padding: var(--spacing-md);
        display: flex;
        flex-direction: column;
        gap: var(--spacing-lg);
    }

    .settings-section {
        background-color: var(--bg-secondary);
        border: 1px solid var(--border);
        border-radius: var(--radius-lg);
        padding: var(--spacing-md);
    }

    .settings-section.danger-zone {
        border-color: var(--danger-light);
        background-color: color-mix(
            in srgb,
            var(--danger-light) 10%,
            var(--bg-secondary)
        );
    }

    .settings-section h3 {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
        font-size: 14px;
        font-weight: 600;
        color: var(--text-primary);
        margin-bottom: var(--spacing-md);
        opacity: 0.8;
    }

    .section-icon {
        font-size: 16px;
    }

    .setting-item {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: var(--spacing-md);
        padding: var(--spacing-sm) 0;
        border-bottom: 1px solid var(--border-light);
    }

    .setting-item-stack {
        align-items: stretch;
        flex-direction: column;
    }

    .setting-item:last-child {
        border-bottom: none;
    }

    .setting-info {
        display: flex;
        flex-direction: column;
        gap: 2px;
        min-width: 0;
    }

    .setting-label {
        font-size: 13px;
        font-weight: 500;
        color: var(--text-primary);
    }

    .setting-label.text-danger {
        color: var(--danger);
    }

    .setting-desc {
        font-size: 11px;
        color: var(--text-tertiary);
    }

    .setting-desc strong {
        color: var(--text-secondary);
        font-weight: 600;
    }

    /* Toggle Switch */
    .toggle-switch {
        position: relative;
        width: 40px;
        height: 22px;
        background-color: var(--bg-tertiary);
        border-radius: 11px;
        border: 1px solid var(--border);
        cursor: pointer;
        transition: all var(--transition-fast);
        flex-shrink: 0;
    }

    .toggle-switch:hover:not(:disabled) {
        border-color: var(--accent);
    }

    .toggle-switch.active {
        background-color: var(--accent);
        border-color: var(--accent);
    }

    .toggle-knob {
        position: absolute;
        top: 2px;
        left: 2px;
        width: 16px;
        height: 16px;
        background-color: white;
        border-radius: 50%;
        transition: transform var(--transition-fast);
        box-shadow: var(--shadow-sm);
    }

    .toggle-switch.active .toggle-knob {
        transform: translateX(18px);
    }

    /* Segmented Control */
    .segmented-control {
        display: flex;
        background: var(--bg-tertiary);
        padding: 2px;
        border-radius: var(--radius-md);
        border: 1px solid var(--border);
    }

    .segmented-control button {
        padding: 4px 10px;
        font-size: 11px;
        border-radius: var(--radius-sm);
        color: var(--text-secondary);
        background: transparent;
        border: none;
        cursor: pointer;
        font-weight: 500;
        transition: all var(--transition-fast);
    }

    .segmented-control button:hover {
        color: var(--text-primary);
    }

    .segmented-control button.active {
        background: var(--bg-primary);
        color: var(--text-primary);
        box-shadow: var(--shadow-sm);
    }

    .setting-select {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
        min-width: 180px;
        flex-shrink: 0;
    }

    .setting-native-select {
        min-width: 140px;
        padding-right: 32px;
    }

    .afk-category-manager {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-sm);
        width: 100%;
    }

    .afk-built-in-pill {
        align-self: flex-start;
        padding: 6px 10px;
        border-radius: 999px;
        border: 1px solid color-mix(in srgb, var(--accent) 30%, var(--border));
        background: color-mix(in srgb, var(--accent) 10%, var(--bg-primary));
        color: var(--text-secondary);
        font-size: 11px;
        font-weight: 600;
    }

    .afk-category-list {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
    }

    .afk-category-pill {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        padding: 6px 10px;
        border-radius: 999px;
        background: var(--bg-primary);
        border: 1px solid var(--border);
        color: var(--text-secondary);
        font-size: 12px;
    }

    .afk-remove-btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 18px;
        height: 18px;
        border-radius: 999px;
        border: none;
        background: transparent;
        color: var(--danger);
        cursor: pointer;
        padding: 0;
        flex-shrink: 0;
        transition:
            background-color var(--transition-fast),
            color var(--transition-fast);
    }

    .afk-remove-btn:hover {
        background: color-mix(in srgb, var(--danger) 14%, transparent);
        color: var(--text-primary);
    }

    .afk-category-empty {
        color: var(--text-tertiary);
        font-size: 11px;
    }

    .afk-category-add-row {
        display: flex;
        gap: 8px;
        align-items: center;
        width: 100%;
    }

    .afk-category-input {
        flex: 1;
        min-width: 0;
    }

    .afk-category-error {
        color: var(--danger);
        font-size: 11px;
    }

    .theme-select {
        justify-content: flex-end;
    }

    .preview-dot {
        width: 10px;
        height: 10px;
        border-radius: 50%;
        background: linear-gradient(
            135deg,
            var(--preview-bg) 50%,
            var(--preview-accent) 50%
        );
        border: 1px solid var(--border);
    }

    /* Google Calendar */
    .gcal-connected {
        color: var(--success);
        font-weight: 500;
    }

    .gcal-error {
        margin-top: var(--spacing-sm);
        padding: var(--spacing-sm) var(--spacing-md);
        background: var(--danger-light);
        color: var(--danger);
        border-radius: var(--radius-sm);
        font-size: 11px;
    }

    /* Update status */
    .update-error {
        color: var(--danger);
    }
    .update-success {
        color: var(--success);
    }

    .update-progress-inline {
        width: 60px;
        flex-shrink: 0;
    }
    .progress-bar-sm {
        height: 3px;
        background: var(--bg-tertiary);
        border-radius: 2px;
        overflow: hidden;
    }
    .progress-fill-sm {
        height: 100%;
        background: var(--accent);
        transition: width 0.2s ease-out;
    }

    /* About Section */
    .settings-section.about {
        text-align: center;
        background: transparent;
        border: none;
        padding-top: 0;
    }

    .version {
        font-size: 11px;
        color: var(--text-tertiary);
    }

    .window-track-confirm-text {
        margin-bottom: 14px;
        color: var(--text-secondary);
        line-height: 1.5;
        font-size: 13px;
    }

    .window-track-confirm-list {
        margin-bottom: 20px;
        padding: 12px 14px;
        border: 1px solid var(--border);
        border-radius: var(--radius-md);
        background: var(--bg-secondary);
        color: var(--text-secondary);
        font-size: 12px;
        line-height: 1.6;
    }

    .window-track-confirm-list p {
        margin-bottom: 6px;
        color: var(--text-primary);
        font-weight: 600;
    }

    .window-track-confirm-list ul {
        margin: 0;
        padding-left: 18px;
    }
</style>
