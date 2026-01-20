param(
    [Parameter(Mandatory = $true)]
    [string]$v
)

$ErrorActionPreference = "Stop"

# Helper functions
function Write-Step($msg) { Write-Host "`n>> $msg" -ForegroundColor Cyan }
function Write-Success($msg) { Write-Host "   [OK] $msg" -ForegroundColor Green }
function Write-Fail($msg) { Write-Host "   [ER] $msg" -ForegroundColor Red }

# Validate version format (semantic versioning: X.Y.Z)
if ($v -notmatch '^\d+\.\d+\.\d+$') {
    Write-Fail "Invalid version format. Expected: X.Y.Z (e.g., 0.1.24)"
    exit 1
}

Write-Host "`n=== MyTodos Release Automation ===" -ForegroundColor Magenta
Write-Host "Version: v$v" -ForegroundColor Magenta

# ============================================================
# STEP 1: Update version in config files
# ============================================================
Write-Step "Updating version in config files..."

$tauriConfigPath = "src-tauri\tauri.conf.json"
$packageJsonPath = "package.json"
$cargoTomlPath = "src-tauri\Cargo.toml"

foreach ($file in @($tauriConfigPath, $packageJsonPath, $cargoTomlPath)) {
    if (-not (Test-Path $file)) { Write-Fail "File not found: $file"; exit 1 }
}

# Update tauri.conf.json
$tauriConfig = Get-Content $tauriConfigPath -Raw
$tauriConfig = $tauriConfig -replace '"version"\s*:\s*"[^"]*"', "`"version`": `"$v`""
Set-Content -Path $tauriConfigPath -Value $tauriConfig -NoNewline
Write-Success "Updated $tauriConfigPath"

# Update package.json
$packageJson = Get-Content $packageJsonPath -Raw
$packageJson = $packageJson -replace '"version"\s*:\s*"[^"]*"', "`"version`": `"$v`""
Set-Content -Path $packageJsonPath -Value $packageJson -NoNewline
Write-Success "Updated $packageJsonPath"

# Update Cargo.toml
$cargoToml = Get-Content $cargoTomlPath -Raw
$cargoToml = $cargoToml -replace '(\[package\][^\[]*?version\s*=\s*")[^"]*"', "`${1}$v`""
Set-Content -Path $cargoTomlPath -Value $cargoToml -NoNewline
Write-Success "Updated $cargoTomlPath"

# ============================================================
# STEP 2: Git operations (commit & tag)
# ============================================================
Write-Step "Performing git operations..."

git add -A
if ($LASTEXITCODE -ne 0) { Write-Fail "Failed to stage files"; exit 1 }

git commit -m "chore: bump version to v$v"
if ($LASTEXITCODE -ne 0) { Write-Fail "Failed to create commit"; exit 1 }
Write-Success "Created commit"

git tag -a "v$v" -m "Release v$v"
if ($LASTEXITCODE -ne 0) { Write-Fail "Failed to create tag"; exit 1 }
Write-Success "Created tag v$v"

$currentBranch = git rev-parse --abbrev-ref HEAD
git push origin $currentBranch
if ($LASTEXITCODE -ne 0) { Write-Fail "Failed to push commit"; exit 1 }
Write-Success "Pushed commit to origin/$currentBranch"

git push origin "v$v"
if ($LASTEXITCODE -ne 0) { Write-Fail "Failed to push tag"; exit 1 }
Write-Success "Pushed tag v$v"

# ============================================================
# STEP 3: Clean dist folder
# ============================================================
Write-Step "Cleaning dist folder..."
if (Test-Path "dist") { Remove-Item -Recurse -Force "dist" }
New-Item -ItemType Directory -Force -Path "dist" | Out-Null
Write-Success "dist/ folder cleaned"

# ============================================================
# STEP 4: Build for all platforms
# ============================================================
Write-Step "Building for Windows (native)..."
if (-not (Test-Path "node_modules")) { npm install }
npm run tauri build
if ($LASTEXITCODE -ne 0) { Write-Fail "Windows build failed"; exit 1 }

$winArtifacts = Get-ChildItem -Path "src-tauri/target/release/bundle/msi/*.msi", "src-tauri/target/release/bundle/nsis/*-setup.exe" -ErrorAction SilentlyContinue
if ($winArtifacts.Count -gt 0) {
    Copy-Item -Path $winArtifacts.FullName -Destination "dist/" -Force
    Write-Success "Windows artifacts: $($winArtifacts.Name -join ', ')"
}
else {
    Write-Fail "No Windows artifacts found!"
}

Write-Step "Building for Linux (Podman)..."
$PodmanExe = "podman"

# Ensure podman machine is running
$machine = & $PodmanExe machine list --format json | ConvertFrom-Json
if ($machine -and -not $machine.Running) {
    Write-Step "Starting Podman machine..."
    & $PodmanExe machine start
}

& $PodmanExe build -f Dockerfile.linux -t mytodos-builder-linux .
if ($LASTEXITCODE -ne 0) { Write-Fail "Linux Podman build failed"; exit 1 }

$containerId = & $PodmanExe create mytodos-builder-linux
& $PodmanExe cp "${containerId}:/output/." dist/
& $PodmanExe rm $containerId
Write-Success "Linux artifacts copied to dist/"

# ============================================================
# STEP 5: Wait for macOS release & Upload local artifacts
# ============================================================
Write-Step "Waiting for macOS GitHub Action to create release v$v..."
Write-Host "   (The tag push triggers GitHub Actions to build macOS and create the release)"

$tag = "v$v"
$maxTries = 20  # 20 tries * 30 seconds = 10 minutes max wait
$tryCount = 0

while ($tryCount -lt $maxTries) {
    $release = gh release view $tag --repo SujithChristopher/MyTodos-releases 2>&1
    if ($release -match "title:" -or $release -match "tag:") {
        Write-Success "Release $tag found on GitHub!"
        break
    }
    $tryCount++
    Write-Host "   [..] Release not found yet. Waiting 30s... ($tryCount/$maxTries)"
    Start-Sleep -Seconds 30
}

if ($tryCount -eq $maxTries) {
    Write-Fail "Timed out waiting for release $tag."
    Write-Host "   The macOS GitHub Action may still be running. You can manually upload later with:"
    Write-Host "   gh release upload $tag dist/* --repo SujithChristopher/MyTodos-releases --clobber"
    exit 1
}

# Collect artifacts
$files = Get-ChildItem -Path "dist" -Include *.msi, *-setup.exe, *.deb, *.rpm, *.AppImage -Recurse | ForEach-Object { $_.FullName }

if ($files.Count -eq 0) {
    Write-Fail "No artifacts found in dist/ to upload."
    exit 1
}

Write-Step "Uploading $($files.Count) local artifacts to release $tag..."
$files | ForEach-Object { Write-Host "   - $(Split-Path $_ -Leaf)" }

gh release upload $tag $files --repo SujithChristopher/MyTodos-releases --clobber

if ($LASTEXITCODE -ne 0) { Write-Fail "Failed to upload artifacts"; exit 1 }

Write-Host "`n=== Release v$v Complete! ===" -ForegroundColor Green
Write-Host "View at: https://github.com/SujithChristopher/MyTodos-releases/releases/tag/$tag" -ForegroundColor Yellow
