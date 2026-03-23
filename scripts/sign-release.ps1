param(
    [Parameter(Mandatory = $true)]
    [string]$v,
    [switch]$Upload = $false,
    [switch]$Force = $false
)

$ErrorActionPreference = "Stop"

$ReleasesRepo = "SujithChristopher/MyTodos-releases"

function Write-Step($msg) { Write-Host "`n>> $msg" -ForegroundColor Cyan }
function Write-Success($msg) { Write-Host "   [OK] $msg" -ForegroundColor Green }
function Write-Fail($msg) { Write-Host "   [ER] $msg" -ForegroundColor Red }
function Write-Warn($msg) { Write-Host "   [!!] $msg" -ForegroundColor Yellow }

function Load-EnvFile {
    if (-not (Test-Path ".env")) {
        Write-Warn ".env file not found. Ensure TAURI_SIGNING_PRIVATE_KEY is set."
        return
    }

    Get-Content ".env" | ForEach-Object {
        if ($_ -match '^\s*([^#=]+)\s*=\s*(.*)$') {
            $key = $matches[1].Trim()
            $value = $matches[2].Trim().Trim("'")
            [Environment]::SetEnvironmentVariable($key, $value, "Process")
        }
    }

    Write-Success "Loaded keys from .env"
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

Load-EnvFile

if (-not [Environment]::GetEnvironmentVariable("TAURI_SIGNING_PRIVATE_KEY", "Process")) {
    Write-Fail "TAURI_SIGNING_PRIVATE_KEY not set. Cannot sign artifacts."
    exit 1
}

if (-not (Test-Path "dist")) {
    Write-Fail "dist/ folder not found. Please run build or download artifacts first."
    exit 1
}

$artifacts = Get-ChildItem -Path "dist" -Include *.msi, *-setup.exe, *.AppImage, *.app.tar.gz, *.deb, *.rpm -Recurse
if ($artifacts.Count -eq 0) {
    Write-Fail "No artifacts found in dist/ to sign."
    exit 1
}

Write-Step "Signing $($artifacts.Count) artifact(s)..."

foreach ($file in $artifacts) {
    $sigPath = "$($file.FullName).sig"
    if ((Test-Path $sigPath) -and (-not $Force)) {
        Write-Host "   Skipping $($file.Name) (signature exists)" -ForegroundColor Gray
        continue
    }

    Write-Host "   Signing $($file.Name)..." -NoNewline

    try {
        $tauriPath = ".\node_modules\.bin\tauri.cmd"
        if (-not (Test-Path $tauriPath)) {
            $tauriPath = "tauri"
        }

        $output = & $tauriPath signer sign $file.FullName 2>&1 | Out-String
        if ($LASTEXITCODE -ne 0) {
            Write-Host " [FAILED]" -ForegroundColor Red
            Write-Error "Signing failed: $output"
        }

        if ($output -match "Public signature:") {
            $parts = $output -split "Public signature:"
            $sigPart = $parts[1]
            if ($sigPart -match "Make sure to include") {
                $subParts = $sigPart -split "Make sure to include"
                $signature = $subParts[0].Trim()
            } else {
                $signature = $sigPart.Trim()
            }
        } else {
            $signature = $output.Trim()
        }

        Set-Content -Path $sigPath -Value $signature -NoNewline
        Write-Host " [OK]" -ForegroundColor Green
    } catch {
        Write-Host " [ERROR]" -ForegroundColor Red
        Write-Warn $_
    }
}

Write-Step "Generating updater manifests..."

$tag = "v$v"
$releaseUrl = "https://github.com/SujithChristopher/MyTodos-releases/releases/download/$tag"
$platformStates = @{}

Import-PerPlatformManifests -dir "dist" -states $platformStates
Import-LegacyLatestJson -path "dist/latest.json" -states $platformStates

$notes = "MyTodos $tag"
$pubDate = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

$nsisExeSig = Get-ChildItem -Path "dist" -Filter "*-setup.exe.sig" -ErrorAction SilentlyContinue | Select-Object -First 1
$nsisExe = Get-ChildItem -Path "dist" -Filter "*-setup.exe" -ErrorAction SilentlyContinue | Where-Object { $_.Name -notlike "*.sig" } | Select-Object -First 1
if ($nsisExeSig -and $nsisExe) {
    Set-PlatformState `
        -states $platformStates `
        -platform "windows-x86_64" `
        -version $v `
        -url "$releaseUrl/$($nsisExe.Name)" `
        -signature (Get-Content $nsisExeSig.FullName -Raw).Trim() `
        -notes $notes `
        -pubDate $pubDate
    Write-Success "Added windows-x86_64"
}

$appImageSig = Get-ChildItem -Path "dist" -Filter "*.AppImage.sig" -ErrorAction SilentlyContinue | Select-Object -First 1
$appImage = Get-ChildItem -Path "dist" -Filter "*.AppImage" -ErrorAction SilentlyContinue | Where-Object { $_.Name -notlike "*.sig" } | Select-Object -First 1
if ($appImageSig -and $appImage) {
    Set-PlatformState `
        -states $platformStates `
        -platform "linux-x86_64" `
        -version $v `
        -url "$releaseUrl/$($appImage.Name)" `
        -signature (Get-Content $appImageSig.FullName -Raw).Trim() `
        -notes $notes `
        -pubDate $pubDate
    Write-Success "Added linux-x86_64"
}

$macArtifacts = Get-ChildItem -Path "dist" -Filter "*.app.tar.gz" -ErrorAction SilentlyContinue | Where-Object { $_.Name -notlike "*.sig" }
foreach ($macApp in $macArtifacts) {
    $sigPath = "$($macApp.FullName).sig"
    if (-not (Test-Path $sigPath)) {
        continue
    }

    $platform = "darwin-x86_64"
    if ($macApp.Name -match "aarch64") {
        $platform = "darwin-aarch64"
    }

    Set-PlatformState `
        -states $platformStates `
        -platform $platform `
        -version $v `
        -url "$releaseUrl/$($macApp.Name)" `
        -signature (Get-Content $sigPath -Raw).Trim() `
        -notes $notes `
        -pubDate $pubDate
    Write-Success "Added $platform"
}

$manifestFiles = @()
foreach ($platform in @("windows-x86_64", "linux-x86_64", "darwin-aarch64", "darwin-x86_64")) {
    if (-not $platformStates.ContainsKey($platform)) {
        continue
    }

    $manifestFiles += Write-PlatformManifest -outputDir "dist" -platform $platform -state $platformStates[$platform]
}

if ($platformStates.ContainsKey("windows-x86_64")) {
    $legacyLatest = @{
        version = $platformStates["windows-x86_64"].version
        notes = $platformStates["windows-x86_64"].notes
        pub_date = $platformStates["windows-x86_64"].pub_date
        platforms = @{
            "windows-x86_64" = @{
                signature = $platformStates["windows-x86_64"].signature
                url = $platformStates["windows-x86_64"].url
            }
        }
    } | ConvertTo-Json -Depth 4

    Set-Content -Path "dist/latest.json" -Value $legacyLatest
    Write-Warn "Updated legacy latest.json for Windows only."
}

if ($manifestFiles.Count -gt 0) {
    Write-Success "Generated $($manifestFiles.Count) per-platform manifest file(s)"
} else {
    Write-Warn "No per-platform manifests were generated"
}

if ($Upload) {
    Write-Step "Uploading signatures and updater manifests to GitHub..."

    $filesToUpload = Get-ChildItem -Path "dist" -Include *.sig, latest.json, latest-*.json -Recurse | ForEach-Object { $_.FullName }
    if ($filesToUpload.Count -gt 0) {
        gh release upload $tag $filesToUpload --repo $ReleasesRepo --clobber
        if ($LASTEXITCODE -ne 0) {
            Write-Fail "Upload failed"
            exit 1
        }
        Write-Success "Uploaded $($filesToUpload.Count) file(s)."
    } else {
        Write-Warn "Nothing to upload."
    }
} else {
    Write-Host "`nRun with -Upload to publish to GitHub." -ForegroundColor Yellow
}

Write-Host "`nDone." -ForegroundColor Cyan
