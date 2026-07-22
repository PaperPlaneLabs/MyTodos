$script:DefaultMyTodosReleaseNotesPath = Join-Path `
    (Split-Path -Parent $PSScriptRoot) `
    "src\lib\data\releases.json"

function Get-MyTodosReleaseEntry {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Version,
        [string]$Path = $script:DefaultMyTodosReleaseNotesPath
    )

    if (-not (Test-Path $Path)) {
        throw "Release notes catalog not found: $Path"
    }

    try {
        $catalog = Get-Content $Path -Raw | ConvertFrom-Json
    } catch {
        throw "Release notes catalog is not valid JSON: $($_.Exception.Message)"
    }

    $matches = @($catalog | Where-Object { $_.version -eq $Version })
    if ($matches.Count -ne 1) {
        throw "Expected exactly one release-notes entry for version $Version; found $($matches.Count). Add it to $Path before releasing."
    }

    $entry = $matches[0]
    if ([string]::IsNullOrWhiteSpace($entry.title) -or [string]::IsNullOrWhiteSpace($entry.summary)) {
        throw "Release $Version must include a title and summary."
    }
    if (@($entry.highlights).Count -eq 0 -and @($entry.fixes).Count -eq 0) {
        throw "Release $Version must include at least one highlight or fix."
    }

    return $entry
}

function Format-MyTodosReleaseNotes {
    param(
        [Parameter(Mandatory = $true)]
        $Entry
    )

    $lines = [System.Collections.Generic.List[string]]::new()
    $lines.Add("## $($Entry.title)")
    $lines.Add("")
    $lines.Add([string]$Entry.summary)

    if (@($Entry.highlights).Count -gt 0) {
        $lines.Add("")
        $lines.Add("### Highlights")
        foreach ($highlight in @($Entry.highlights)) {
            $lines.Add("- $highlight")
        }
    }

    if (@($Entry.fixes).Count -gt 0) {
        $lines.Add("")
        $lines.Add("### Fixes")
        foreach ($fix in @($Entry.fixes)) {
            $lines.Add("- $fix")
        }
    }

    return ($lines -join [Environment]::NewLine)
}
