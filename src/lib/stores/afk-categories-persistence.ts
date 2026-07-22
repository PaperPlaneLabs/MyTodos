export const DEFAULT_AFK_CATEGORIES = ["Meeting", "Lunch", "Snack"];

export interface LegacyAfkCategories {
  categories: string[];
  hasStoredValue: boolean;
}

interface AfkCategoryPersistence {
  getAll(): Promise<string[]>;
  merge(names: string[]): Promise<string[]>;
}

export function normalizeAfkCategoryLabel(value: string): string {
  return value.trim().replace(/\s+/g, " ");
}

export function normalizeAfkCategories(
  values: string[],
  reservedLabel: string,
): string[] {
  const uniqueCategories: string[] = [];
  const seen = new Set<string>();
  const reservedKey = reservedLabel.toLocaleLowerCase();

  for (const value of values) {
    const normalizedValue = normalizeAfkCategoryLabel(value);
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

export function parseLegacyAfkCategories(
  storedValue: string | null,
  reservedLabel: string,
): LegacyAfkCategories {
  if (storedValue === null) {
    return {
      categories: [...DEFAULT_AFK_CATEGORIES],
      hasStoredValue: false,
    };
  }

  try {
    const parsedValue: unknown = JSON.parse(storedValue);
    if (!Array.isArray(parsedValue)) {
      return {
        categories: [...DEFAULT_AFK_CATEGORIES],
        hasStoredValue: true,
      };
    }

    return {
      categories: normalizeAfkCategories(
        parsedValue.filter(
          (value): value is string => typeof value === "string",
        ),
        reservedLabel,
      ),
      hasStoredValue: true,
    };
  } catch {
    return {
      categories: [...DEFAULT_AFK_CATEGORIES],
      hasStoredValue: true,
    };
  }
}

export async function loadDurableAfkCategories(
  persistence: AfkCategoryPersistence,
  legacy: LegacyAfkCategories,
  clearLegacy: () => void,
): Promise<string[]> {
  if (!legacy.hasStoredValue) {
    return persistence.getAll();
  }

  const categories = await persistence.merge(legacy.categories);
  clearLegacy();
  return categories;
}
