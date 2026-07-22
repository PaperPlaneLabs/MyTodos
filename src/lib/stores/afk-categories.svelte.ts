import { db } from "$lib/services/db";
import {
  loadDurableAfkCategories,
  normalizeAfkCategoryLabel,
  parseLegacyAfkCategories,
  type LegacyAfkCategories,
} from "$lib/stores/afk-categories-persistence";

export const CURRENT_TASK_RELATED_CATEGORY_ID = "__current_task_related__";
export const CURRENT_TASK_RELATED_CATEGORY_LABEL = "Current task related";

const AFK_CATEGORIES_STORAGE_KEY = "afkCategories";

export interface AfkCategoryOption {
  id: string;
  label: string;
  type: "current-task" | "system";
}

type AddCategoryResult =
  | { added: true; value: string }
  | { added: false; error: string };

function loadStoredCategories(): LegacyAfkCategories {
  if (typeof window === "undefined") {
    return parseLegacyAfkCategories(null, CURRENT_TASK_RELATED_CATEGORY_LABEL);
  }

  const storedValue = localStorage.getItem(AFK_CATEGORIES_STORAGE_KEY);
  return parseLegacyAfkCategories(
    storedValue,
    CURRENT_TASK_RELATED_CATEGORY_LABEL,
  );
}

const initialCategories = loadStoredCategories();
let customCategories = $state<string[]>(initialCategories.categories);
let initialized = false;
let initPromise: Promise<void> | null = null;
let error = $state<string | null>(null);

function buildOptions(includeCurrentTask: boolean): AfkCategoryOption[] {
  const options: AfkCategoryOption[] = customCategories.map((label) => ({
    id: label,
    label,
    type: "system",
  }));

  if (includeCurrentTask) {
    options.unshift({
      id: CURRENT_TASK_RELATED_CATEGORY_ID,
      label: CURRENT_TASK_RELATED_CATEGORY_LABEL,
      type: "current-task",
    });
  }

  return options;
}

export const afkCategoryStore = {
  get customCategories() {
    return customCategories;
  },

  get error() {
    return error;
  },

  async init(force = false): Promise<void> {
    if (!force && initialized) return;
    if (initPromise) return initPromise;

    initPromise = (async () => {
      const legacy = loadStoredCategories();
      try {
        error = null;
        customCategories = await loadDurableAfkCategories(
          db.afkCategories,
          legacy,
          () => localStorage.removeItem(AFK_CATEGORIES_STORAGE_KEY),
        );
        initialized = true;
      } catch (loadError) {
        error =
          loadError instanceof Error
            ? loadError.message
            : "Failed to load AFK categories";
        customCategories = legacy.categories;
        console.error("Failed to load AFK categories:", loadError);
      } finally {
        initPromise = null;
      }
    })();

    return initPromise;
  },

  buildOptions(includeCurrentTask: boolean): AfkCategoryOption[] {
    return buildOptions(includeCurrentTask);
  },

  isCurrentTaskCategory(categoryId: string): boolean {
    return categoryId === CURRENT_TASK_RELATED_CATEGORY_ID;
  },

  getDefaultSelection(includeCurrentTask: boolean): string {
    const options = buildOptions(includeCurrentTask);
    return options[0]?.id ?? "";
  },

  async addCategory(value: string): Promise<AddCategoryResult> {
    const normalizedValue = normalizeAfkCategoryLabel(value);
    if (!normalizedValue) {
      return {
        added: false,
        error: "Enter a category name before adding it.",
      };
    }

    if (
      normalizedValue.toLocaleLowerCase() ===
      CURRENT_TASK_RELATED_CATEGORY_LABEL.toLocaleLowerCase()
    ) {
      return {
        added: false,
        error: `"${CURRENT_TASK_RELATED_CATEGORY_LABEL}" is already built in.`,
      };
    }

    if (
      customCategories.some(
        (category) =>
          category.toLocaleLowerCase() === normalizedValue.toLocaleLowerCase(),
      )
    ) {
      return {
        added: false,
        error: `"${normalizedValue}" is already in your AFK list.`,
      };
    }

    try {
      const savedValue = await db.afkCategories.add(normalizedValue);
      customCategories = [...customCategories, savedValue];
      error = null;
      return { added: true, value: savedValue };
    } catch (saveError) {
      error =
        saveError instanceof Error
          ? saveError.message
          : "Failed to save AFK category";
      return { added: false, error };
    }
  },

  async removeCategory(value: string): Promise<void> {
    const normalizedValue = normalizeAfkCategoryLabel(value).toLocaleLowerCase();
    await db.afkCategories.remove(value);
    customCategories = customCategories.filter(
      (category) => category.toLocaleLowerCase() !== normalizedValue,
    );
    error = null;
  },
};
