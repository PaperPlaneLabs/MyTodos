export function resolveTaskTimerMinutes(
  customMinutes: number | undefined,
  selectedMinutes: number,
): number {
  return customMinutes ?? selectedMinutes;
}
