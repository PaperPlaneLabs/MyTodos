export function parseTimeToSeconds(input: string): number {
  const trimmed = input.trim().toLowerCase();

  let totalSeconds = 0;

  const hoursMatch = trimmed.match(/(\d+\.?\d*)\s*h/);
  if (hoursMatch) {
    totalSeconds += parseFloat(hoursMatch[1]) * 3600;
  }

  const minutesMatch = trimmed.match(/(\d+\.?\d*)\s*m(?!s)/);
  if (minutesMatch) {
    totalSeconds += parseFloat(minutesMatch[1]) * 60;
  }

  const secondsMatch = trimmed.match(/(\d+\.?\d*)\s*s/);
  if (secondsMatch) {
    totalSeconds += parseFloat(secondsMatch[1]);
  }

  const plainNumberMatch = trimmed.match(/^(\d+\.?\d*)$/);
  if (plainNumberMatch && totalSeconds === 0) {
    totalSeconds = parseFloat(plainNumberMatch[1]) * 60;
  }

  return Math.round(totalSeconds);
}

export function formatSecondsToTime(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  } else if (minutes > 0) {
    return `${minutes}m ${secs}s`;
  } else {
    return `${secs}s`;
  }
}

export function formatSecondsToHHMMSS(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;

  const pad = (n: number) => n.toString().padStart(2, "0");

  return `${pad(hours)}:${pad(minutes)}:${pad(secs)}`;
}
