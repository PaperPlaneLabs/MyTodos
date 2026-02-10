<script lang="ts">
    import { onMount } from "svelte";
    import { fade, fly } from "svelte/transition";
    import { uiStore, type Theme } from "$lib/stores/ui.svelte";
    import { googleCalendarStore } from "$lib/stores/google-calendar.svelte";
    import { getVersion } from "@tauri-apps/api/app";
    import { check } from "@tauri-apps/plugin-updater";
    import { relaunch } from "@tauri-apps/plugin-process";

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

    let isAutoStartEnabled = $state(false);
    let loading = $state(true);
    let toggling = $state(false);

    let appVersion = $state("");
    let updateStatus = $state<"idle" | "checking" | "up-to-date" | "available" | "downloading" | "error">("idle");
    let updateVersion = $state("");
    let updateError = $state("");
    let updateProgress = $state(0);

    onMount(async () => {
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
</script>

<div class="settings-view" transition:fade={{ duration: 200 }}>
    <header class="settings-header">
        <button class="back-btn" onclick={() => uiStore.closeSettingsView()}>
            <svg
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
            >
                <path d="m15 18-6-6 6-6" />
            </svg>
            <span>Back</span>
        </button>
        <h2>Settings</h2>
    </header>

    <div class="settings-content">
        <section
            class="settings-section"
            transition:fly={{ y: 20, duration: 300, delay: 100 }}
        >
            <h3>
                <span class="section-icon">🚀</span>
                Startup
            </h3>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">Start at login</span>
                    <span class="setting-desc"
                        >Automatically launch MyTodos when you log in</span
                    >
                </div>
                <button
                    class="toggle-switch"
                    class:active={isAutoStartEnabled}
                    class:loading={loading || toggling}
                    onclick={toggleAutoStart}
                    disabled={loading || toggling}
                    title={isAutoStartEnabled
                        ? "Disable autostart"
                        : "Enable autostart"}
                >
                    <span class="toggle-knob"></span>
                </button>
            </div>
        </section>

        <section
            class="settings-section"
            transition:fly={{ y: 20, duration: 300, delay: 150 }}
        >
            <h3>
                <span class="section-icon">🔄</span>
                Updates
            </h3>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">App Updates</span>
                    <span class="setting-desc" class:update-error={updateStatus === "error"} class:update-success={updateStatus === "up-to-date"}>
                        {#if updateStatus === "checking"}
                            Checking for updates...
                        {:else if updateStatus === "up-to-date"}
                            You're up to date!
                        {:else if updateStatus === "available"}
                            Update available: <strong>v{updateVersion}</strong>
                        {:else if updateStatus === "downloading"}
                            Downloading update... {updateProgress}%
                        {:else if updateStatus === "error"}
                            {updateError}
                        {:else}
                            Check if a newer version is available
                        {/if}
                    </span>
                </div>
                {#if updateStatus === "available"}
                    <button
                        class="btn btn-primary btn-sm"
                        onclick={downloadAndInstallUpdate}
                    >
                        Update Now
                    </button>
                {:else if updateStatus === "downloading"}
                    <div class="update-progress-inline">
                        <div class="progress-bar-sm">
                            <div class="progress-fill-sm" style="width: {updateProgress}%"></div>
                        </div>
                    </div>
                {:else}
                    <button
                        class="btn btn-secondary btn-sm"
                        onclick={checkForUpdates}
                        disabled={updateStatus === "checking"}
                    >
                        {#if updateStatus === "checking"}
                            Checking...
                        {:else if updateStatus === "error"}
                            Retry
                        {:else}
                            Check for Updates
                        {/if}
                    </button>
                {/if}
            </div>
        </section>

        <section
            class="settings-section"
            transition:fly={{ y: 20, duration: 300, delay: 200 }}
        >
            <h3>
                <span class="section-icon">📅</span>
                Google Calendar
            </h3>

            {#if !googleCalendarStore.connected}
                <div class="setting-item">
                    <div class="setting-info">
                        <span class="setting-label">Connect Google Calendar</span>
                        <span class="setting-desc">Sync task deadlines as all-day events</span>
                    </div>
                    <button
                        class="btn btn-primary btn-sm"
                        onclick={() => googleCalendarStore.connect()}
                        disabled={googleCalendarStore.connecting}
                    >
                        {googleCalendarStore.connecting ? "Connecting..." : "Connect"}
                    </button>
                </div>
            {:else}
                <div class="setting-item">
                    <div class="setting-info">
                        <span class="setting-label gcal-connected">Connected</span>
                        <span class="setting-desc">Tasks with deadlines sync to your Google Calendar</span>
                    </div>
                    <button
                        class="btn btn-secondary btn-sm"
                        onclick={() => googleCalendarStore.disconnect()}
                    >
                        Disconnect
                    </button>
                </div>
                <div class="setting-item">
                    <div class="setting-info">
                        <span class="setting-label">Manual Sync</span>
                        <span class="setting-desc">
                            {#if googleCalendarStore.lastSyncResult}
                                Last sync: {googleCalendarStore.lastSyncResult.synced} synced{#if googleCalendarStore.lastSyncResult.failed > 0}, {googleCalendarStore.lastSyncResult.failed} failed{/if}
                            {:else}
                                Sync all tasks with deadlines
                            {/if}
                        </span>
                    </div>
                    <button
                        class="btn btn-secondary btn-sm"
                        onclick={() => googleCalendarStore.syncAll()}
                        disabled={googleCalendarStore.syncing}
                    >
                        {googleCalendarStore.syncing ? "Syncing..." : "Sync Now"}
                    </button>
                </div>
            {/if}

            {#if googleCalendarStore.error}
                <div class="gcal-error">
                    {googleCalendarStore.error}
                </div>
            {/if}
        </section>

        <section
            class="settings-section"
            transition:fly={{ y: 20, duration: 300, delay: 250 }}
        >
            <h3>
                <span class="section-icon">🎨</span>
                Appearance
            </h3>

            <div class="themes-grid">
                {#each themes as t}
                    <button
                        class="theme-card"
                        class:active={uiStore.theme === t.id}
                        onclick={() => uiStore.setTheme(t.id)}
                    >
                        <div
                            class="theme-preview"
                            style="--preview-bg: {t.bg}; --preview-accent: {t.accent}"
                        >
                            <div class="preview-dot"></div>
                        </div>
                        <span class="theme-name">{t.name}</span>
                    </button>
                {/each}
            </div>
        </section>

        <section
            class="settings-section about"
            transition:fly={{ y: 20, duration: 300, delay: 350 }}
        >
            <h3>
                <span class="section-icon">ℹ️</span>
                About
            </h3>
            <p class="version">MyTodos v{appVersion}</p>
        </section>
    </div>
</div>

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

    .settings-section h3 {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
        font-size: 14px;
        font-weight: 600;
        color: var(--text-primary);
        margin-bottom: var(--spacing-md);
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

    .setting-desc {
        font-size: 11px;
        color: var(--text-tertiary);
    }

    /* Toggle Switch */
    .toggle-switch {
        position: relative;
        width: 44px;
        height: 24px;
        background-color: var(--bg-tertiary);
        border-radius: 12px;
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

    .toggle-switch.loading {
        opacity: 0.6;
        cursor: wait;
    }

    .toggle-knob {
        position: absolute;
        top: 2px;
        left: 2px;
        width: 18px;
        height: 18px;
        background-color: white;
        border-radius: 50%;
        transition: transform var(--transition-fast);
        box-shadow: var(--shadow-sm);
    }

    .toggle-switch.active .toggle-knob {
        transform: translateX(20px);
    }

    .themes-grid {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: var(--spacing-md);
        margin-top: var(--spacing-sm);
    }

    .theme-card {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: var(--spacing-sm);
        padding: var(--spacing-sm);
        background: var(--bg-primary);
        border: 2px solid transparent;
        border-radius: var(--radius-md);
        cursor: pointer;
        transition: all var(--transition-fast);
    }

    .theme-card:hover {
        background: var(--bg-hover);
        border-color: var(--border);
    }

    .theme-card.active {
        border-color: var(--accent);
        background: var(--accent-light);
    }

    .theme-preview {
        width: 100%;
        aspect-ratio: 16/9;
        background-color: var(--preview-bg);
        border-radius: var(--radius-sm);
        border: 1px solid var(--border);
        display: flex;
        align-items: center;
        justify-content: center;
        position: relative;
    }

    .preview-dot {
        width: 12px;
        height: 12px;
        background-color: var(--preview-accent);
        border-radius: 50%;
        box-shadow: 0 0 8px var(--preview-accent);
    }

    .theme-name {
        font-size: 11px;
        font-weight: 500;
        color: var(--text-primary);
    }

    /* Google Calendar */
    .gcal-connected {
        color: var(--success);
    }

    .gcal-error {
        margin-top: var(--spacing-sm);
        padding: var(--spacing-sm) var(--spacing-md);
        background: var(--danger-light);
        color: var(--danger);
        border-radius: var(--radius-sm);
        font-size: 12px;
    }

    /* Update status */
    .update-error {
        color: var(--danger);
    }

    .update-success {
        color: var(--success);
    }

    /* Update progress */
    .update-progress-inline {
        width: 80px;
        flex-shrink: 0;
    }

    .progress-bar-sm {
        height: 4px;
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
    }

    .version {
        font-size: 12px;
        color: var(--text-tertiary);
    }
</style>
