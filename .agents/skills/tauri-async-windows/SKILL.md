---
name: tauri-async-windows
description: Patterns for deadlock-free multi-window management in Tauri
---

# Tauri Async Multi-window Management

This skill covers the best practices for creating and managing multiple windows in a Tauri application to avoid deadlocks and maintain a clean SPA architecture.

## 1. Async Command Pattern

### The Problem
If a Tauri command that creates a window (using `WebviewWindowBuilder`) is synchronous (`pub fn`), it runs on the main thread. However, `WebviewWindowBuilder::build()` often needs to dispatch work to the main thread. This leads to a deadlock where the command is waiting for the main thread, but the main thread is blocked by the command.

### The Solution: `async fn`
Always define window-creating commands as `async`. This allows Tauri to run them on a background thread pool, leaving the main thread free to handle the window construction.

```rust
// BAD: Synchronous command causing deadlock
#[tauri::command]
pub fn open_window(app: AppHandle) {
    WebviewWindowBuilder::new(&app, "label", Default::default()).build().unwrap();
}

// GOOD: Asynchronous command
#[tauri::command]
pub async fn open_window(app: AppHandle) -> Result<(), String> {
    WebviewWindowBuilder::new(&app, "label", Default::default())
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

## 2. SPA Label-Based Routing

Instead of loading separate HTML files for different windows, use the main SPA (`index.html`) and render different UI components based on the window's label.

### Backend Setup
Use `Default::default()` for the URL in the builder. This points to the application's root (index.html).

```rust
WebviewWindowBuilder::new(&app, "break", Default::default())
    .title("Break Reminder")
    .build()
```

### Frontend Implementation (Svelte 5 Example)
In your main `+page.svelte` (or entry point), check the window label on mount or dynamically via `$state`.

```typescript
// app.d.ts
interface Window {
  __TAURI_INTERNALS__?: {
    metadata?: {
      currentWindow?: {
        label: string;
      };
    };
  };
}

// +page.svelte
const isBreakWindow = $derived(
  window.__TAURI_INTERNALS__?.metadata?.currentWindow?.label === "break"
);
```

```svelte
{#if isBreakWindow}
  <BreakView />
{:else}
  <MainAppUI />
{/if}
```

## 3. Window Positioning
Always use logical coordinates and account for scale factors when positioning windows programmatically to ensure they appear centered or correctly docked on high-DPI displays.
