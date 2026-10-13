# Loss forensics: why does Trinity lose? Takes the lost games of the latest
# calibration (sprt\results), finds each loss's turning point and searches
# it again with ten times the time (see src/forensics.rs). Counts search,
# evaluation and time losses. Double-click 16-Loss-Forensics.bat (about
# 15-30 minutes on the PC).

param(
    [int]$Losses = 200,           # how many lost games to examine (an even sample)
    [int]$Workers = 0,            # 0 = automatic
    [switch]$NoUpdate
)

. (Join-Path $PSScriptRoot "common.ps1")

$keepAwake = $null
try {
    Assert-Tools
    Invoke-Housekeeping
    if (-not $NoUpdate) { Update-Repository }

    $results = Join-Path $script:Work "results"
    $pgns = @(Get-ChildItem -LiteralPath $results -Filter "*_calibration_*_vs_*.pgn" -ErrorAction SilentlyContinue)
    if ($pgns.Count -eq 0) { Stop-WithMessage "No calibration games found in $results. Run 5-Calibrate.bat first." }
    # The newest calibration: files share its date-and-time prefix.
    $stamp = ($pgns | ForEach-Object { $_.Name.Substring(0, 16) } | Sort-Object | Select-Object -Last 1)
    $files = @($pgns | Where-Object { $_.Name.StartsWith($stamp) } | ForEach-Object { $_.FullName })
    Write-Host "Using the calibration of $stamp ($($files.Count) opponents)."

    if ($Workers -le 0) { $Workers = [Math]::Max(1, (Get-DefaultConcurrency) - 4) }
    # Stockfish (downloaded by 13-Similarity-Test.bat) judges which move was
    # the mistake; only as a referee (see the originality rule in CLAUDE.md).
    $referee = @(Get-ChildItem -LiteralPath (Join-Path (Join-Path $script:Work "similarity") "Stockfish") -Recurse -Filter "*.exe" -ErrorAction SilentlyContinue) | Select-Object -First 1
    $refArgs = @()
    if ($referee) {
        Write-Host "Referee: $($referee.FullName)"
        $refArgs = @("--referee=$($referee.FullName)")
    } else {
        Write-Warn "No Stockfish found (run 13-Similarity-Test.bat once to download it); Trinity judges itself."
    }
    $trinity = Build-Trinity "origin/main"
    $keepAwake = Start-KeepAwake
    Write-Step "Loss forensics: up to $Losses lost games, $Workers at a time"
    $log = Join-Path $results "forensics_$stamp.log"
    $old = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        & $trinity (@("forensics", "$Losses", "$Workers") + $refArgs + $files) | Tee-Object -FilePath $log
    } finally {
        $ErrorActionPreference = $old
    }
    $text = Get-Content $log -Raw
    $start = $text.IndexOf("LOSS FORENSICS")
    if ($start -lt 0) { Stop-WithMessage "The forensics run did not finish. See $log" }
    $summary = "Calibration games of $stamp`r`n" + $text.Substring($start).TrimEnd()
    Set-Content -Path (Join-Path $results "LATEST-FORENSICS.txt") -Value $summary
    try {
        Publish-RepoFiles "Loss forensics of the calibration of $stamp" @{ "test-results/forensics-$stamp.txt" = $summary }
    } catch {
        Write-Warn "Could not save the result on GitHub: $($_.Exception.Message)"
    }
    Write-Host ""
    Write-Good "Done. Copy the LOSS FORENSICS block above and paste it to Claude (also saved in sprt\results\LATEST-FORENSICS.txt)."
} catch {
    Write-UnexpectedError $_
    exit 1
} finally {
    Stop-KeepAwake $keepAwake
}
