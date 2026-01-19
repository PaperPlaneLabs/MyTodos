param(
    [Parameter(Mandatory=$true)]
    [string]$v
)

# Validate version format (semantic versioning: X.Y.Z)
if ($v -notmatch '^\d+\.\d+\.\d+$') {
    Write-Error "Invalid version format. Expected semantic versioning format: X.Y.Z (e.g., 0.1.1)"
    exit 1
}

Write-Host "Setting version to $v" -ForegroundColor Green

# Define file paths
$tauriConfigPath = "src-tauri\tauri.conf.json"
$packageJsonPath = "package.json"
$cargoTomlPath = "src-tauri\Cargo.toml"

# Check if files exist
$files = @($tauriConfigPath, $packageJsonPath, $cargoTomlPath)
foreach ($file in $files) {
    if (-not (Test-Path $file)) {
        Write-Error "File not found: $file"
        exit 1
    }
}

Write-Host "`nUpdating version in files..." -ForegroundColor Cyan

# Update tauri.conf.json
Write-Host "  - $tauriConfigPath"
$tauriConfig = Get-Content $tauriConfigPath -Raw
$tauriConfig = $tauriConfig -replace '"version"\s*:\s*"[^"]*"', "`"version`": `"$v`""
Set-Content -Path $tauriConfigPath -Value $tauriConfig -NoNewline

# Update package.json
Write-Host "  - $packageJsonPath"
$packageJson = Get-Content $packageJsonPath -Raw
$packageJson = $packageJson -replace '"version"\s*:\s*"[^"]*"', "`"version`": `"$v`""
Set-Content -Path $packageJsonPath -Value $packageJson -NoNewline

# Update Cargo.toml (only package version, not dependencies)
Write-Host "  - $cargoTomlPath"
$cargoToml = Get-Content $cargoTomlPath -Raw
$cargoToml = $cargoToml -replace '(\[package\][^\[]*?version\s*=\s*")[^"]*"', "`${1}$v`""
Set-Content -Path $cargoTomlPath -Value $cargoToml -NoNewline

Write-Host "`nFiles updated successfully!" -ForegroundColor Green

# Git operations
Write-Host "`nPerforming git operations..." -ForegroundColor Cyan

# Stage all files
Write-Host "  - Staging all files..."
git add -A
if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to stage files"
    exit 1
}

# Commit changes
$commitMessage = "chore: bump version to v$v"
Write-Host "  - Creating commit: $commitMessage"
git commit -m $commitMessage
if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to create commit"
    exit 1
}

# Create annotated tag
$tagMessage = "Release v$v"
Write-Host "  - Creating tag: v$v"
git tag -a "v$v" -m $tagMessage
if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to create tag"
    exit 1
}

# Get current branch
$currentBranch = git rev-parse --abbrev-ref HEAD
if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to get current branch"
    exit 1
}

# Push commit
Write-Host "  - Pushing commit to origin/$currentBranch..."
git push origin $currentBranch
if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to push commit"
    exit 1
}

# Push tag
Write-Host "  - Pushing tag v$v to origin..."
git push origin "v$v"
if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to push tag"
    exit 1
}

Write-Host "`nRelease v$v created successfully!" -ForegroundColor Green
Write-Host "`nThe GitHub Actions workflow will now build and publish the release." -ForegroundColor Yellow
Write-Host "Monitor progress at: https://github.com/SujithChristopher/MyTodos/actions" -ForegroundColor Yellow
