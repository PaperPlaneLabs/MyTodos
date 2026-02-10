param(
    [Parameter(Mandatory = $true)]
    [string]$v
)

$ErrorActionPreference = "Stop"

# Helper functions
function Write-Step($msg) { Write-Host "`n>> $msg" -ForegroundColor Cyan }
function Write-Success($msg) { Write-Host "   [OK] $msg" -ForegroundColor Green }
function Write-Fail($msg) { Write-Host "   [ER] $msg" -ForegroundColor Red }

# Load signing keys from .env for auto-update signing
if (Test-Path ".env") {
    Get-Content ".env" | ForEach-Object {
        if ($_ -match '^\s*([^#=]+)\s*=\s*(.*)$') {
            $key = $matches[1].Trim()
            $value = $matches[2].Trim().Trim("'")
            [Environment]::SetEnvironmentVariable($key, $value, "Process")
        }
    }
    Write-Success "Loaded signing keys from .env"
}

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
# STEP 3: Clean build artifacts
# ============================================================
Write-Step "Cleaning dist and bundle folders..."
if (Test-Path "dist") { Remove-Item -Recurse -Force "dist" }
New-Item -ItemType Directory -Force -Path "dist" | Out-Null

# Clean Windows bundle directory to remove old version files
if (Test-Path "src-tauri/target/release/bundle") { 
    Remove-Item -Recurse -Force "src-tauri/target/release/bundle" 
}
Write-Success "Cleaned dist/ and bundle/ folders"


# ============================================================
# STEP 4: Build for all platforms
# ============================================================
Write-Step "Building for Windows (native)..."
if (-not (Test-Path "node_modules")) { npm install }
npm run tauri build
if ($LASTEXITCODE -ne 0) { Write-Fail "Windows build failed"; exit 1 }

$winArtifacts = Get-ChildItem -Path "src-tauri/target/release/bundle/msi/*.msi", "src-tauri/target/release/bundle/msi/*.msi.sig", "src-tauri/target/release/bundle/nsis/*-setup.exe", "src-tauri/target/release/bundle/nsis/*-setup.exe.sig" -ErrorAction SilentlyContinue
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

$signingKey = [Environment]::GetEnvironmentVariable("TAURI_SIGNING_PRIVATE_KEY", "Process")
$signingKeyPassword = [Environment]::GetEnvironmentVariable("TAURI_SIGNING_PRIVATE_KEY_PASSWORD", "Process")

$buildArgs = @()
if ($signingKey) {
    $buildArgs += "--build-arg", "TAURI_SIGNING_PRIVATE_KEY=$signingKey"
    if ($signingKeyPassword) {
        $buildArgs += "--build-arg", "TAURI_SIGNING_PRIVATE_KEY_PASSWORD=$signingKeyPassword"
    }
    Write-Success "Passing signing keys to Linux build"
} else {
    Write-Host "   [!!] TAURI_SIGNING_PRIVATE_KEY not set - Linux .sig files will not be generated" -ForegroundColor Yellow
}

& $PodmanExe build -f Dockerfile.linux -t mytodos-builder-linux @buildArgs .
if ($LASTEXITCODE -ne 0) { Write-Fail "Linux Podman build failed"; exit 1 }

$containerId = & $PodmanExe create mytodos-builder-linux
& $PodmanExe cp "${containerId}:/output/." dist/
& $PodmanExe rm $containerId
Write-Success "Linux artifacts copied to dist/"

# ============================================================
# STEP 5: Commit Cargo.lock if changed
# ============================================================
$cargoLockChanged = git diff --quiet src-tauri/Cargo.lock; $cargoLockChanged = ($LASTEXITCODE -ne 0)
if ($cargoLockChanged) {
    Write-Step "Committing updated Cargo.lock..."
    git add src-tauri/Cargo.lock
    git commit -m "chore: update Cargo.lock for v$v"
    git push origin (git rev-parse --abbrev-ref HEAD)
    Write-Success "Cargo.lock committed and pushed"
}
else {
    Write-Success "Cargo.lock unchanged, no commit needed"
}

# ============================================================
# STEP 6: Wait for macOS release & Upload local artifacts
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
$files = Get-ChildItem -Path "dist" -Include *.msi, *.msi.sig, *-setup.exe, *-setup.exe.sig, *.deb, *.deb.sig, *.rpm, *.rpm.sig, *.AppImage, *.AppImage.sig -Recurse | ForEach-Object { $_.FullName }

if ($files.Count -eq 0) {
    Write-Fail "No artifacts found in dist/ to upload."
    exit 1
}

Write-Step "Uploading $($files.Count) local artifacts to release $tag..."
$files | ForEach-Object { Write-Host "   - $(Split-Path $_ -Leaf)" }

gh release upload $tag $files --repo SujithChristopher/MyTodos-releases --clobber

if ($LASTEXITCODE -ne 0) { Write-Fail "Failed to upload artifacts"; exit 1 }

# ============================================================
# STEP 7: Generate and upload latest.json for auto-updater
# ============================================================
Write-Step "Generating latest.json for auto-updater..."

$releaseUrl = "https://github.com/SujithChristopher/MyTodos-releases/releases/download/$tag"
$platforms = @{}

# Try to download existing latest.json from CI (may have macOS entries)
try {
    $existingJson = gh release download $tag --pattern "latest.json" --repo SujithChristopher/MyTodos-releases --dir dist --clobber 2>&1
    $existingLatest = Get-Content "dist/latest.json" -Raw | ConvertFrom-Json
    if ($existingLatest.platforms) {
        foreach ($prop in $existingLatest.platforms.PSObject.Properties) {
            $platforms[$prop.Name] = $prop.Value
            Write-Success "Kept existing platform: $($prop.Name)"
        }
    }
} catch {
    Write-Host "   [..] No existing latest.json from CI - creating from scratch" -ForegroundColor Yellow
}

# Windows x86_64 (NSIS exe)
$nsisExeSig = Get-ChildItem -Path "dist" -Filter "*-setup.exe.sig" -ErrorAction SilentlyContinue | Select-Object -First 1
$nsisExe = Get-ChildItem -Path "dist" -Filter "*-setup.exe" -ErrorAction SilentlyContinue | Where-Object { $_.Name -notlike "*.sig" } | Select-Object -First 1
if ($nsisExeSig -and $nsisExe) {
    $platforms["windows-x86_64"] = @{
        signature = (Get-Content $nsisExeSig.FullName -Raw).Trim()
        url = "$releaseUrl/$($nsisExe.Name)"
    }
    Write-Success "Added windows-x86_64 (NSIS)"
} else {
    Write-Host "   [!!] No Windows NSIS .sig found - skipping windows-x86_64" -ForegroundColor Yellow
}

# Linux x86_64 (AppImage)
$appImageSig = Get-ChildItem -Path "dist" -Filter "*.AppImage.sig" -ErrorAction SilentlyContinue | Select-Object -First 1
$appImage = Get-ChildItem -Path "dist" -Filter "*.AppImage" -ErrorAction SilentlyContinue | Where-Object { $_.Name -notlike "*.sig" } | Select-Object -First 1
if ($appImageSig -and $appImage) {
    $platforms["linux-x86_64"] = @{
        signature = (Get-Content $appImageSig.FullName -Raw).Trim()
        url = "$releaseUrl/$($appImage.Name)"
    }
    Write-Success "Added linux-x86_64 (AppImage)"
} else {
    Write-Host "   [!!] No Linux AppImage .sig found - skipping linux-x86_64" -ForegroundColor Yellow
}

# macOS - download sigs from release if not already in platforms
$macTargets = @(
    @{ platform = "darwin-aarch64"; artifact = "my-todos_aarch64.app.tar.gz" },
    @{ platform = "darwin-x86_64"; artifact = "my-todos_x64.app.tar.gz" }
)

foreach ($target in $macTargets) {
    if (-not $platforms.ContainsKey($target.platform)) {
        $sigFile = "$($target.artifact).sig"
        try {
            gh release download $tag --pattern $sigFile --repo SujithChristopher/MyTodos-releases --dir dist --clobber 2>$null
            if (Test-Path "dist/$sigFile") {
                $platforms[$target.platform] = @{
                    signature = (Get-Content "dist/$sigFile" -Raw).Trim()
                    url = "$releaseUrl/$($target.artifact)"
                }
                Write-Success "Added $($target.platform) (from release)"
            }
        } catch {
            Write-Host "   [!!] No sig for $($target.platform) - skipping" -ForegroundColor Yellow
        }
    }
}

if ($platforms.Count -eq 0) {
    Write-Fail "No platform signatures found! Cannot generate latest.json."
    Write-Host "   Make sure TAURI_SIGNING_PRIVATE_KEY is set in .env"
    Write-Host "   The auto-updater will NOT work without latest.json"
} else {
    $latestJson = @{
        version = $v
        notes = "MyTodos v$v"
        pub_date = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
        platforms = $platforms
    } | ConvertTo-Json -Depth 3

    $latestJsonPath = "dist/latest.json"
    Set-Content -Path $latestJsonPath -Value $latestJson
    Write-Success "Generated latest.json with $($platforms.Count) platform(s)"

    gh release upload $tag $latestJsonPath --repo SujithChristopher/MyTodos-releases --clobber
    if ($LASTEXITCODE -ne 0) {
        Write-Fail "Failed to upload latest.json"
    } else {
        Write-Success "Uploaded latest.json to release $tag"
    }
}

Write-Host "`n=== Release v$v Complete! ===" -ForegroundColor Green
Write-Host "View at: https://github.com/SujithChristopher/MyTodos-releases/releases/tag/$tag" -ForegroundColor Yellow
