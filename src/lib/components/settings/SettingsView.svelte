<script lang="ts">
    import { onMount } from "svelte";
    import { fade, fly, slide } from "svelte/transition";
    import { uiStore, type Theme, type WindowOrientation } from "$lib/stores/ui.svelte";
    import { googleCalendarStore } from "$lib/stores/google-calendar.svelte";
    import { projectStore } from "$lib/stores/projects.svelte";
    import { taskStore } from "$lib/stores/tasks.svelte";
    import { timerStore } from "$lib/stores/timer.svelte";
    import { db } from "$lib/services/db";
    import { getVersion } from "@tauri-apps/api/app";
    import { check } from "@tauri-apps/plugin-updater";
    import { relaunch } from "@tauri-apps/plugin-process";
    import Modal from "$lib/components/common/Modal.svelte";

    const themes: { id: Theme; name: string; bg: string; accent: string }[] = [
        { id: "light", name: "Light", bg: "#ffffff", accent: "#6366f1" },
        { id: "dark", name: "Dark", bg: "#1a1a1a", accent: "#818cf8" },
        { id: "minecraft", name: "Minecraft", bg: "#3d2b1f", accent: "#3c8527" },
        { id: "retro", name: "Retro", bg: "#0c0c0c", accent: "#ffb000" },
        { id: "ocean", name: "Ocean", bg: "#001219", accent: "#0a9396" },
        { id: "nord", name: "Nord", bg: "#2e3440", accent: "#88c0d0" },
    ];
    const breakReminderIntervals = Array.from({ length: 11 }, (_, index) => 10 + index * 5);

    let isAutoStartEnabled = $state(false);
    let loading = $state(true);
    let toggling = $state(false);
    let themeDropdownOpen = $state(false);
    let showResetConfirm = $state(false);
    let showBreakIntervalConfirm = $state(false);
    let pendingBreakInterval = $state<number | null>(null);
    let breakIntervalSelectValue = $state("30");

    let appVersion = $state("");
    let updateStatus = $state<"idle" | "checking" | "up-to-date" | "available" | "downloading" | "error">("idle");
    let updateVersion = $state("");
    let updateError = $state("");
    let updateProgress = $state(0);

    onMount(async () => {
        breakIntervalSelectValue = String(timerStore.breakReminderIntervalMinutes);

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
            await timerStore.stop();
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

    function toggleThemeDropdown() {
        themeDropdownOpen = !themeDropdownOpen;
    }

    function selectTheme(id: Theme) {
        uiStore.setTheme(id);
        themeDropdownOpen = false;
    }

    function handleWindowOrientation(orientation: WindowOrientation) {
        uiStore.setWindowOrientation(orientation);
        if (orientation === "left") {
            db.window.dock("left");
        } else if (orientation === "right") {
            db.window.dock("right");
        } else {
            db.window.center();
        }
    }

    function updateBreakReminderInterval(event: Event) {
        const target = event.currentTarget as HTMLSelectElement;
        const nextInterval = Number(target.value);
        if (!Number.isFinite(nextInterval)) {
            breakIntervalSelectValue = String(timerStore.breakReminderIntervalMinutes);
            return;
        }

        if (nextInterval === timerStore.breakReminderIntervalMinutes) {
            breakIntervalSelectValue = String(timerStore.breakReminderIntervalMinutes);
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
        breakIntervalSelectValue = String(timerStore.breakReminderIntervalMinutes);
    }
</script>

<div class="settings-view" transition:fade={{ duration: 200 }}>
    <header class="settings-header">
        <button class="back-btn" onclick={() => uiStore.closeSettingsView()}>
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="m15 18-6-6 6-6" />
            </svg>
            <span>Back</span>
        </button>
        <h2>Settings</h2>
    </header>

    <div class="settings-content">
        <!-- GENERAL SECTION -->
        <section class="settings-section" transition:fly={{ y: 20, duration: 300, delay: 100 }}>
            <h3><span class="section-icon">⚙️</span> General</h3>
            
            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">Start at login</span>
                    <span class="setting-desc">Automatically launch MyTodos when you log in</span>
                </div>
                <button
                    class="toggle-switch"
                    class:active={isAutoStartEnabled}
                    class:loading={loading || toggling}
                    onclick={toggleAutoStart}
                    disabled={loading || toggling}
                    title={isAutoStartEnabled ? "Disable autostart" : "Enable autostart"}
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
                        class:active={uiStore.windowOrientation === 'left'} 
                        onclick={() => handleWindowOrientation('left')}
                    >Left</button>
                    <button 
                        class:active={uiStore.windowOrientation === 'center'} 
                        onclick={() => handleWindowOrientation('center')}
                    >FreeForm</button>
                    <button 
                        class:active={uiStore.windowOrientation === 'right'} 
                        onclick={() => handleWindowOrientation('right')}
                    >Right</button>
                </div>
            </div>

             <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">Compact Mode</span>
                    <span class="setting-desc">Reduce spacing for a denser layout</span>
                </div>
                <button
                    class="toggle-switch"
                    class:active={uiStore.compactMode}
                    onclick={() => uiStore.setCompactMode(!uiStore.compactMode)}
                    title={uiStore.compactMode ? "Disable compact mode" : "Enable compact mode"}
                >
                    <span class="toggle-knob"></span>
                </button>
            </div>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">Break Reminders</span>
                    <span class="setting-desc">Show a break prompt while a timer is running</span>
                </div>
                <button
                    class="toggle-switch"
                    class:active={timerStore.breakReminderEnabled}
                    onclick={() => timerStore.setBreakReminderEnabled(!timerStore.breakReminderEnabled)}
                    title={timerStore.breakReminderEnabled ? "Disable break reminders" : "Enable break reminders"}
                >
                    <span class="toggle-knob"></span>
                </button>
            </div>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">Break Interval</span>
                    <span class="setting-desc">
                        {#if timerStore.breakReminderEnabled}
                            Remind every {timerStore.breakReminderIntervalMinutes} minutes
                        {:else}
                            Reminders are currently disabled
                        {/if}
                    </span>
                </div>
                <select
                    class="input break-interval-select"
                    bind:value={breakIntervalSelectValue}
                    onchange={updateBreakReminderInterval}
                    disabled={!timerStore.breakReminderEnabled}
                >
                    {#each breakReminderIntervals as interval}
                        <option value={interval}>{interval} minutes</option>
                    {/each}
                </select>
            </div>
             
              <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">App Updates</span>
                    <span class="setting-desc" class:update-error={updateStatus === "error"} class:update-success={updateStatus === "up-to-date"}>
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
                    <button class="btn btn-primary btn-sm" onclick={downloadAndInstallUpdate}>Update</button>
                {:else if updateStatus === "downloading"}
                     <div class="update-progress-inline">
                        <div class="progress-bar-sm">
                            <div class="progress-fill-sm" style="width: {updateProgress}%"></div>
                        </div>
                    </div>
                {:else}
                    <button class="btn btn-secondary btn-sm" onclick={checkForUpdates} disabled={updateStatus === "checking"}>
                        Check
                    </button>
                {/if}
            </div>
        </section>

        <!-- APPEARANCE SECTION -->
        <section class="settings-section" transition:fly={{ y: 20, duration: 300, delay: 150 }}>
            <h3><span class="section-icon">🎨</span> Appearance</h3>
            
            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">Theme</span>
                    <span class="setting-desc">Customize application look</span>
                </div>
                
                <div class="theme-dropdown-container">
                    <button class="theme-dropdown-trigger" onclick={toggleThemeDropdown}>
                        {#if uiStore.theme}
                            {@const current = themes.find(t => t.id === uiStore.theme)}
                            <span class="theme-name-current">{current?.name || 'Select Theme'}</span>
                            <div class="preview-dot" style="--preview-bg: {current?.bg}; --preview-accent: {current?.accent}"></div>
                        {/if}
                        <span class="chevron">▼</span>
                    </button>

                    {#if themeDropdownOpen}
                        <div class="theme-dropdown-menu" transition:slide={{ duration: 150 }}>
                            {#each themes as t}
                                <button class="theme-option" class:selected={uiStore.theme === t.id} onclick={() => selectTheme(t.id)}>
                                    <span class="theme-name">{t.name}</span>
                                    <div class="preview-dot" style="--preview-bg: {t.bg}; --preview-accent: {t.accent}"></div>
                                </button>
                            {/each}
                        </div>
                    {/if}
                </div>
            </div>
            
            {#if themeDropdownOpen}
                <div class="backdrop" onclick={() => themeDropdownOpen = false}></div>
            {/if}
        </section>

        <!-- INTEGRATIONS SECTION -->
        <section class="settings-section" transition:fly={{ y: 20, duration: 300, delay: 200 }}>
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
                    <button class="btn btn-primary btn-sm" onclick={() => googleCalendarStore.connect()} disabled={googleCalendarStore.connecting}>
                         {googleCalendarStore.connecting ? "..." : "Connect"}
                    </button>
                 {:else}
                    <div style="display:flex; gap: 8px;">
                         <button class="btn btn-secondary btn-sm" onclick={() => googleCalendarStore.syncAll()} disabled={googleCalendarStore.syncing}>
                            {googleCalendarStore.syncing ? "..." : "Sync"}
                        </button>
                        <button class="btn btn-secondary btn-sm" onclick={() => googleCalendarStore.disconnect()}>
                            Disconnect
                        </button>
                    </div>
                 {/if}
            </div>
             {#if googleCalendarStore.error}
                <div class="gcal-error">{googleCalendarStore.error}</div>
            {/if}
        </section>

        <!-- DATA MANAGEMENT SECTION -->
        <section class="settings-section danger-zone" transition:fly={{ y: 20, duration: 300, delay: 250 }}>
            <h3><span class="section-icon">⚠️</span> Data Management</h3>
            
            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label text-danger">Reset Application</span>
                    <span class="setting-desc">Permanently delete all projects and tasks</span>
                </div>
                <button class="btn btn-danger btn-sm" onclick={() => showResetConfirm = true}>
                    Reset Data
                </button>
            </div>
        </section>

        <!-- ABOUT SECTION -->
        <section class="settings-section about" transition:fly={{ y: 20, duration: 300, delay: 300 }}>
             <p class="version">MyTodos v{appVersion}</p>
        </section>
    </div>
</div>

<Modal open={showResetConfirm} title="⚠️ Irreversible Action" onClose={() => showResetConfirm = false}>
    {#snippet children()}
        <div class="reset-confirm-content">
            <p style="margin-bottom: 20px; color: var(--text-secondary); line-height: 1.5;">
                Are you absolutely sure you want to delete all data? This action cannot be undone.
            </p>
            <div style="display: flex; gap: 10px; justify-content: flex-end;">
                <button class="btn btn-secondary" onclick={() => showResetConfirm = false}>Cancel</button>
                <button class="btn btn-danger" onclick={handleClearData}>Yes, Delete Everything</button>
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
            <p style="margin-bottom: 20px; color: var(--text-secondary); line-height: 1.5;">
                Set break reminders to every {pendingBreakInterval ?? timerStore.breakReminderIntervalMinutes} minutes?
            </p>
            <div style="display: flex; gap: 10px; justify-content: flex-end;">
                <button class="btn btn-secondary" onclick={cancelBreakIntervalChange}>No</button>
                <button class="btn btn-primary" onclick={confirmBreakIntervalChange}>Yes</button>
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
        background-color: color-mix(in srgb, var(--danger-light) 10%, var(--bg-secondary));
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

    .break-interval-select {
        width: 140px;
        font-size: 12px;
        padding: 6px 10px;
        color: var(--text-primary);
        background-color: var(--bg-primary);
        border: 1px solid var(--border);
        color-scheme: light;
    }

    .break-interval-select option {
        color: var(--text-primary);
        background-color: var(--bg-primary);
    }

    :global([data-theme="dark"]) .break-interval-select,
    :global([data-theme="retro"]) .break-interval-select,
    :global([data-theme="ocean"]) .break-interval-select,
    :global([data-theme="nord"]) .break-interval-select,
    :global([data-theme="minecraft"]) .break-interval-select {
        color-scheme: dark;
    }

    .break-interval-select:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }

    /* Theme Dropdown */
    .theme-dropdown-container {
        position: relative;
    }

    .theme-dropdown-trigger {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 6px 12px;
        background: var(--bg-primary);
        border: 1px solid var(--border);
        border-radius: var(--radius-md);
        cursor: pointer;
        min-width: 140px;
        justify-content: space-between;
        color: var(--text-primary);
        font-size: 13px;
        transition: all var(--transition-fast);
    }

    .theme-dropdown-trigger:hover {
        border-color: var(--accent);
    }

    .theme-name-current {
        font-weight: 500;
    }

    .chevron {
        font-size: 10px;
        opacity: 0.5;
    }

    .theme-dropdown-menu {
        position: absolute;
        top: 100%;
        right: 0;
        margin-top: 4px;
        background: var(--bg-primary);
        border: 1px solid var(--border);
        border-radius: var(--radius-md);
        box-shadow: var(--shadow-lg);
        width: 160px;
        z-index: 1000;
        padding: 4px;
        overflow: hidden;
    }

    .theme-option {
        display: flex;
        align-items: center;
        justify-content: space-between;
        width: 100%;
        padding: 6px 10px;
        border: none;
        background: transparent;
        cursor: pointer;
        border-radius: var(--radius-sm);
        color: var(--text-primary);
        font-size: 12px;
        transition: all var(--transition-fast);
    }

    .theme-option:hover {
        background: var(--bg-hover);
    }

    .theme-option.selected {
        background: var(--accent-light);
        color: var(--accent);
    }

    .preview-dot {
        width: 10px;
        height: 10px;
        border-radius: 50%;
        background: linear-gradient(135deg, var(--preview-bg) 50%, var(--preview-accent) 50%);
        border: 1px solid var(--border);
    }

    .backdrop {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        z-index: 900;
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
    .update-error { color: var(--danger); }
    .update-success { color: var(--success); }

    .update-progress-inline { width: 60px; flex-shrink: 0; }
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
</style>
