# PowerShell script to setup local build environment for MyTodos
# Checks for: Node.js, Rust, Podman, GitHub CLI

$ErrorActionPreference = "Continue"

function Write-Step($msg) { Write-Host "`n>> $msg" -ForegroundColor Cyan }
function Write-Done($msg) { Write-Host "   [OK] $msg" -ForegroundColor Green }
function Write-Warn($msg) { Write-Host "   [!!] $msg" -ForegroundColor Yellow }
function Write-Fail($msg) { Write-Host "   [ER] $msg" -ForegroundColor Red }

Write-Step "Checking Build Dependencies..."

# 1. Node.js
try {
    $node = node -v
    Write-Done "Node.js found: $node"
}
catch {
    Write-Fail "Node.js not found. Please install from https://nodejs.org/"
}

# 2. Rust
try {
    $rust = rustc --version
    Write-Done "Rust found: $rust"
}
catch {
    Write-Fail "Rust not found. Please install from https://rustup.rs/"
}

# 3. Podman (for Linux builds)
try {
    $podman = podman --version
    Write-Done "Podman found: $podman"
    
    # Check if podman machine is running
    $machine = podman machine list --format json | ConvertFrom-Json
    if ($machine.Running -eq $true) {
        Write-Done "Podman machine is running."
    }
    else {
        Write-Warn "Podman machine is NOT running. Run 'podman machine start' before building."
    }
}
catch {
    Write-Fail "Podman not found. Required for local Linux builds. Install from https://podman.io/"
}

# 4. GitHub CLI (for publishing)
try {
    $gh = gh --version
    Write-Done "GitHub CLI found: $(($gh -split '\n')[0])"
    
    # Check auth status
    $auth = gh auth status 2>&1
    if ($auth -match "Logged in to github.com") {
        Write-Done "GitHub CLI is authenticated."
    }
    else {
        Write-Warn "GitHub CLI is NOT authenticated. Run 'gh auth login'."
    }
}
catch {
    Write-Fail "GitHub CLI not found. Required for publishing releases. Install from https://cli.github.com/"
}

# 5. Tauri CLI
Write-Step "Checking Tauri CLI..."
if (Get-Command tauri -ErrorAction SilentlyContinue) {
    Write-Done "Tauri CLI (global) found."
}
else {
    Write-Warn "Tauri CLI not found globally. We will use 'npm run tauri' instead."
}

Write-Host "`nEnvironment setup check complete!`n" -ForegroundColor Cyan
