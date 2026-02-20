param(
    [Parameter(Mandatory = $true)]
    [string]$v,
    [switch]$Upload = $false,
    [switch]$Force = $false
)

$ErrorActionPreference = "Stop"

function Write-Step($msg) { Write-Host "`n>> $msg" -ForegroundColor Cyan }
function Write-Success($msg) { Write-Host "   [OK] $msg" -ForegroundColor Green }
function Write-Fail($msg) { Write-Host "   [ER] $msg" -ForegroundColor Red }
function Write-Warn($msg) { Write-Host "   [!!] $msg" -ForegroundColor Yellow }

# 1. Load signing keys from .env
if (Test-Path ".env") {
    Get-Content ".env" | ForEach-Object {
        if ($_ -match '^\s*([^#=]+)\s*=\s*(.*)$') {
            $key = $matches[1].Trim()
            $value = $matches[2].Trim().Trim("'")
            [Environment]::SetEnvironmentVariable($key, $value, "Process")
        }
    }
    Write-Success "Loaded keys from .env"
}
else {
    Write-Warn ".env file not found. Ensure TAURI_SIGNING_PRIVATE_KEY is set."
}

if (-not [Environment]::GetEnvironmentVariable("TAURI_SIGNING_PRIVATE_KEY", "Process")) {
    Write-Fail "TAURI_SIGNING_PRIVATE_KEY not set. Cannot sign artifacts."
    exit 1
}

# 2. Identify artifacts in dist/
if (-not (Test-Path "dist")) {
    Write-Fail "dist/ folder not found. Please run build or download artifacts first."
    exit 1
}

$artifacts = Get-ChildItem -Path "dist" -Include *.msi, *-setup.exe, *.AppImage, *.app.tar.gz, *.deb, *.rpm -Recurse

if ($artifacts.Count -eq 0) {
    Write-Fail "No artifacts found in dist/ to sign."
    exit 1
}

Write-Step "Signing $($artifacts.Count) artifacts..."

foreach ($file in $artifacts) {
    $sigPath = "$($file.FullName).sig"
    if ((Test-Path $sigPath) -and (-not $Force)) {
        Write-Host "   Skipping $($file.Name) (signature exists)" -ForegroundColor Gray
        continue
    }

    Write-Host "   Signing $($file.Name)..." -NoNewline
    
    try {
        # Run signature command
        $tauriPath = ".\node_modules\.bin\tauri.cmd"
        if (-not (Test-Path $tauriPath)) { $tauriPath = "tauri" } # Fallback to global
        
        $output = & $tauriPath signer sign $file.FullName 2>&1 | Out-String
        
        if ($LASTEXITCODE -ne 0) {
            Write-Host " [FAILED]" -ForegroundColor Red
            Write-Error "Signing failed: $output"
        }
        
        # Clean up signature
        # Output format: "Your file was signed successfully... Public signature: <BASE64>... Make sure to include..."
        if ($output -match "Public signature:") {
            $parts = $output -split "Public signature:"
            # Take everything after "Public signature:"
            $sigPart = $parts[1]
             
            # The signature is likely followed by "Make sure to include" or newline
            if ($sigPart -match "Make sure to include") {
                $subParts = $sigPart -split "Make sure to include"
                $signature = $subParts[0].Trim()
            }
            else {
                $signature = $sigPart.Trim()
            }
        }
        else {
            # Fallback if no text matched (maybe direct output?)
            $signature = $output.Trim()
        }

        Set-Content -Path $sigPath -Value $signature -NoNewline
        Write-Host " [OK]" -ForegroundColor Green
    }
    catch {
        Write-Host " [ERROR]" -ForegroundColor Red
        Write-Warn $_
    }
}

# 3. Generate latest.json
Write-Step "Generating latest.json..."

$tag = "v$v"
$releaseUrl = "https://github.com/SujithChristopher/MyTodos-releases/releases/download/$tag"
$platforms = @{}

# Load existing latest.json to preserve other platforms
if (Test-Path "dist/latest.json") {
    try {
        $existing = Get-Content "dist/latest.json" -Raw | ConvertFrom-Json
        if ($existing.platforms) {
            foreach ($prop in $existing.platforms.PSObject.Properties) {
                # Preserve existing platform ONLY if we don't have a local update for it
                if (-not $platforms.ContainsKey($prop.Name)) {
                    $platforms[$prop.Name] = $prop.Value
                }
            }
        }
    }
    catch { Write-Warn "Could not read existing latest.json" }
}

# Windows
$nsisExeSig = Get-ChildItem -Path "dist" -Filter "*-setup.exe.sig" | Select-Object -First 1
$nsisExe = Get-ChildItem -Path "dist" -Filter "*-setup.exe" | Where-Object { $_.Name -notlike "*.sig" } | Select-Object -First 1
if ($nsisExeSig -and $nsisExe) {
    $platforms["windows-x86_64"] = @{
        signature = (Get-Content $nsisExeSig.FullName -Raw).Trim()
        url       = "$releaseUrl/$($nsisExe.Name)"
    }
    Write-Success "Added windows-x86_64"
}

# Linux
$appImageSig = Get-ChildItem -Path "dist" -Filter "*.AppImage.sig" | Select-Object -First 1
$appImage = Get-ChildItem -Path "dist" -Filter "*.AppImage" | Where-Object { $_.Name -notlike "*.sig" } | Select-Object -First 1
if ($appImageSig -and $appImage) {
    $platforms["linux-x86_64"] = @{
        signature = (Get-Content $appImageSig.FullName -Raw).Trim()
        url       = "$releaseUrl/$($appImage.Name)"
    }
    Write-Success "Added linux-x86_64"
}

# macOS (local check)
$macAppSig = Get-ChildItem -Path "dist" -Filter "*.app.tar.gz.sig" | Select-Object -First 1
$macApp = Get-ChildItem -Path "dist" -Filter "*.app.tar.gz" | Where-Object { $_.Name -notlike "*.sig" } | Select-Object -First 1
if ($macAppSig -and $macApp) {
    $arch = "darwin-x86_64"
    if ($macApp.Name -match "aarch64") { $arch = "darwin-aarch64" }
    
    $platforms[$arch] = @{
        signature = (Get-Content $macAppSig.FullName -Raw).Trim()
        url       = "$releaseUrl/$($macApp.Name)"
    }
    Write-Success "Added $arch"
}

if ($platforms.Count -gt 0) {
    $latestJson = @{
        version   = $v
        notes     = "MyTodos v$v"
        pub_date  = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
        platforms = $platforms
    } | ConvertTo-Json -Depth 3

    Set-Content -Path "dist/latest.json" -Value $latestJson
    Write-Success "Generated latest.json with $($platforms.Count) platform(s)"
}
else {
    Write-Warn "No platforms added to latest.json"
}

# 4. Upload
if ($Upload) {
    Write-Step "Uploading signatures and latest.json to GitHub..."
    
    $filesToUpload = Get-ChildItem -Path "dist" -Include *.sig, latest.json -Recurse | ForEach-Object { $_.FullName }
    
    if ($filesToUpload.Count -gt 0) {
        # Use --clobber to overwrite existing invalid signatures
        gh release upload $tag $filesToUpload --repo SujithChristopher/MyTodos-releases --clobber
        Write-Success "Uploaded $($filesToUpload.Count) files."
    }
    else {
        Write-Warn "Nothing to upload."
    }
}
else {
    Write-Host "`nRun with -Upload to publish to GitHub." -ForegroundColor Yellow
}

Write-Host "`nDone." -ForegroundColor Cyan
