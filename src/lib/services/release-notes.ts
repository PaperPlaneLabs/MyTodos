import releaseNotesJson from "$lib/data/releases.json";

export interface ReleaseNote {
  version: string;
  date: string;
  title: string;
  summary: string;
  highlights: string[];
  fixes: string[];
}

export const releaseNotes = releaseNotesJson as ReleaseNote[];

export function compareVersions(left: string, right: string): number {
  const parse = (version: string) =>
    version.split(".").map((part) => Number.parseInt(part, 10) || 0);
  const leftParts = parse(left);
  const rightParts = parse(right);
  const length = Math.max(leftParts.length, rightParts.length);

  for (let index = 0; index < length; index += 1) {
    const difference = (leftParts[index] ?? 0) - (rightParts[index] ?? 0);
    if (difference !== 0) return difference;
  }
  return 0;
}

export function getUnseenReleaseNotes(
  currentVersion: string,
  lastSeenVersion: string | null,
  notes: ReleaseNote[] = releaseNotes,
): ReleaseNote[] {
  const available = notes.filter(
    (note) => compareVersions(note.version, currentVersion) <= 0,
  );

  const unseen = lastSeenVersion
    ? available.filter(
        (note) => compareVersions(note.version, lastSeenVersion) > 0,
      )
    : available.filter((note) => note.version === currentVersion);

  return unseen.sort((left, right) =>
    compareVersions(left.version, right.version),
  );
}

export function getReleaseHistory(
  notes: ReleaseNote[] = releaseNotes,
): ReleaseNote[] {
  return [...notes].sort((left, right) =>
    compareVersions(right.version, left.version),
  );
}
