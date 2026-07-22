# Releasing MyTodos

MyTodos uses `src/lib/data/releases.json` as the single source of truth for
release notes. The same entry is bundled into the app, displayed by the updater,
shown after relaunch, and published as the GitHub release body.

## Prepare a release

1. Add exactly one entry for the new `X.Y.Z` version to
   `src/lib/data/releases.json`.
2. Include a title, summary, and at least one highlight or fix.
3. Validate without changing files or contacting GitHub:

   ```powershell
   .\create-release.ps1 -v 0.1.65 -ValidateOnly
   ```

4. Create the release using the normal local path:

   ```powershell
   .\create-release.ps1 -v 0.1.65
   ```

   Or push the version/tag and let GitHub Actions build all platforms:

   ```powershell
   .\create-release.ps1 -v 0.1.65 -Online
   ```

The release script refuses to continue if the entry is missing or incomplete.
It updates `package.json`, both package-lock version fields,
`src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, and the root `my-todos`
package entry in `src-tauri/Cargo.lock` before committing and
tagging.

## Other release paths

`scripts/sign-release.ps1` reads the same catalog and refuses to sign or publish
an unnoted version. GitHub Actions also validates the tag against all version
files before starting platform builds.

The canonical release repository is `PaperPlaneLabs/MyTodos`; the desktop
updater downloads platform manifests from that repository.
