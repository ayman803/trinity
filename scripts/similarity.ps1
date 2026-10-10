# Move-similarity test (release checklist): how often Trinity picks the
# same move as other engines in the same positions. Downloads the latest
# Stockfish and Reckless for Windows; also uses the calibration opponents
# saved in sprt\opponents. Double-click 13-Similarity-Test.bat (about 20 minutes).

param(
    [int]$Positions = 1000,
    [int]$MoveTime = 100
)

. (Join-Path $PSScriptRoot "common.ps1")

# Download the newest Windows build of a GitHub project; returns the .exe path.
function Get-LatestEngine([string]$Repo, [string]$Name, [string]$Pattern) {
    $dir = Join-Path (Join-Path $script:Work "similarity") $Name
    $found = @(Get-ChildItem -LiteralPath $dir -Recurse -Filter "*.exe" -ErrorAction SilentlyContinue)
    if ($found.Count -gt 0) { return $found[0].FullName }
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -UseBasicParsing
    $asset = @($release.assets | Where-Object { $_.name -match $Pattern }) | Select-Object -First 1
    if (-not $asset) { Write-Warn "No Windows download found for $Name; skipping it."; return $null }
    $file = Join-Path $dir $asset.name
    Save-Download $asset.browser_download_url $file
    if ($file -like "*.zip") { Expand-Archive -LiteralPath $file -DestinationPath $dir -Force }
    $found = @(Get-ChildItem -LiteralPath $dir -Recurse -Filter "*.exe")
    if ($found.Count -eq 0) { Write-Warn "No .exe inside the $Name download; skipping it."; return $null }
    return $found[0].FullName
}

try {
    Assert-Tools
    Invoke-Housekeeping
    Update-Repository
    $trinity = Build-Trinity "origin/main"
    $engines = @("Trinity=$trinity")
    $sf = Get-LatestEngine "official-stockfish/Stockfish" "Stockfish" "windows-x86-64-avx2\.zip$"
    if ($sf) { $engines += "Stockfish=$sf" }
    $rk = Get-LatestEngine "codedeliveryservice/Reckless" "Reckless" "(?i)windows.*\.(exe|zip)$"
    if ($rk) { $engines += "Reckless=$rk" }
    $opp = Join-Path $script:Work "opponents"
    foreach ($o in @(@("akimbo", "akimbo-1.0.0.exe"), @("Patricia", "Patricia.exe"), @("Bread", "Bread.exe"), @("Prune", "Prune.exe"))) {
        $f = Join-Path $opp $o[1]
        if (Test-Path -LiteralPath $f) { $engines += "$($o[0])=$f" }
    }
    Write-Step "Move-similarity test: $($engines.Count) engines, $Positions positions, $MoveTime ms per move"
    $results = Join-Path $script:Work "results"
    New-Item -ItemType Directory -Force -Path $results | Out-Null
    $out = Join-Path $results "LATEST-SIMILARITY.txt"
    $old = $ErrorActionPreference; $ErrorActionPreference = "Continue"
    & $trinity similarity "$Positions" "$MoveTime" @engines | Tee-Object -FilePath $out
    $ErrorActionPreference = $old
    Write-Host ""
    Write-Good "Done. Copy the table above (also saved in sprt\results\LATEST-SIMILARITY.txt) and paste it to Claude."
} catch {
    Write-UnexpectedError $_
    exit 1
}
