import type { PrimaryView } from "$lib/stores/ui.svelte";

export function shouldShowBottomTimer(primaryView: PrimaryView): boolean {
  return primaryView === "projects";
}

export function shouldSelectWeekGridTarget(target: Element | null): boolean {
  return !target?.closest(".timed-item, .resize-handle");
}
