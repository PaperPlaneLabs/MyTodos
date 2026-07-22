---
name: release-mytodos
description: Prepare, publish, monitor, recover, and verify MyTodos desktop releases. Use for version bumps, releases.json What's New entries, create-release.ps1, Git tags, the PaperPlaneLabs/MyTodos GitHub Actions release workflow, updater manifests, release assets, failed pushes, partial releases, or desktop updater JSON errors in this repository.
---

# Release MyTodos

Manage releases from the repository root. Treat publishing and tag changes as
external, potentially destructive actions; inspect state before mutating it.

## Select the operation

- For a new release, follow **Prepare** then **Publish**.
- For an active release, follow **Monitor** then **Verify**.
- For a failed or partial release, follow **Recover**. Do not rerun the release
  script until the existing commit, tag, workflow, and assets are understood.
- For an updater error, inspect the live platform manifest before changing code.

## Establish state

Read `docs/releasing.md`, `create-release.ps1`,
`.github/workflows/release.yml`, `src/lib/data/releases.json`, and
`src-tauri/tauri.conf.json`. Then run:

```powershell
git status --short
git remote -v
git log -3 --oneline --decorate
gh auth status
gh repo view PaperPlaneLabs/MyTodos --json nameWithOwner,url,defaultBranchRef
```

Use `PaperPlaneLabs/MyTodos` as the canonical release repository. Confirm the
worktree changes are intentional: `create-release.ps1` stages everything with
`git add -A`.

## Prepare

1. Choose an unused `X.Y.Z` version. Check local and remote tags.
2. Add exactly one matching entry to `src/lib/data/releases.json`. Include a
   title, summary, and at least one highlight or fix. Describe user-visible
   behavior, not implementation details.
3. Run the non-mutating gate:

```powershell
.\create-release.ps1 -v X.Y.Z -ValidateOnly
```

4. Run checks proportional to the changed product code. Always run
   `git diff --check`; validate PowerShell syntax and workflow YAML when release
   tooling changed.
5. Reinspect `git status --short` and the pending diff before publication.

Do not manually synchronize version files. The release script updates
`package.json`, both npm lockfile root versions, `tauri.conf.json`,
`Cargo.toml`, and the root `my-todos` package in `Cargo.lock`.

## Publish

Prefer the CI-backed multi-platform path:

```powershell
.\create-release.ps1 -v X.Y.Z -Online
```

The script validates notes, loads signing configuration, updates versions,
commits, creates an annotated tag, and pushes the branch and tags. Online mode
then lets `.github/workflows/release.yml` build all platforms.

Do not declare the release complete when the push succeeds. Continue through
monitoring and live updater verification.

## Monitor

Find and watch the tag-triggered run:

```powershell
gh run list --workflow release.yml --repo PaperPlaneLabs/MyTodos --limit 5
gh run watch RUN_ID --repo PaperPlaneLabs/MyTodos --exit-status
```

The `upload-updater-jsons` job waits for every platform build. During that
window the GitHub release can exist while `latest-windows-x86_64.json` and the
other platform JSONs still return 404. Report this as publication in progress,
not malformed JSON, when the workflow is still running.

## Verify

After Actions succeeds, run the bundled read-only verifier:

```powershell
.\.agents\skills\release-mytodos\scripts\verify-release.ps1 -Version X.Y.Z
```

Require all of the following before reporting success:

- the tag workflow completed successfully;
- the GitHub release is published, not draft or prerelease;
- each `releases/latest/download/latest-<platform>.json` returns valid JSON;
- every manifest reports `X.Y.Z` and contains notes, date, URL, and signature;
- every referenced updater artifact is reachable.

If testing inside the app, retry **Check for Updates** after the manifests are
live. Restart the app only to clear previously displayed status.

## Recover

First inspect without mutation:

```powershell
git status --short
git log -3 --oneline --decorate
git tag -n --list vX.Y.Z
git ls-remote origin refs/heads/main refs/tags/vX.Y.Z refs/tags/vX.Y.Z^{}
gh run list --workflow release.yml --repo PaperPlaneLabs/MyTodos --limit 5
gh release view vX.Y.Z --repo PaperPlaneLabs/MyTodos
```

Apply the matching recovery:

- **Push failed after local commit/tag, remote tag absent:** fix the cause,
  verify the remote branch is an ancestor, stage only intended files, amend the
  unpushed commit if needed, recreate the local annotated tag at the amended
  commit, then push the branch and that exact tag.
- **Remote tag exists:** never move, delete, or force-push it without explicit
  user approval. Prefer repairing assets or rerunning the workflow against the
  existing tag with `workflow_dispatch`.
- **Workflow failed:** inspect the failed job logs. Fix source/tooling on a new
  release when the tag already represents an immutable published build; rerun a
  job only when the commit itself is correct.
- **Release exists but manifest is missing:** inspect
  `upload-updater-jsons`. A running prerequisite means wait; a failed final job
  means repair or rerun that job. Do not edit application updater code merely
  because publication is incomplete.
- **Old repository URL fails:** set `origin` and hardcoded release references to
  `https://github.com/PaperPlaneLabs/MyTodos.git`, then verify with
  `git ls-remote` before pushing.

After any recovery, rerun the bundled verifier and report the commit SHA, tag,
workflow URL, release URL, and updater result.

## Safety rules

- Never rerun `create-release.ps1` blindly after it has created a commit or tag.
- Never use `git push --force`, rewrite a remote release tag, or clobber assets
  without explicit approval and a verified target.
- Never expose signing keys or `.env` values in logs.
- Never treat HTTP 200 alone as valid updater metadata; parse and validate it.
- Never report success while the workflow or manifest upload is still running.
