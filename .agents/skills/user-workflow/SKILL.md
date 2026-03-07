---
name: user-workflow
description: Artifact-driven implementation workflow for MyTodos
---

# Artifact-Driven Implementation Workflow

This skill documents the preferred development methodology for working on the MyTodos codebase, emphasizing clarity, traceability, and verified progress.

## 1. Context First
Before proposing changes, always read:
- `AGENTS.md`: Technical constraints and build commands.
- `GEMINI.md`: Architectural overview and SPA patterns.
- Existing logic in `timer.svelte.ts` (state) and `db.ts` (bridge).

## 2. Iterative Documentation
Follow these stage-dependent artifact steps:

### Phase 1: Planning
- **`implementation_plan.md`**: Define the "Why" and "What". Group changes by component.
- **`task.md`**: Create a granular checklist.

### Phase 2: Execution
- Update `task.md` frequently (use `[/]` for in-progress).
- Call `task_boundary` before major tool batches.

### Phase 3: Verification
- **`walkthrough.md`**: Summarize accomplishments, note any deviations from the plan, and provide proof (e.g., successful build logs or manual test results).
- Cleanup: Remove test buttons, logs, and obsolete routes before finishing.

## 3. Communication Style
- Be proactive but non-intrusive.
- Batch independent questions to minimize interruptions.
- Focus on technical decisions in `notify_user` calls.
