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

            let contentLength = 0;
            let totalDownloaded = 0;

            await update.downloadAndInstall((event) => {
                if (event.event === "Started") {
                    contentLength = event.data.contentLength ?? 0;
                    totalDownloaded = 0;
                    downloadProgress = 0;
                } else if (event.event === "Progress") {
                    totalDownloaded += event.data.chunkLength;
                    if (contentLength > 0) {
                        downloadProgress = Math.min(
                            Math.round((totalDownloaded / contentLength) * 100),
                            99,
                        );
                    } else {
                        downloadProgress = Math.min(downloadProgress + 3, 95);
                    }
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
            <div class="update-content" role="status" aria-live="polite">
                <span class="update-icon">⬇️</span>
                <span class="update-text"
                    >Downloading update... {downloadProgress}%</span
                >
            </div>
            <div
                class="progress-bar"
                role="progressbar"
                aria-label="Update download progress"
                aria-valuemin="0"
                aria-valuemax="100"
                aria-valuenow={downloadProgress}
            >
                <div
                    class="progress-fill"
                    style="width: {downloadProgress}%"
                ></div>
            </div>
        {:else if error}
            <div class="update-content" role="alert">
                <span class="update-icon">⚠️</span>
                <span class="update-text error">{error}</span>
                <button
                    type="button"
                    class="update-btn retry"
                    onclick={downloadAndInstall}
                    >Retry</button
                >
                <button
                    type="button"
                    class="update-btn dismiss"
                    aria-label="Dismiss update notification"
                    onclick={dismiss}
                >
                    ×
                </button>
            </div>
        {:else}
            <div class="update-content" role="status" aria-live="polite">
                <span class="update-icon">✨</span>
                <span class="update-text">
                    Update available: <strong>v{updateAvailable.version}</strong
                    >
                </span>
                <button
                    type="button"
                    class="update-btn primary"
                    onclick={downloadAndInstall}
                    >Update</button
                >
                <button
                    type="button"
                    class="update-btn dismiss"
                    aria-label="Dismiss update notification"
                    onclick={dismiss}
                >
                    ×
                </button>
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
        background: var(--bg-secondary);
        border-top: 1px solid var(--border);
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
        color: var(--text-primary);
    }

    .update-text.error {
        color: var(--danger);
    }

    .update-text strong {
        color: var(--accent);
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
        background: var(--accent);
        color: var(--accent-contrast);
    }

    .update-btn.primary:hover {
        background: var(--accent-hover);
    }

    .update-btn.retry {
        background: var(--bg-tertiary);
        color: var(--text-primary);
        border: 1px solid var(--border);
    }

    .update-btn.dismiss {
        background: transparent;
        color: var(--text-secondary);
        padding: 4px 6px;
    }

    .update-btn.dismiss:hover {
        color: var(--text-primary);
    }

    .progress-bar {
        height: 3px;
        background: var(--bg-tertiary);
        border-radius: 2px;
        margin-top: 6px;
        overflow: hidden;
    }

    .progress-fill {
        height: 100%;
        background: var(--accent);
        transition: width 0.2s ease-out;
    }
</style>
