# Timer Pause/Resume Continuity Walkthrough

## What Changed
- The timer runtime now keeps a local elapsed offset so the displayed timer can continue across pauses without affecting the running-segment calculations used elsewhere.
- `timerStore.pause()` and auto-pause now preserve the current displayed elapsed time instead of clearing it.
- `timerStore.resume()` now resumes from the preserved elapsed value instead of resetting the widget back to zero.
- Added a focused Vitest regression in [timer-runtime-utils.test.ts](/f:/personal_projects/MyTodos/src/lib/stores/timer-runtime-utils.test.ts) to verify resumed timers continue from the paused elapsed value.

## Verification
- `npx vitest run src/lib/stores/timer-runtime-utils.test.ts`
  Result: passed
- `npm run check`
  Result: 0 errors, 6 warnings

## Notes
- The `svelte-check` warnings are pre-existing accessibility warnings in [AppHeader.svelte](/f:/personal_projects/MyTodos/src/lib/components/layout/AppHeader.svelte) and [SettingsView.svelte](/f:/personal_projects/MyTodos/src/lib/components/settings/SettingsView.svelte).
- This fix preserves elapsed continuity for pause/resume during the current app session. If you also want the same paused elapsed value restored after closing and reopening the app, that would need a backend persistence change because `pause_timer` currently writes the segment to `time_entries` and resets `active_timer.elapsed_seconds` to `0`.

# WCAG Accessibility Remediation Walkthrough

## What Changed
- Strengthened the shared accessibility foundation in [global.css](/f:/personal_projects/MyTodos/src/lib/styles/global.css), [theme.css](/f:/personal_projects/MyTodos/src/lib/styles/theme.css), and [Modal.svelte](/f:/personal_projects/MyTodos/src/lib/components/common/Modal.svelte):
  Added visible focus styles, reduced-motion handling, improved contrast tokens for themed controls, and real dialog semantics with focus management.
- Reworked custom controls in [AppHeader.svelte](/f:/personal_projects/MyTodos/src/lib/components/layout/AppHeader.svelte), [SettingsView.svelte](/f:/personal_projects/MyTodos/src/lib/components/settings/SettingsView.svelte), [DateTimePicker.svelte](/f:/personal_projects/MyTodos/src/lib/components/common/DateTimePicker.svelte), [CalendarHeader.svelte](/f:/personal_projects/MyTodos/src/lib/components/calendar/CalendarHeader.svelte), and [ContextMenu.svelte](/f:/personal_projects/MyTodos/src/lib/components/common/ContextMenu.svelte):
  Added accessible names and state attributes, replaced clickable backdrop `<div>` elements with real buttons or native form controls, and added keyboard navigation where menu semantics were already exposed.
- Fixed list and task control accessibility in [TaskListSection.svelte](/f:/personal_projects/MyTodos/src/lib/components/tasks/TaskListSection.svelte) and [ProjectListSection.svelte](/f:/personal_projects/MyTodos/src/lib/components/projects/ProjectListSection.svelte):
  Labeled task checkboxes and timer actions, removed fake button semantics from non-activatable task rows, and added keyboard-based reordering on the project/task drag handles.
- Tightened remaining color-contrast hotspots in several surfaces, including [CalendarDayCell.svelte](/f:/personal_projects/MyTodos/src/lib/components/calendar/CalendarDayCell.svelte), [DayTaskList.svelte](/f:/personal_projects/MyTodos/src/lib/components/calendar/DayTaskList.svelte), [TimeEntryPanel.svelte](/f:/personal_projects/MyTodos/src/lib/components/calendar/TimeEntryPanel.svelte), [UpdateNotification.svelte](/f:/personal_projects/MyTodos/src/lib/components/common/UpdateNotification.svelte), [BreakView.svelte](/f:/personal_projects/MyTodos/src/lib/components/common/BreakView.svelte), [ResumeView.svelte](/f:/personal_projects/MyTodos/src/lib/components/resume/ResumeView.svelte), and [CollapseHandle.svelte](/f:/personal_projects/MyTodos/src/lib/components/layout/CollapseHandle.svelte).

## Verification
- `npm run check`
  Result: 0 errors, 0 warnings
- Theme-token contrast spot-check
  Result: the shared secondary/tertiary text tokens and accent/success/warning/danger contrast pairs now meet or exceed WCAG AA in the built-in themes.

## Notes
- The original six Svelte accessibility warnings are resolved.
- I did not run live screen-reader testing in this pass, so the remaining risk is behavioral rather than lint-level:
  Focus order, announcement quality, and keyboard reordering should still be spot-checked manually with NVDA/VoiceOver in the running Tauri app.

# Welcome-Back AFK Categories Walkthrough

## What Changed
- Added a shared AFK category store in [afk-categories.svelte.ts](/f:/personal_projects/MyTodos/src/lib/stores/afk-categories.svelte.ts) backed by `localStorage`, with default categories of `Meeting`, `Lunch`, and `Snack`, plus a built-in `Current task related` option for the welcome-back flow.
- Expanded Settings in [SettingsView.svelte](/f:/personal_projects/MyTodos/src/lib/components/settings/SettingsView.svelte) so AFK categories can be added and removed without editing code.
- Reworked the welcome-back experience in [ResumeView.svelte](/f:/personal_projects/MyTodos/src/lib/components/resume/ResumeView.svelte):
  The window now asks why the user was away, can attribute the away time back to the active task, or log it under a named AFK category before resuming or returning to the app.
- Extended the backend time-logging path in [timer_service.rs](/f:/personal_projects/MyTodos/src-tauri/src/services/timer_service.rs), [timer.rs](/f:/personal_projects/MyTodos/src-tauri/src/commands/timer.rs), and [db.ts](/f:/personal_projects/MyTodos/src/lib/services/db.ts):
  Named AFK categories now use the existing system-project/task pattern under an `Away` system project, while current-task-related away time uses a normal manual entry against the active task.
- Added a cross-window sync event in [timer-events.ts](/f:/personal_projects/MyTodos/src/lib/stores/timer-events.ts) and [timer.svelte.ts](/f:/personal_projects/MyTodos/src/lib/stores/timer.svelte.ts) so stats and visible task/project totals refresh after the resume window logs away time.

## Verification
- `cargo test test_log_afk_time -- --nocapture`
  Result: passed, including the new named-AFK logging regression tests in [timer_tests.rs](/f:/personal_projects/MyTodos/src-tauri/tests/timer_tests.rs)
- `npm run check`
  Result: 0 errors, 0 warnings

## Notes
- I increased the resume window height in [window.rs](/f:/personal_projects/MyTodos/src-tauri/src/commands/window.rs) so the new reason picker fits without crowding.
- Follow-up stabilization:
  The AFK category store now loads its initial state without mutating Svelte state during derived/template evaluation, and [+page.svelte](/f:/personal_projects/MyTodos/src/routes/+page.svelte) now skips the main-app bootstrap for the `break` and `resume` windows so child windows stay isolated.
- I did not run a live Tauri interaction test for the full lock/unlock workflow in this pass, so the remaining risk is manual UX validation:
  The welcome-back window should still be clicked through once to confirm the reason picker, resume button, and “save and open app” path all feel right in the desktop app.
