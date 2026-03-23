# Break Tracking Cleanup Walkthrough

## What Changed
- Break logging still persists through a dedicated project/task pair, but those records are now marked `is_system = 1`.
- Normal project and task queries now exclude system records, so breaks no longer show up in the main project/task UI.
- Daily work totals and visible time-entry queries now exclude system tasks, so break time no longer contributes to the header's cumulative work time.
- Stats remain able to surface break data because the stats queries still read the persisted break entries.
- The stats heading was renamed from `Today's Work` to `Today's Activity` to better fit the fact that breaks can appear there.

## Verification
- `cargo test --test break_tracking_visibility_tests --test time_stats_tests --test time_entries_tests`
  Result: passed
- `npm run check`
  Result: 0 errors, 6 warnings

## Notes
- The `svelte-check` warnings are pre-existing accessibility warnings in [AppHeader.svelte](/f:/personal_projects/MyTodos/src/lib/components/layout/AppHeader.svelte) and [SettingsView.svelte](/f:/personal_projects/MyTodos/src/lib/components/settings/SettingsView.svelte).
