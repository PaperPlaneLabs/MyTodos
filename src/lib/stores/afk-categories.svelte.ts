export const CURRENT_TASK_RELATED_CATEGORY_ID = "__current_task_related__";
export const CURRENT_TASK_RELATED_CATEGORY_LABEL = "Current task related";

const AFK_CATEGORIES_STORAGE_KEY = "afkCategories";
const DEFAULT_AFK_CATEGORIES = ["Meeting", "Lunch", "Snack"];

export interface AfkCategoryOption {
  id: string;
  label: string;
  type: "current-task" | "system";
}

type AddCategoryResult =
  | { added: true; value: string }
  | { added: false; error: string };

function normalizeLabel(value: string): string {
  return value.trim().replace(/\s+/g, " ");
}

function normalizeCategories(values: string[]): string[] {
  const uniqueCategories: string[] = [];
  const seen = new Set<string>();
  const reservedKey = CURRENT_TASK_RELATED_CATEGORY_LABEL.toLocaleLowerCase();

  for (const value of values) {
    const normalizedValue = normalizeLabel(value);
    const normalizedKey = normalizedValue.toLocaleLowerCase();

    if (
      !normalizedValue ||
      normalizedKey === reservedKey ||
      seen.has(normalizedKey)
    ) {
      continue;
    }

    seen.add(normalizedKey);
    uniqueCategories.push(normalizedValue);
  }

  return uniqueCategories;
}

function loadStoredCategories(): string[] {
  if (typeof window === "undefined") {
    return [...DEFAULT_AFK_CATEGORIES];
  }

  const storedValue = localStorage.getItem(AFK_CATEGORIES_STORAGE_KEY);
  if (storedValue === null) {
    return [...DEFAULT_AFK_CATEGORIES];
  }

  try {
    const parsedValue = JSON.parse(storedValue);
    if (!Array.isArray(parsedValue)) {
      return [...DEFAULT_AFK_CATEGORIES];
    }

    return normalizeCategories(
      parsedValue.filter((value): value is string => typeof value === "string"),
    );
  } catch (error) {
    console.error("Failed to parse AFK categories:", error);
    return [...DEFAULT_AFK_CATEGORIES];
  }
}

let customCategories = $state<string[]>(loadStoredCategories());

function persistCategories(): void {
  if (typeof window === "undefined") {
    return;
  }

  localStorage.setItem(
    AFK_CATEGORIES_STORAGE_KEY,
    JSON.stringify(customCategories),
  );
}

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

  init() {
    customCategories = loadStoredCategories();
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

  addCategory(value: string): AddCategoryResult {
    const normalizedValue = normalizeLabel(value);
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

    customCategories = [...customCategories, normalizedValue];
    persistCategories();

    return { added: true, value: normalizedValue };
  },

  removeCategory(value: string): void {
    const normalizedValue = normalizeLabel(value).toLocaleLowerCase();
    customCategories = customCategories.filter(
      (category) => category.toLocaleLowerCase() !== normalizedValue,
    );

    persistCategories();
  },
};
