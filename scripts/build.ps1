# PowerShell script for building MyTodos locally
# Builds for: Windows (native), Linux (Podman)

param(
    [string]$Platform = "all",  # Options: all, windows, linux
    [switch]$Clean = $false,
    [switch]$Publish = $false   # Whether to upload to GitHub Release
)

$ErrorActionPreference = "Stop"
Set-Location "$PSScriptRoot\.."

# Helper functions
function Write-Step($msg) { Write-Host "`n>> $msg" -ForegroundColor Cyan }
function Write-Success($msg) { Write-Host "   [OK] $msg" -ForegroundColor Green }
function Write-Warn($msg) { Write-Host "   [!!] $msg" -ForegroundColor Yellow }
function Write-Fail($msg) { Write-Host "   [ER] $msg" -ForegroundColor Red }

# 1. Build for Windows (native)
function Build-Windows {
    Write-Step "Building for Windows (native)..."
    if (-not (Test-Path "node_modules")) { 
        Write-Step "Installing npm dependencies..."
        npm install 
    }
    
    npm run tauri build
    
    New-Item -ItemType Directory -Force -Path "dist" | Out-Null
    
    $artifacts = Get-ChildItem -Path "src-tauri/target/release/bundle/msi/*.msi", "src-tauri/target/release/bundle/nsis/*-setup.exe" -ErrorAction SilentlyContinue
    if ($artifacts.Count -gt 0) {
        Copy-Item -Path $artifacts.FullName -Destination "dist/" -Force
        Write-Success "Windows artifacts copied to dist/"
    }
    else {
        Write-Fail "Windows build finished but no artifacts found!"
        exit 1
    }
}

# 2. Build for Linux (Podman)
function Build-Linux {
    Write-Step "Building for Linux (Podman)..."
    $PodmanExe = "podman"
    
    # Check podman
    try { 
        & $PodmanExe --version | Out-Null 
    }
    catch { 
        Write-Fail "Podman not found! Skipping Linux build."
        return 
    }

    # Ensure podman machine is running on Windows
    $machine = & $PodmanExe machine list --format json | ConvertFrom-Json
    if ($machine -and -not $machine.Running) {
        Write-Step "Starting Podman machine..."
        & $PodmanExe machine start
    }

    # Build container
    Write-Step "Building Podman container (this may take a few minutes)..."
    & $PodmanExe build -f Dockerfile.linux -t mytodos-builder-linux .
    
    if ($LASTEXITCODE -ne 0) {
        Write-Fail "Podman build failed! Check the error messages above."
        return
    }
    
    # Extract artifacts
    Write-Step "Extracting Linux artifacts..."
    $containerId = & $PodmanExe create mytodos-builder-linux
    if (-not $containerId) {
        Write-Fail "Failed to create container from image."
        return
    }
    New-Item -ItemType Directory -Force -Path "dist" | Out-Null
    & $PodmanExe cp "${containerId}:/output/." dist/
    & $PodmanExe rm $containerId
    Write-Success "Linux artifacts copied to dist/"
}

# 3. Publish to GitHub Release
function Publish-Release {
    Write-Step "Publishing to GitHub Release..."
    
    # Check GH CLI
    try { & gh --version | Out-Null } catch { Write-Fail "GitHub CLI (gh) not found! Cannot publish."; return }
    
    # Check version
    $package = Get-Content package.json | ConvertFrom-Json
    $version = $package.version
    $tag = "v$version"
    
    Write-Step "Releasing version $tag..."
    
    $files = Get-ChildItem -Path "dist" -Include *.msi, *-setup.exe, *.deb, *.AppImage -Recurse | ForEach-Object { $_.FullName }
    
    if ($files.Count -eq 0) {
        Write-Fail "No artifacts found in dist/ to upload."
        return
    }

    # Wait for release to exist (since macOS job might be running)
    Write-Step "Checking if release $tag exists on GitHub..."
    $maxTries = 10
    $tryCount = 0
    while ($tryCount -lt $maxTries) {
        $release = gh release view $tag --repo SujithChristopher/MyTodos-releases 2>&1
        if ($release -match "tag: $tag") {
            Write-Success "Release $tag found."
            break
        }
        Write-Warn "Release $tag not found yet. It might be being created by the macOS worker. Retrying in 30s ($($tryCount+1)/$maxTries)..."
        Start-Sleep -Seconds 30
        $tryCount++
    }

    if ($tryCount -eq $maxTries) {
        Write-Fail "Timed out waiting for release $tag. Please ensure the macOS GitHub Action has started or create it manually."
        return
    }

    Write-Step "Uploading $($files.Count) files..."
    gh release upload $tag $files --repo SujithChristopher/MyTodos-releases --clobber
    Write-Success "All local artifacts uploaded successfully!"
}

# Execution
Write-Host "`n=== MyTodos Local Build System ===" -ForegroundColor Cyan

if ($Clean) {
    Write-Step "Cleaning build artifacts..."
    if (Test-Path "dist") { Remove-Item -Recurse -Force "dist" }
    if (Test-Path "src-tauri/target") { Remove-Item -Recurse -Force "src-tauri/target" }
    Write-Success "Cleanup complete."
}

switch ($Platform.ToLower()) {
    "all" { Build-Windows; Build-Linux }
    "windows" { Build-Windows }
    "linux" { Build-Linux }
}

if ($Publish) {
    Publish-Release
}

Write-Host "`nProcess complete!`n" -ForegroundColor Cyan
