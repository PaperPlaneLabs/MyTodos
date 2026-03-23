# Dock Preference Persistence Walkthrough

## What Changed
- The existing `window_state` table now stores an optional `dock_preference` so the last clicked docking mode can survive relaunches.
- New backend helpers and Tauri commands save and load the dock preference, and startup applies it to the main window before normal app initialization continues.
- The header dock buttons and Settings docking control now persist the chosen mode after applying it.
- The main page initializes `uiStore.windowOrientation` from the saved preference, falling back to the live window orientation when nothing has been saved yet.

## Verification
- `cargo test --test window_dock_preference_tests`
  Result: passed
- `npm run check`
  Result: 0 errors, 6 warnings

## Notes
- The `svelte-check` warnings are pre-existing accessibility warnings in [AppHeader.svelte](/f:/personal_projects/MyTodos/src/lib/components/layout/AppHeader.svelte) and [SettingsView.svelte](/f:/personal_projects/MyTodos/src/lib/components/settings/SettingsView.svelte).
