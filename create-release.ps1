param(
    [Parameter(Mandatory = $true)]
    [string]$v,
    [switch]$Online = $false
)

$ErrorActionPreference = "Stop"

$ReleasesRepo = "SujithChristopher/MyTodos"
$ReleaseBody = "See the assets to download this version and install."
$SupportedPlatforms = @(
    "windows-x86_64",
    "linux-x86_64",
    "darwin-aarch64",
    "darwin-x86_64"
)

function Write-Step($msg) { Write-Host "`n>> $msg" -ForegroundColor Cyan }
function Write-Success($msg) { Write-Host "   [OK] $msg" -ForegroundColor Green }
function Write-Fail($msg) { Write-Host "   [ER] $msg" -ForegroundColor Red }
function Write-Warn($msg) { Write-Host "   [!!] $msg" -ForegroundColor Yellow }

function Load-EnvFile {
    if (-not (Test-Path ".env")) {
        return
    }

    Get-Content ".env" | ForEach-Object {
        if ($_ -match '^\s*([^#=]+)\s*=\s*(.*)$') {
            $key = $matches[1].Trim()
            $value = $matches[2].Trim().Trim("'")
            [Environment]::SetEnvironmentVariable($key, $value, "Process")
        }
    }

    Write-Success "Loaded signing keys from .env"
}

function Get-ManifestFileName([string]$platform) {
    switch ($platform) {
        "windows-x86_64" { return "latest-windows-x86_64.json" }
        "linux-x86_64" { return "latest-linux-x86_64.json" }
        "darwin-aarch64" { return "latest-darwin-aarch64.json" }
        "darwin-x86_64" { return "latest-darwin-x86_64.json" }
        default { throw "Unsupported platform manifest: $platform" }
    }
}

function Get-PlatformFromManifestName([string]$name) {
    switch ($name) {
        "latest-windows-x86_64.json" { return "windows-x86_64" }
        "latest-linux-x86_64.json" { return "linux-x86_64" }
        "latest-darwin-aarch64.json" { return "darwin-aarch64" }
        "latest-darwin-x86_64.json" { return "darwin-x86_64" }
        default { return $null }
    }
}

function Set-PlatformState(
    [hashtable]$states,
    [string]$platform,
    [string]$version,
    [string]$url,
    [string]$signature,
    [string]$notes,
    [string]$pubDate
) {
    $states[$platform] = @{
        version = $version
        notes = $notes
        pub_date = $pubDate
        url = $url
        signature = $signature
    }
}

function Import-LegacyLatestJson([string]$path, [hashtable]$states) {
    if (-not (Test-Path $path)) {
        return
    }

    $legacy = Get-Content $path -Raw | ConvertFrom-Json
    if (-not $legacy.platforms) {
        return
    }

    foreach ($prop in $legacy.platforms.PSObject.Properties) {
        if ($states.ContainsKey($prop.Name)) {
            continue
        }

        Set-PlatformState `
            -states $states `
            -platform $prop.Name `
            -version $legacy.version `
            -url $prop.Value.url `
            -signature $prop.Value.signature `
            -notes $legacy.notes `
            -pubDate $legacy.pub_date
    }
}

function Import-PerPlatformManifests([string]$dir, [hashtable]$states) {
    if (-not (Test-Path $dir)) {
        return
    }

    $manifestFiles = Get-ChildItem -Path $dir -Filter "latest-*.json" -File -ErrorAction SilentlyContinue
    foreach ($manifestFile in $manifestFiles) {
        $platform = Get-PlatformFromManifestName $manifestFile.Name
        if (-not $platform) {
            continue
        }

        $manifest = Get-Content $manifestFile.FullName -Raw | ConvertFrom-Json
        Set-PlatformState `
            -states $states `
            -platform $platform `
            -version $manifest.version `
            -url $manifest.url `
            -signature $manifest.signature `
            -notes $manifest.notes `
            -pubDate $manifest.pub_date
    }
}

function Get-PreservedPlatformStates([string]$tempDir) {
    $states = @{}

    if (Test-Path $tempDir) {
        Remove-Item -Recurse -Force $tempDir
    }

    New-Item -ItemType Directory -Path $tempDir | Out-Null

    try {
        gh release download --repo $ReleasesRepo --pattern "latest*.json" --dir $tempDir --clobber *> $null
    } catch {
        Write-Warn "No existing updater manifests found on the latest release."
    }

    Import-PerPlatformManifests -dir $tempDir -states $states
    Import-LegacyLatestJson -path (Join-Path $tempDir "latest.json") -states $states

    return $states
}

function Write-PlatformManifest([string]$outputDir, [string]$platform, [hashtable]$state) {
    $manifestPath = Join-Path $outputDir (Get-ManifestFileName $platform)
    $manifest = @{
        version = $state.version
        notes = $state.notes
        pub_date = $state.pub_date
        url = $state.url
        signature = $state.signature
    } | ConvertTo-Json -Depth 3

    Set-Content -Path $manifestPath -Value $manifest
    return $manifestPath
}

function Ensure-ReleaseExists([string]$tag) {
    gh release view $tag --repo $ReleasesRepo *> $null
    if ($LASTEXITCODE -eq 0) {
        Write-Success "GitHub release $tag already exists"
        return
    }

    gh release create $tag --repo $ReleasesRepo --title "MyTodos $tag" --notes $ReleaseBody
    if ($LASTEXITCODE -ne 0) {
        Write-Fail "Failed to create release $tag"
        exit 1
    }

    Write-Success "Created GitHub release $tag"
}

Load-EnvFile

if ($v -notmatch '^\d+\.\d+\.\d+$') {
    Write-Fail "Invalid version format. Expected: X.Y.Z (e.g., 0.1.24)"
    exit 1
}

$tag = "v$v"
$releaseUrl = "https://github.com/SujithChristopher/MyTodos/releases/download/$tag"
$preserveDir = Join-Path ([System.IO.Path]::GetTempPath()) "mytodos-release-preserve-$v"

Write-Host "`n=== MyTodos Windows Release Automation ===" -ForegroundColor Magenta
Write-Host "Version: $tag" -ForegroundColor Magenta

$platformStates = @{}
if (-not $Online) {
    Write-Step "Capturing previous updater manifests..."
    $platformStates = Get-PreservedPlatformStates -tempDir $preserveDir
    if ($platformStates.Count -gt 0) {
        Write-Success "Preserved $($platformStates.Count) existing platform manifest(s)"
    } else {
        Write-Warn "No previous per-platform updater state was available to preserve."
    }
}

Write-Step "Updating version in config files..."

$tauriConfigPath = "src-tauri\tauri.conf.json"
$packageJsonPath = "package.json"
$cargoTomlPath = "src-tauri\Cargo.toml"

foreach ($file in @($tauriConfigPath, $packageJsonPath, $cargoTomlPath)) {
    if (-not (Test-Path $file)) {
        Write-Fail "File not found: $file"
        exit 1
    }
}

$tauriConfig = Get-Content $tauriConfigPath -Raw
$tauriConfig = $tauriConfig -replace '"version"\s*:\s*"[^"]*"', "`"version`": `"$v`""
Set-Content -Path $tauriConfigPath -Value $tauriConfig -NoNewline
Write-Success "Updated $tauriConfigPath"

$packageJson = Get-Content $packageJsonPath -Raw
$packageJson = $packageJson -replace '"version"\s*:\s*"[^"]*"', "`"version`": `"$v`""
Set-Content -Path $packageJsonPath -Value $packageJson -NoNewline
Write-Success "Updated $packageJsonPath"

$cargoToml = Get-Content $cargoTomlPath -Raw
$cargoToml = $cargoToml -replace '(\[package\][^\[]*?version\s*=\s*")[^"]*"', "`${1}$v`""
Set-Content -Path $cargoTomlPath -Value $cargoToml -NoNewline
Write-Success "Updated $cargoTomlPath"

Write-Step "Performing git operations..."

git add -A
if ($LASTEXITCODE -ne 0) {
    Write-Fail "Failed to stage files"
    exit 1
}

git commit -m "chore: bump version to $tag"
if ($LASTEXITCODE -ne 0) {
    Write-Fail "Failed to create commit"
    exit 1
}
Write-Success "Created commit"

git tag -a $tag -m "Release $tag"
if ($LASTEXITCODE -ne 0) {
    Write-Fail "Failed to create tag"
    exit 1
}
Write-Success "Created tag $tag"

$currentBranch = git rev-parse --abbrev-ref HEAD
git push origin $currentBranch
if ($LASTEXITCODE -ne 0) {
    Write-Fail "Failed to push commit"
    exit 1
}
Write-Success "Pushed commit to origin/$currentBranch"

git push origin --tags
if ($LASTEXITCODE -ne 0) {
    Write-Fail "Failed to push tags"
    exit 1
}
Write-Success "Pushed tags to origin"

if ($Online) {
    Write-Step "Online mode enabled; skipping local build/sign/upload steps."
    Write-Host "Use these commands to monitor and verify the release:" -ForegroundColor Yellow
    Write-Host "  gh run list --workflow release.yml --repo SujithChristopher/MyTodos --limit 5"
    Write-Host "  gh run watch <run-id> --repo SujithChristopher/MyTodos"
    Write-Host "  gh release view $tag --repo SujithChristopher/MyTodos"
    exit 0
}

Write-Step "Cleaning dist and bundle folders..."
if (Test-Path "dist") {
    Remove-Item -Recurse -Force "dist"
}
New-Item -ItemType Directory -Force -Path "dist" | Out-Null

if (Test-Path "src-tauri/target/release/bundle") {
    Remove-Item -Recurse -Force "src-tauri/target/release/bundle"
}
Write-Success "Cleaned dist/ and bundle/ folders"

Write-Step "Building for Windows (native)..."
if (-not (Test-Path "node_modules")) {
    npm install
    if ($LASTEXITCODE -ne 0) {
        Write-Fail "npm install failed"
        exit 1
    }
}

npm run tauri build
if ($LASTEXITCODE -ne 0) {
    Write-Fail "Windows build failed"
    exit 1
}

$winArtifacts = Get-ChildItem -Path `
    "src-tauri/target/release/bundle/msi/*.msi", `
    "src-tauri/target/release/bundle/msi/*.msi.sig", `
    "src-tauri/target/release/bundle/nsis/*-setup.exe", `
    "src-tauri/target/release/bundle/nsis/*-setup.exe.sig" `
    -ErrorAction SilentlyContinue

if (-not $winArtifacts -or $winArtifacts.Count -eq 0) {
    Write-Fail "No Windows artifacts found."
    exit 1
}

Copy-Item -Path $winArtifacts.FullName -Destination "dist/" -Force
Write-Success "Windows artifacts: $($winArtifacts.Name -join ', ')"

Write-Step "Creating or reusing GitHub release..."
Ensure-ReleaseExists -tag $tag

$uploadFiles = Get-ChildItem -Path "dist" -Include *.msi, *.msi.sig, *-setup.exe, *-setup.exe.sig -Recurse | ForEach-Object { $_.FullName }
Write-Step "Uploading Windows artifacts to $tag..."
gh release upload $tag $uploadFiles --repo $ReleasesRepo --clobber
if ($LASTEXITCODE -ne 0) {
    Write-Fail "Failed to upload Windows artifacts"
    exit 1
}
Write-Success "Uploaded $($uploadFiles.Count) Windows artifact(s)"

Write-Step "Generating updater manifests..."

$nsisExeSig = Get-ChildItem -Path "dist" -Filter "*-setup.exe.sig" -ErrorAction SilentlyContinue | Select-Object -First 1
$nsisExe = Get-ChildItem -Path "dist" -Filter "*-setup.exe" -ErrorAction SilentlyContinue | Where-Object { $_.Name -notlike "*.sig" } | Select-Object -First 1

if (-not $nsisExeSig -or -not $nsisExe) {
    Write-Fail "No Windows NSIS updater artifact found."
    exit 1
}

Set-PlatformState `
    -states $platformStates `
    -platform "windows-x86_64" `
    -version $v `
    -url "$releaseUrl/$($nsisExe.Name)" `
    -signature (Get-Content $nsisExeSig.FullName -Raw).Trim() `
    -notes "MyTodos $tag" `
    -pubDate (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

$manifestFiles = @()
foreach ($platform in $SupportedPlatforms) {
    if (-not $platformStates.ContainsKey($platform)) {
        Write-Warn "No manifest state available for $platform"
        continue
    }

    $manifestFiles += Write-PlatformManifest -outputDir "dist" -platform $platform -state $platformStates[$platform]
    Write-Success "Wrote $(Get-ManifestFileName $platform)"
}

$legacyLatest = @{
    version = $v
    notes = "MyTodos $tag"
    pub_date = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    platforms = @{
        "windows-x86_64" = @{
            signature = $platformStates["windows-x86_64"].signature
            url = $platformStates["windows-x86_64"].url
        }
    }
} | ConvertTo-Json -Depth 4

$legacyLatestPath = "dist/latest.json"
Set-Content -Path $legacyLatestPath -Value $legacyLatest
Write-Warn "Generated legacy latest.json for Windows only. Older macOS/Linux builds that still use the legacy endpoint may report an updater error for this skipped release."

if ($manifestFiles.Count -eq 0) {
    Write-Fail "No updater manifest files were generated."
    exit 1
}

$manifestUploadFiles = @($manifestFiles + $legacyLatestPath)
gh release upload $tag $manifestUploadFiles --repo $ReleasesRepo --clobber
if ($LASTEXITCODE -ne 0) {
    Write-Fail "Failed to upload updater manifest files"
    exit 1
}
Write-Success "Uploaded $($manifestUploadFiles.Count) updater manifest file(s)"

Write-Host "`n=== Release $tag Complete! ===" -ForegroundColor Green
Write-Host "View at: https://github.com/SujithChristopher/MyTodos/releases/tag/$tag" -ForegroundColor Yellow
Write-Host "macOS builds are now manual via GitHub Actions workflow dispatch when you want to publish them." -ForegroundColor Yellow
