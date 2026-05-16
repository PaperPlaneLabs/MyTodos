<script lang="ts">
    import { onMount } from "svelte";
    import { fade, fly } from "svelte/transition";
    import { uiStore } from "$lib/stores/ui.svelte";
    import { timerStore } from "$lib/stores/timer.svelte";
    import { windowTrackingStore } from "$lib/stores/window-tracking.svelte";
    import { db, type TimeStats, type WindowActivityStats } from "$lib/services/db";

    let stats = $state<TimeStats | null>(null);
    let windowStats = $state<WindowActivityStats | null>(null);
    let loading = $state(true);
    let error = $state<string | null>(null);
    let refreshIntervalId: number | null = null;

    async function loadStats() {
        try {
            if (windowTrackingStore.enabled) {
                await windowTrackingStore.refresh();
                windowStats = await db.windowTracking.getStats();
                stats = null;
            } else {
                stats = await db.timeEntries.getTimeStats(true); // include_active_timer
                windowStats = null;
            }
        } catch (e) {
            console.error("Failed to load stats:", e);
            error = "Failed to load statistics";
        } finally {
            loading = false;
        }
    }

    // Load initially
    onMount(() => {
        loadStats();
    });

    // Refresh when timer state changes
    $effect(() => {
        // Watch timer change signal
        timerStore.changeSignal;
        windowTrackingStore.changeSignal;

        // Reload stats (but not on initial mount)
        if (!loading) {
            loadStats();
        }
    });

    // Periodic refresh while timer is running
    $effect(() => {
        if (timerStore.isRunning || windowTrackingStore.enabled) {
            refreshIntervalId = window.setInterval(() => {
                loadStats();
            }, 5000); // Refresh every 5 seconds
        } else {
            if (refreshIntervalId !== null) {
                clearInterval(refreshIntervalId);
                refreshIntervalId = null;
            }
        }

        return () => {
            if (refreshIntervalId !== null) {
                clearInterval(refreshIntervalId);
            }
        };
    });

    // Group apps under 10 min into "Others"
    const OTHERS_THRESHOLD_SECONDS = 600;

    function groupSmallApps(apps: WindowActivityStats["today_apps"]) {
        const big = apps.filter((a) => a.total_seconds >= OTHERS_THRESHOLD_SECONDS);
        const small = apps.filter((a) => a.total_seconds < OTHERS_THRESHOLD_SECONDS);
        if (small.length === 0) return big;
        const othersSeconds = small.reduce((sum, a) => sum + a.total_seconds, 0);
        return [
            ...big,
            {
                app_identifier: "others",
                app_name: "Others",
                total_seconds: othersSeconds,
                color: "#6b7280",
                kind: "app" as const,
            },
        ];
    }

    let groupedTodayApps = $derived(
        windowStats ? groupSmallApps(windowStats.today_apps) : [],
    );
    let groupedApps = $derived(
        windowStats ? groupSmallApps(windowStats.apps) : [],
    );

    // Derived values for charts
    let maxTodaySeconds = $derived(
        windowTrackingStore.enabled
            ? groupedTodayApps.reduce(
                  (max, app) => Math.max(max, app.total_seconds),
                  0,
              ) || 0
            : stats?.today_tasks.reduce(
                  (max, t) => Math.max(max, t.total_seconds),
                  0,
              ) || 0,
    );

    let totalProjectSeconds = $derived(
        windowTrackingStore.enabled
            ? windowStats?.apps.reduce((sum, app) => sum + app.total_seconds, 0) || 0
            : stats?.projects.reduce((sum, p) => sum + p.total_seconds, 0) || 0,
    );

    // Week days for the bar chart (Mon-Sun)
    const weekDays = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

    let weekData = $derived.by(() => {
        const dailyData = windowTrackingStore.enabled
            ? windowStats?.week_daily
            : stats?.week_daily;
        if (!dailyData) return weekDays.map((d) => ({ day: d, seconds: 0 }));

        const today = new Date();
        const dayOfWeek = today.getDay(); // 0 = Sunday
        const mondayOffset = dayOfWeek === 0 ? 6 : dayOfWeek - 1;
        const monday = new Date(today);
        monday.setDate(today.getDate() - mondayOffset);
        monday.setHours(0, 0, 0, 0);

        return weekDays.map((day, i) => {
            const date = new Date(monday);
            date.setDate(monday.getDate() + i);
            const dateStr = date.toISOString().split("T")[0];
            const entry = dailyData.find((d) => d.date === dateStr);
            return { day, seconds: entry?.total_seconds || 0 };
        });
    });

    let maxWeekSeconds = $derived(
        weekData.reduce((max, d) => Math.max(max, d.seconds), 0) || 1,
    );

    // Pie chart calculations
    function getPieSlices(
        projects: { total_seconds: number; color: string }[],
    ) {
        if (projects.length === 0) return [];

        const total = projects.reduce((sum, p) => sum + p.total_seconds, 0);
        let currentAngle = 0;

        return projects.map((p) => {
            const percentage = p.total_seconds / total;
            const startAngle = currentAngle;
            const endAngle = currentAngle + percentage * 360;
            currentAngle = endAngle;

            return {
                ...p,
                percentage,
                startAngle,
                endAngle,
            };
        });
    }

    function describeArc(
        x: number,
        y: number,
        radius: number,
        startAngle: number,
        endAngle: number,
    ): string {
        const start = polarToCartesian(x, y, radius, endAngle);
        const end = polarToCartesian(x, y, radius, startAngle);
        const largeArcFlag = endAngle - startAngle <= 180 ? "0" : "1";

        return [
            "M",
            x,
            y,
            "L",
            start.x,
            start.y,
            "A",
            radius,
            radius,
            0,
            largeArcFlag,
            0,
            end.x,
            end.y,
            "Z",
        ].join(" ");
    }

    function polarToCartesian(
        cx: number,
        cy: number,
        radius: number,
        angleInDegrees: number,
    ) {
        const angleInRadians = ((angleInDegrees - 90) * Math.PI) / 180.0;
        return {
            x: cx + radius * Math.cos(angleInRadians),
            y: cy + radius * Math.sin(angleInRadians),
        };
    }

    function formatDuration(seconds: number): string {
        const hours = Math.floor(seconds / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);
        if (hours > 0) {
            return `${hours}h ${minutes}m`;
        }
        return `${minutes}m`;
    }
</script>

<div class="stats-view" transition:fade={{ duration: 200 }}>
    <header class="stats-header">
        <button class="back-btn" onclick={() => uiStore.closeStatsView()}>
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
        <h2>Time Statistics</h2>
    </header>

    {#if loading}
        <div class="loading-state">
            <div class="spinner"></div>
            <p>Loading statistics...</p>
        </div>
    {:else if error}
        <div class="error-state">
            <p>{error}</p>
        </div>
    {:else if stats || windowStats}
        <div class="stats-content">
            <!-- Today's Activity Section -->
            <section
                class="stats-section"
                transition:fly={{ y: 20, duration: 300, delay: 100 }}
            >
                <h3>
                    <span class="section-icon">📅</span>
                    Today's Activity
                </h3>
                {#if windowTrackingStore.enabled && windowStats}
                    {#if groupedTodayApps.length === 0}
                        <div class="empty-state">
                            <p>No active-window time tracked today</p>
                        </div>
                    {:else}
                        <div class="bar-chart horizontal">
                            {#each groupedTodayApps as app}
                                <div class="bar-item">
                                    <div class="bar-label">
                                        <span class="task-name" title={app.app_name}
                                            >{app.app_name}</span
                                        >
                                        <span class="task-time"
                                            >{formatDuration(
                                                app.total_seconds,
                                            )}</span
                                        >
                                    </div>
                                    <div class="bar-track">
                                        <div
                                            class="bar-fill"
                                            style="width: {(app.total_seconds /
                                                maxTodaySeconds) *
                                                100}%; background-color: {app.color}"
                                        ></div>
                                    </div>
                                    <span
                                        class="project-tag"
                                        style="color: {app.color}"
                                        >{app.kind === "afk"
                                            ? "Away time"
                                            : "Active window"}</span
                                    >
                                </div>
                            {/each}
                        </div>
                    {/if}
                {:else if stats && stats.today_tasks.length === 0}
                    <div class="empty-state">
                        <p>No time tracked today</p>
                    </div>
                {:else if stats}
                    <div class="bar-chart horizontal">
                        {#each stats.today_tasks as task}
                            <div class="bar-item">
                                <div class="bar-label">
                                    <span
                                        class="task-name"
                                        title={task.task_title}
                                        >{task.task_title}</span
                                    >
                                    <span class="task-time"
                                        >{formatDuration(
                                            task.total_seconds,
                                        )}</span
                                    >
                                </div>
                                <div class="bar-track">
                                    <div
                                        class="bar-fill"
                                        style="width: {(task.total_seconds /
                                            maxTodaySeconds) *
                                            100}%; background-color: {task.project_color ||
                                            'var(--accent)'}"
                                    ></div>
                                </div>
                                {#if task.project_name}
                                    <span
                                        class="project-tag"
                                        style="color: {task.project_color ||
                                            'var(--text-tertiary)'}"
                                        >{task.project_name}</span
                                    >
                                {/if}
                            </div>
                        {/each}
                    </div>
                {/if}
            </section>

            <!-- This Week Section -->
            <section
                class="stats-section"
                transition:fly={{ y: 20, duration: 300, delay: 200 }}
            >
                <h3>
                    <span class="section-icon">📊</span>
                    This Week
                </h3>
                <div class="week-chart">
                    {#each weekData as dayData, i}
                        <div class="week-bar-container">
                            <div class="week-bar-value">
                                {#if dayData.seconds > 0}
                                    {formatDuration(dayData.seconds)}
                                {/if}
                            </div>
                            <div class="week-bar-track">
                                <div
                                    class="week-bar-fill"
                                    style="height: {maxWeekSeconds > 0
                                        ? (dayData.seconds / maxWeekSeconds) *
                                          100
                                        : 0}%"
                                ></div>
                            </div>
                            <div class="week-bar-label">{dayData.day}</div>
                        </div>
                    {/each}
                </div>
            </section>

            <!-- By Project Section -->
            <section
                class="stats-section"
                transition:fly={{ y: 20, duration: 300, delay: 300 }}
            >
                <h3>
                    <span class="section-icon">📁</span>
                    {windowTrackingStore.enabled ? "By Activity" : "By Project"}
                </h3>
                {#if windowTrackingStore.enabled && windowStats}
                    {#if groupedApps.length === 0}
                        <div class="empty-state">
                            <p>No activity data yet</p>
                        </div>
                    {:else}
                        <div class="pie-section">
                            <svg class="pie-chart" viewBox="0 0 100 100">
                                {#each getPieSlices(groupedApps) as slice}
                                    <path
                                        d={describeArc(
                                            50,
                                            50,
                                            45,
                                            slice.startAngle,
                                            slice.endAngle,
                                        )}
                                        fill={slice.color}
                                        class="pie-slice"
                                    />
                                {/each}
                                <circle
                                    cx="50"
                                    cy="50"
                                    r="25"
                                    fill="var(--bg-primary)"
                                />
                                <text
                                    x="50"
                                    y="48"
                                    text-anchor="middle"
                                    class="pie-total-label">Total</text
                                >
                                <text
                                    x="50"
                                    y="56"
                                    text-anchor="middle"
                                    class="pie-total-value"
                                >
                                    {formatDuration(totalProjectSeconds)}
                                </text>
                            </svg>
                            <div class="pie-legend">
                                {#each groupedApps as app}
                                    <div class="legend-item">
                                        <span
                                            class="legend-color"
                                            style="background-color: {app.color}"
                                        ></span>
                                        <span class="legend-name">{app.app_name}</span>
                                        <span class="legend-time"
                                            >{formatDuration(
                                                app.total_seconds,
                                            )}</span
                                        >
                                    </div>
                                {/each}
                            </div>
                        </div>
                    {/if}
                {:else if stats && stats.projects.length === 0}
                    <div class="empty-state">
                        <p>No project data yet</p>
                    </div>
                {:else if stats}
                    <div class="pie-section">
                        <svg class="pie-chart" viewBox="0 0 100 100">
                            {#each getPieSlices(stats.projects) as slice}
                                <path
                                    d={describeArc(
                                        50,
                                        50,
                                        45,
                                        slice.startAngle,
                                        slice.endAngle,
                                    )}
                                    fill={slice.color}
                                    class="pie-slice"
                                />
                            {/each}
                            <circle
                                cx="50"
                                cy="50"
                                r="25"
                                fill="var(--bg-primary)"
                            />
                            <text
                                x="50"
                                y="48"
                                text-anchor="middle"
                                class="pie-total-label">Total</text
                            >
                            <text
                                x="50"
                                y="56"
                                text-anchor="middle"
                                class="pie-total-value"
                            >
                                {formatDuration(totalProjectSeconds)}
                            </text>
                        </svg>
                        <div class="pie-legend">
                            {#each stats.projects as project}
                                <div class="legend-item">
                                    <span
                                        class="legend-color"
                                        style="background-color: {project.color}"
                                    ></span>
                                    <span class="legend-name"
                                        >{project.name}</span
                                    >
                                    <span class="legend-time"
                                        >{formatDuration(
                                            project.total_seconds,
                                        )}</span
                                    >
                                </div>
                            {/each}
                        </div>
                    </div>
                {/if}
            </section>
        </div>
    {/if}
</div>

<style>
    .stats-view {
        display: flex;
        flex-direction: column;
        height: 100%;
        overflow: hidden;
        background-color: var(--bg-primary);
    }

    .stats-header {
        display: flex;
        align-items: center;
        gap: var(--spacing-md);
        padding: var(--spacing-md);
        border-bottom: 1px solid var(--border);
        background-color: var(--bg-secondary);
    }

    .stats-header h2 {
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

    .stats-content {
        flex: 1;
        overflow-y: auto;
        padding: var(--spacing-md);
        display: flex;
        flex-direction: column;
        gap: var(--spacing-lg);
    }

    .stats-section {
        background-color: var(--bg-secondary);
        border: 1px solid var(--border);
        border-radius: var(--radius-lg);
        padding: var(--spacing-md);
    }

    .stats-section h3 {
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

    .loading-state,
    .error-state,
    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: var(--spacing-lg);
        color: var(--text-tertiary);
        font-size: 13px;
    }

    .spinner {
        width: 24px;
        height: 24px;
        border: 2px solid var(--border);
        border-top-color: var(--accent);
        border-radius: 50%;
        animation: spin 0.8s linear infinite;
        margin-bottom: var(--spacing-sm);
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    /* Horizontal Bar Chart */
    .bar-chart.horizontal {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-sm);
    }

    .bar-item {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .bar-label {
        display: flex;
        justify-content: space-between;
        font-size: 12px;
    }

    .task-name {
        color: var(--text-primary);
        font-weight: 500;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        max-width: 60%;
    }

    .task-time {
        color: var(--text-secondary);
        font-family: var(--font-mono);
        font-size: 11px;
    }

    .bar-track {
        height: 8px;
        background-color: var(--bg-tertiary);
        border-radius: 4px;
        overflow: hidden;
    }

    .bar-fill {
        height: 100%;
        border-radius: 4px;
        transition: width 0.5s ease;
    }

    .project-tag {
        font-size: 10px;
        font-weight: 500;
    }

    /* Week Chart */
    .week-chart {
        display: flex;
        justify-content: space-between;
        align-items: flex-end;
        height: 140px;
        gap: 8px;
        padding-top: var(--spacing-md);
    }

    .week-bar-container {
        flex: 1;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 4px;
        height: 100%;
    }

    .week-bar-value {
        font-size: 9px;
        color: var(--text-tertiary);
        font-family: var(--font-mono);
        height: 14px;
        min-height: 14px;
    }

    .week-bar-track {
        flex: 1;
        width: 100%;
        background-color: var(--bg-tertiary);
        border-radius: 4px;
        display: flex;
        align-items: flex-end;
        overflow: hidden;
    }

    .week-bar-fill {
        width: 100%;
        background: linear-gradient(
            180deg,
            var(--accent) 0%,
            var(--accent-hover) 100%
        );
        border-radius: 4px;
        transition: height 0.5s ease;
    }

    .week-bar-label {
        font-size: 10px;
        color: var(--text-tertiary);
        font-weight: 500;
    }

    /* Pie Chart */
    .pie-section {
        display: flex;
        gap: var(--spacing-md);
        align-items: center;
    }

    .pie-chart {
        width: 120px;
        height: 120px;
        flex-shrink: 0;
    }

    .pie-slice {
        transition: opacity 0.2s;
    }

    .pie-slice:hover {
        opacity: 0.8;
    }

    .pie-total-label {
        font-size: 6px;
        fill: var(--text-tertiary);
    }

    .pie-total-value {
        font-size: 7px;
        font-weight: 600;
        fill: var(--text-primary);
        font-family: var(--font-mono);
    }

    .pie-legend {
        flex: 1;
        display: flex;
        flex-direction: column;
        gap: 6px;
        min-width: 0;
    }

    .legend-item {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 12px;
    }

    .legend-color {
        width: 10px;
        height: 10px;
        border-radius: 2px;
        flex-shrink: 0;
    }

    .legend-name {
        flex: 1;
        color: var(--text-primary);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .legend-time {
        color: var(--text-secondary);
        font-family: var(--font-mono);
        font-size: 11px;
        flex-shrink: 0;
    }
</style>
