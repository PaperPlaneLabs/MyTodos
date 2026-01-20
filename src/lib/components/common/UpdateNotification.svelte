<script lang="ts">
    import { onMount } from "svelte";
    import { check } from "@tauri-apps/plugin-updater";
    import { relaunch } from "@tauri-apps/plugin-process";

    interface UpdateInfo {
        version: string;
        currentVersion: string;
        body?: string;
    }

    let updateAvailable = $state<UpdateInfo | null>(null);
    let isDownloading = $state(false);
    let downloadProgress = $state(0);
    let error = $state<string | null>(null);
    let dismissed = $state(false);

    onMount(async () => {
        try {
            const update = await check();
            if (update) {
                updateAvailable = {
                    version: update.version,
                    currentVersion: update.currentVersion,
                    body: update.body,
                };
            }
        } catch (err) {
            console.error("Failed to check for updates:", err);
        }
    });

    async function downloadAndInstall() {
        if (!updateAvailable) return;

        try {
            isDownloading = true;
            error = null;

            const update = await check();
            if (!update) return;

            await update.downloadAndInstall((event) => {
                if (event.event === "Started") {
                    downloadProgress = 0;
                } else if (event.event === "Progress") {
                    // Increment progress gradually since we don't have total size
                    downloadProgress = Math.min(downloadProgress + 3, 95);
                } else if (event.event === "Finished") {
                    downloadProgress = 100;
                }
            });

            await relaunch();
        } catch (err) {
            error = err instanceof Error ? err.message : "Update failed";
            isDownloading = false;
        }
    }

    function dismiss() {
        dismissed = true;
    }
</script>

{#if updateAvailable && !dismissed}
    <div class="update-notification">
        {#if isDownloading}
            <div class="update-content">
                <span class="update-icon">⬇️</span>
                <span class="update-text"
                    >Downloading update... {downloadProgress}%</span
                >
            </div>
            <div class="progress-bar">
                <div
                    class="progress-fill"
                    style="width: {downloadProgress}%"
                ></div>
            </div>
        {:else if error}
            <div class="update-content">
                <span class="update-icon">⚠️</span>
                <span class="update-text error">{error}</span>
                <button class="update-btn retry" onclick={downloadAndInstall}
                    >Retry</button
                >
                <button class="update-btn dismiss" onclick={dismiss}>×</button>
            </div>
        {:else}
            <div class="update-content">
                <span class="update-icon">✨</span>
                <span class="update-text">
                    Update available: <strong>v{updateAvailable.version}</strong
                    >
                </span>
                <button class="update-btn primary" onclick={downloadAndInstall}
                    >Update</button
                >
                <button class="update-btn dismiss" onclick={dismiss}>×</button>
            </div>
        {/if}
    </div>
{/if}

<style>
    .update-notification {
        position: fixed;
        bottom: 0;
        left: 0;
        right: 0;
        background: var(--color-surface-elevated, #1a1a2e);
        border-top: 1px solid var(--color-border, #333);
        padding: 8px 12px;
        z-index: 9999;
        animation: slideUp 0.3s ease-out;
    }

    @keyframes slideUp {
        from {
            transform: translateY(100%);
            opacity: 0;
        }
        to {
            transform: translateY(0);
            opacity: 1;
        }
    }

    .update-content {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 13px;
    }

    .update-icon {
        font-size: 14px;
    }

    .update-text {
        flex: 1;
        color: var(--color-text, #e0e0e0);
    }

    .update-text.error {
        color: var(--color-danger, #ff6b6b);
    }

    .update-text strong {
        color: var(--color-primary, #7c3aed);
    }

    .update-btn {
        padding: 4px 10px;
        border-radius: 4px;
        border: none;
        font-size: 12px;
        cursor: pointer;
        transition: all 0.2s;
    }

    .update-btn.primary {
        background: var(--color-primary, #7c3aed);
        color: white;
    }

    .update-btn.primary:hover {
        background: var(--color-primary-hover, #6d28d9);
    }

    .update-btn.retry {
        background: var(--color-surface, #2a2a40);
        color: var(--color-text, #e0e0e0);
        border: 1px solid var(--color-border, #333);
    }

    .update-btn.dismiss {
        background: transparent;
        color: var(--color-text-secondary, #888);
        padding: 4px 6px;
    }

    .update-btn.dismiss:hover {
        color: var(--color-text, #e0e0e0);
    }

    .progress-bar {
        height: 3px;
        background: var(--color-surface, #2a2a40);
        border-radius: 2px;
        margin-top: 6px;
        overflow: hidden;
    }

    .progress-fill {
        height: 100%;
        background: var(--color-primary, #7c3aed);
        transition: width 0.2s ease-out;
    }
</style>
