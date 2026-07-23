param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^\d+\.\d+\.\d+$')]
    [string]$Version,

    [string]$Repository = "PaperPlaneLabs/MyTodos"
)

$ErrorActionPreference = "Stop"
$tag = "v$Version"
$platforms = @(
    "windows-x86_64",
    "linux-x86_64",
    "darwin-aarch64",
    "darwin-x86_64"
)
$failures = [System.Collections.Generic.List[string]]::new()

function Add-Failure([string]$Message) {
    $failures.Add($Message)
    Write-Host "[FAIL] $Message" -ForegroundColor Red
}

function Write-Ok([string]$Message) {
    Write-Host "[OK]   $Message" -ForegroundColor Green
}

function Invoke-GhJson([string[]]$Arguments) {
    $output = & gh @Arguments 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "gh $($Arguments -join ' ') failed: $($output -join [Environment]::NewLine)"
    }
    return ($output -join [Environment]::NewLine) | ConvertFrom-Json
}

Add-Type -AssemblyName System.Net.Http
$handler = [System.Net.Http.HttpClientHandler]::new()
$handler.AllowAutoRedirect = $true
$client = [System.Net.Http.HttpClient]::new($handler)
$client.Timeout = [TimeSpan]::FromSeconds(20)
$client.DefaultRequestHeaders.UserAgent.ParseAdd("Todoz-release-verifier/1.0")

try {
    $runs = @(Invoke-GhJson @(
        "run", "list",
        "--workflow", "release.yml",
        "--repo", $Repository,
        "--branch", $tag,
        "--limit", "1",
        "--json", "databaseId,status,conclusion,url,headSha"
    ))

    if ($runs.Count -eq 0) {
        Add-Failure "No release workflow run found for $tag"
    } else {
        $run = $runs[0]
        if ($run.status -eq "completed" -and $run.conclusion -eq "success") {
            Write-Ok "Workflow succeeded: $($run.url)"
        } else {
            Add-Failure "Workflow is $($run.status) with conclusion '$($run.conclusion)': $($run.url)"
        }
    }

    try {
        $release = Invoke-GhJson @(
            "release", "view", $tag,
            "--repo", $Repository,
            "--json", "url,isDraft,isPrerelease,tagName,assets"
        )

        if ($release.isDraft -or $release.isPrerelease) {
            Add-Failure "Release is not a final published release: $($release.url)"
        } else {
            Write-Ok "Release is published: $($release.url)"
        }
    } catch {
        Add-Failure $_.Exception.Message
    }

    foreach ($platform in $platforms) {
        $manifestUrl = "https://github.com/$Repository/releases/latest/download/latest-$platform.json"

        try {
            $manifestText = $client.GetStringAsync($manifestUrl).GetAwaiter().GetResult()
            $manifest = $manifestText | ConvertFrom-Json

            if ($manifest.version -ne $Version) {
                Add-Failure "$platform manifest version is '$($manifest.version)', expected '$Version'"
                continue
            }

            foreach ($field in @("notes", "pub_date", "url", "signature")) {
                if ([string]::IsNullOrWhiteSpace([string]$manifest.$field)) {
                    Add-Failure "$platform manifest has no $field"
                }
            }

            if ([string]::IsNullOrWhiteSpace([string]$manifest.url)) {
                continue
            }

            $response = $client.GetAsync(
                [string]$manifest.url,
                [System.Net.Http.HttpCompletionOption]::ResponseHeadersRead
            ).GetAwaiter().GetResult()
            try {
                if (-not $response.IsSuccessStatusCode) {
                    Add-Failure "$platform updater artifact returned HTTP $([int]$response.StatusCode)"
                    continue
                }
            } finally {
                $response.Dispose()
            }

            Write-Ok "$platform manifest and updater artifact are valid"
        } catch {
            Add-Failure "$platform verification failed: $($_.Exception.Message)"
        }
    }
} finally {
    $client.Dispose()
    $handler.Dispose()
}

if ($failures.Count -gt 0) {
    Write-Host "`nRelease verification failed with $($failures.Count) problem(s)." -ForegroundColor Red
    exit 1
}

Write-Host "`nRelease $tag is fully published and updater-ready." -ForegroundColor Green
