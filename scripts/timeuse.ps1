# Time use: how Trinity spends its clock in the calibration games (see
# src/timeuse.rs). One report per calibration found in sprt\results, saved
# on GitHub as test-results/timeuse-<date>.txt. Double-click
# 18-Time-Use.bat (about a minute; no games are played).

param([switch]$NoUpdate)

. (Join-Path $PSScriptRoot "common.ps1")

try {
    Assert-Tools
    Invoke-Housekeeping
    if (-not $NoUpdate) { Update-Repository }

    $results = Join-Path $script:Work "results"
    $pgns = @(Get-ChildItem -LiteralPath $results -Filter "*_calibration_*_vs_*.pgn" -ErrorAction SilentlyContinue)
    if ($pgns.Count -eq 0) { Stop-WithMessage "No calibration games found in $results. Run 5-Calibrate.bat first." }
    $trinity = Build-Trinity "origin/main"
    $stamps = @($pgns | ForEach-Object { $_.Name.Substring(0, 16) } | Sort-Object -Unique)
    $files = @{}
    $all = @()
    foreach ($stamp in $stamps) {
        $group = @($pgns | Where-Object { $_.Name.StartsWith($stamp) } | ForEach-Object { $_.FullName })
        Write-Step "Calibration of $stamp ($($group.Count) opponents)"
        $old = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        try {
            $text = (& $trinity (@("timeuse") + $group) 2>&1 | Out-String)
        } finally {
            $ErrorActionPreference = $old
        }
        Write-Host $text
        if ($text -match "TIME USE") {
            $block = "Calibration games of $stamp`r`n" + $text.Trim()
            $files["test-results/timeuse-$stamp.txt"] = $block
            $all += $block
        }
    }
    if ($all.Count -eq 0) { Stop-WithMessage "No report could be made." }
    Set-Content -Path (Join-Path $results "LATEST-TIMEUSE.txt") -Value ($all -join "`r`n`r`n")
    try {
        Publish-RepoFiles "Time use in the calibration games" $files
    } catch {
        Write-Warn "Could not save the reports on GitHub: $($_.Exception.Message)"
    }
    Write-Host ""
    Write-Good "Done. The TIME USE blocks above are saved in sprt\results\LATEST-TIMEUSE.txt; paste them to Claude (or just say 'time use done' if they were saved on GitHub)."
} catch {
    Write-UnexpectedError $_
    exit 1
}
