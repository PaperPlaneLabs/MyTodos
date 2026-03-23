## Maintainability Phase 1 Plan

### Goal
- Reduce the cost and risk of everyday changes without changing product behavior.
- Prioritize refactors that make timer work, UI work, and app startup logic easier to reason about.

### Why This First
- `src/routes/+page.svelte` is the biggest frontend hotspot and currently mixes page composition, modal orchestration, drag/drop, timer actions, and deletion flows.
- `src/lib/stores/timer.svelte.ts` combines timer lifecycle, break reminders, UI syncing, and backend event handling in one place.
- `src-tauri/src/lib.rs` and `src-tauri/src/commands/timer.rs` are accumulating orchestration and domain logic that should be easier to test in isolation.

### Non-Goals
- No feature redesigns.
- No behavior changes to timer semantics unless required to preserve current behavior during extraction.
- No broad styling rewrites.

### Workstreams

### 1. Frontend Route Decomposition
- Refactor `src/routes/+page.svelte` into page composition plus focused feature components.
- Extract project list, task list, timer widget, and modal host responsibilities into separate files.
- Keep data ownership in stores, but move view-only orchestration out of the route.

Target outcome:
- `+page.svelte` becomes primarily a composition layer.
- Feature changes stop requiring edits across hundreds of unrelated lines.

### 2. Timer Store Decomposition
- Split `src/lib/stores/timer.svelte.ts` by concern while preserving the existing public API.
- Separate timer lifecycle, break reminder scheduling, auto-pause event handling, and derived display totals.
- Keep one exported `timerStore`, but back it with smaller internal modules.

Target outcome:
- Timer behavior becomes easier to test and safer to extend.
- Future work like idle detection or resume prompts can land in isolated modules.

### 3. Rust Timer Service Boundary
- Move timer business rules out of `src-tauri/src/commands/timer.rs` into a reusable timer service layer.
- Keep Tauri command files thin and focused on IPC argument handling.
- Share timer pause/recovery logic between commands and system event handlers through the service instead of direct command-level helpers.

Target outcome:
- Runtime behavior is easier to test without duplicating logic.
- Backend timer logic gains a clear “single source of truth.”

### 4. App Bootstrap Cleanup
- Reduce responsibilities in `src-tauri/src/lib.rs` by extracting startup and lifecycle concerns.
- Separate setup for tray, startup recovery, system listeners, and window lifecycle.
- Keep the main `run()` path readable and declarative.

Target outcome:
- Startup changes become localized.
- Platform-specific lifecycle behavior is easier to follow.

### 5. Quality Gates
- Add or tighten automation for:
  - `npm run check`
  - `npm run test`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
- Treat existing warnings as debt to burn down rather than background noise.

Recommended sequence:
1. `npm run check`
2. `npm run test`
3. `cargo test`
4. `cargo clippy --all-targets -- -D warnings`

Target outcome:
- Refactors are protected by consistent automated feedback.

### Suggested Sequence
1. Decompose `+page.svelte` first because it lowers frontend change risk immediately.
2. Decompose `timer.svelte.ts` next because it is the highest-value shared state hotspot.
3. Extract the Rust timer service after the frontend timer split, using the stable API shape as a guide.
4. Clean up `lib.rs` once timer service boundaries are clearer.
5. Finish by tightening CI and addressing the remaining warnings uncovered during refactors.

### Acceptance Criteria
- The app behavior remains unchanged for core task and timer flows.
- `+page.svelte` is substantially smaller and acts mainly as a composition shell.
- `timer.svelte.ts` is reduced to a thin public facade or removed in favor of smaller timer modules.
- Backend timer logic can be exercised without depending on Tauri commands directly.
- CI or local quality gates cover both frontend and backend checks.

### Risks
- UI extraction can accidentally break event wiring if moved too quickly.
- Timer refactors can introduce state drift unless the public store contract remains stable.
- Backend extraction can duplicate logic temporarily if the service boundary is not defined first.

### Mitigations
- Refactor one seam at a time and keep behavior identical until after extraction.
- Add targeted tests around timer lifecycle before and during store/service extraction.
- Prefer adapter layers over changing every call site at once.
