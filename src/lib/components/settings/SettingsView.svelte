<script lang="ts">
    import { onMount } from "svelte";
    import { fade, fly } from "svelte/transition";
    import { uiStore } from "$lib/stores/ui.svelte";

    let isAutoStartEnabled = $state(false);
    let loading = $state(true);
    let toggling = $state(false);

    onMount(async () => {
        try {
            const { isEnabled } = await import("@tauri-apps/plugin-autostart");
            isAutoStartEnabled = await isEnabled();
        } catch (e) {
            console.error("Failed to check autostart status:", e);
        } finally {
            loading = false;
        }
    });

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
            transition:fly={{ y: 20, duration: 300, delay: 200 }}
        >
            <h3>
                <span class="section-icon">🎨</span>
                Appearance
            </h3>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">Theme</span>
                    <span class="setting-desc"
                        >Switch between light and dark mode</span
                    >
                </div>
                <button
                    class="theme-toggle"
                    onclick={() => uiStore.toggleTheme()}
                >
                    {uiStore.theme === "dark" ? "🌙 Dark" : "☀️ Light"}
                </button>
            </div>
        </section>

        <section
            class="settings-section about"
            transition:fly={{ y: 20, duration: 300, delay: 300 }}
        >
            <h3>
                <span class="section-icon">ℹ️</span>
                About
            </h3>
            <p class="version">MyTodos v0.1.21</p>
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

    /* Theme Toggle Button */
    .theme-toggle {
        padding: 6px 12px;
        background-color: var(--bg-primary);
        border: 1px solid var(--border);
        border-radius: var(--radius-md);
        font-size: 12px;
        font-weight: 500;
        color: var(--text-primary);
        transition: all var(--transition-fast);
        white-space: nowrap;
    }

    .theme-toggle:hover {
        background-color: var(--bg-hover);
        border-color: var(--accent);
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
