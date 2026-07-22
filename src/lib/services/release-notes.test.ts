import { describe, expect, it } from "vitest";

import {
  compareVersions,
  getReleaseHistory,
  getUnseenReleaseNotes,
  releaseNotes,
  type ReleaseNote,
} from "$lib/services/release-notes";

const notes: ReleaseNote[] = ["0.1.65", "0.1.66", "0.2.0"].map(
  (version) => ({
    version,
    date: "2026-07-22",
    title: version,
    summary: "Summary",
    highlights: [],
    fixes: [],
  }),
);

describe("release notes", () => {
  it("contains unique, complete release entries", () => {
    const versions = releaseNotes.map((note) => note.version);
    expect(new Set(versions).size).toBe(versions.length);
    for (const note of releaseNotes) {
      expect(note.version).toMatch(/^\d+\.\d+\.\d+$/);
      expect(note.title.trim()).not.toBe("");
      expect(note.summary.trim()).not.toBe("");
      expect(note.highlights.length + note.fixes.length).toBeGreaterThan(0);
    }
  });

  it("compares numeric semantic version segments", () => {
    expect(compareVersions("0.1.10", "0.1.9")).toBeGreaterThan(0);
    expect(compareVersions("1.0.0", "1.0.0")).toBe(0);
  });

  it("shows only the current release when no version was previously recorded", () => {
    expect(getUnseenReleaseNotes("0.1.66", null, notes)).toEqual([notes[1]]);
  });

  it("shows every skipped release up to the installed version", () => {
    expect(getUnseenReleaseNotes("0.2.0", "0.1.65", notes)).toEqual([
      notes[1],
      notes[2],
    ]);
  });

  it("orders release history newest first", () => {
    expect(getReleaseHistory(notes).map((note) => note.version)).toEqual([
      "0.2.0",
      "0.1.66",
      "0.1.65",
    ]);
  });
});
