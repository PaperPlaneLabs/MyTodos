// Global type augmentations for the app

// Globals injected by Tauri's initialization_script before each window loads
interface Window {
    __BREAK_MESSAGE__?: string;
    __BREAK_THEME__?: string;
    __RESUME_DATA__?: {
        taskId: number | null;
        taskTitle: string;
        awayTimeSeconds: number;
        theme: string | null;
    };
    __TAURI_INTERNALS__?: {
        metadata?: {
            currentWindow?: { label: string };
        };
    };
}
