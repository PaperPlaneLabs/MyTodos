import { describe, expect, it, vi } from "vitest";

import {
  loadDurableAfkCategories,
  parseLegacyAfkCategories,
} from "$lib/stores/afk-categories-persistence";

describe("AFK category persistence", () => {
  it("merges legacy categories into SQLite before clearing localStorage", async () => {
    const legacy = parseLegacyAfkCategories(
      JSON.stringify(["Meeting", " Others ", "others"]),
      "Current task related",
    );
    const clearLegacy = vi.fn();
    const merge = vi
      .fn<(names: string[]) => Promise<string[]>>()
      .mockResolvedValue(["Meeting", "Lunch", "Snack", "Others"]);

    const categories = await loadDurableAfkCategories(
      { getAll: vi.fn(), merge },
      legacy,
      clearLegacy,
    );

    expect(merge).toHaveBeenCalledWith(["Meeting", "Others"]);
    expect(clearLegacy).toHaveBeenCalledOnce();
    expect(categories).toEqual(["Meeting", "Lunch", "Snack", "Others"]);
  });

  it("loads SQLite directly when no legacy value exists", async () => {
    const legacy = parseLegacyAfkCategories(null, "Current task related");
    const getAll = vi
      .fn<() => Promise<string[]>>()
      .mockResolvedValue(["Meeting", "Lunch", "Snack", "Others"]);
    const merge = vi.fn<(names: string[]) => Promise<string[]>>();
    const clearLegacy = vi.fn();

    const categories = await loadDurableAfkCategories(
      { getAll, merge },
      legacy,
      clearLegacy,
    );

    expect(getAll).toHaveBeenCalledOnce();
    expect(merge).not.toHaveBeenCalled();
    expect(clearLegacy).not.toHaveBeenCalled();
    expect(categories).toContain("Others");
  });
});
