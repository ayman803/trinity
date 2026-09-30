# Run an SPRT test: does version "dev" play better than version "base"?
#
# Plays many fast games between the two versions and stops as soon as the
# statistics are clear (usually a few thousand to a few tens of thousands
# of games). Double-click 2-Run-SPRT-Test.bat to start it.

param(
    [string]$Dev = "",            # branch to test (asked interactively if empty)
    [string]$Base = "main",       # version to compare against
    [int]$Concurrency = 14,       # games played at the same time (= CPU threads used)
    [string]$TC = "8+0.08",       # time control: 8 seconds + 0.08 seconds per move
    [int]$Hash = 16,
    [double]$Elo0 = 0,            # SPRT bounds (normalized Elo). [0, 5] tests a
    [double]$Elo1 = 5,            #  gain; use -Elo0 -5 -Elo1 0 for "no regression".
    [int]$MaxRounds = 50000,      # each round is 2 games (both colours)
    [switch]$NoUpdate
)

. (Join-Path $PSScriptRoot "common.ps1")

function Select-Branch {
    $lines = Get-NativeOutput git @("-C", $script:Root, "for-each-ref", "--sort=-committerdate",
        "--format=%(refname:short)|%(committerdate:relative)|%(subject)", "refs/remotes/origin")
    $branches = @()
    foreach ($l in ($lines -split "`n")) {
        $parts = $l -split "\|", 3
        $name = $parts[0] -replace "^origin/", ""
        if ($name -eq "HEAD" -or $name -eq "origin" -or $name -eq $Base) { continue }
        $branches += , @($name, $parts[1], $parts[2])
    }
    if ($branches.Count -eq 0) { Stop-WithMessage "There are no branches to test besides '$Base'." }
    Write-Host ""
    Write-Host "Which version should be tested against '$Base'?"
    $shown = [Math]::Min(9, $branches.Count)
    for ($i = 0; $i -lt $shown; $i++) {
        $b = $branches[$i]
        Write-Host ("  {0}) {1}   ({2}: {3})" -f ($i + 1), $b[0], $b[1], $b[2])
    }
    $answer = Read-Host "Type a number and press Enter (just Enter = 1)"
    if (-not $answer) { $answer = "1" }
    $n = 0
    if (-not [int]::TryParse($answer, [ref]$n) -or $n -lt 1 -or $n -gt $shown) {
        Stop-WithMessage "'$answer' is not one of the numbers shown."
    }
    return $branches[$n - 1][0]
}

$keepAwake = $null
try {
    Assert-Tools
    if (-not $NoUpdate) { Update-Repository }
    if (-not $Dev) { $Dev = Select-Branch }

    $devExe = Build-Trinity "origin/$Dev"
    $baseExe = Build-Trinity "origin/$Base"
    $devBench = Get-BenchNodes $devExe
    $baseBench = Get-BenchNodes $baseExe
    Write-Host "Bench fingerprints: $Dev = $devBench, $Base = $baseBench"
    if ($devBench -eq $baseBench) {
        Write-Warn "Both versions search identically (same bench). That is expected only for pure speed-ups."
    }

    $fastchess = Get-Fastchess
    $book = Get-OpeningBook
    $results = Join-Path $script:Work "results"
    New-Item -ItemType Directory -Force -Path $results | Out-Null
    $stamp = Get-Date -Format "yyyy-MM-dd_HH-mm"
    $name = "${stamp}_$($Dev -replace '[^A-Za-z0-9._-]', '_')_vs_$Base"
    $log = Join-Path $results "$name.log"
    $pgn = Join-Path $results "$name.pgn"

    $fcArgs = @(
        "-engine", "cmd=$devExe", "name=$Dev",
        "-engine", "cmd=$baseExe", "name=$Base",
        "-each", "proto=uci", "tc=$TC", "option.Hash=$Hash", "option.Threads=1",
        "-openings", "file=$book", "format=epd", "order=random",
        "-repeat", "-games", "2", "-rounds", "$MaxRounds",
        "-concurrency", "$Concurrency",
        "-sprt", "elo0=$Elo0", "elo1=$Elo1", "alpha=0.05", "beta=0.05", "model=normalized",
        "-draw", "movenumber=40", "movecount=8", "score=10",
        "-resign", "movecount=3", "score=600", "twosided=true",
        "-ratinginterval", "20",
        "-pgnout", "file=$pgn"
    )

    Write-Step "Testing '$Dev' against '$Base' ($Concurrency games at a time, $TC)"
    Write-Host "This can take several hours. You can leave it running overnight."
    Write-Host "To stop early, press Ctrl+C (the result so far is still saved)."
    $keepAwake = Start-KeepAwake
    $started = Get-Date

    $old = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        & $fastchess @fcArgs | Tee-Object -FilePath $log
    } finally {
        $ErrorActionPreference = $old
    }

    # Summarise in plain words.
    $text = Get-Content $log -Raw
    $verdict = "UNFINISHED (stopped before the statistics were conclusive)"
    if ($text -match "H1 was accepted") { $verdict = "PASSED - '$Dev' is stronger than '$Base'" }
    elseif ($text -match "H0 was accepted") { $verdict = "FAILED - '$Dev' is not stronger than '$Base'" }
    $eloLine = ($text -split "`n" | Where-Object { $_ -match "^Elo:" } | Select-Object -Last 1)
    $gamesLine = ($text -split "`n" | Where-Object { $_ -match "^Games:" } | Select-Object -Last 1)
    $llrLine = ($text -split "`n" | Where-Object { $_ -match "^LLR:" } | Select-Object -Last 1)
    $hours = [Math]::Round(((Get-Date) - $started).TotalHours, 1)

    $summary = @(
        "SPRT RESULT: $verdict",
        "Test: $Dev (bench $devBench) vs $Base (bench $baseBench), tc $TC, bounds [$Elo0, $Elo1]",
        "$gamesLine".Trim(), "$eloLine".Trim(), "$llrLine".Trim(),
        "Duration: $hours hours. Full log: sprt\results\$name.log"
    ) | Where-Object { $_ }
    $summaryText = $summary -join "`r`n"
    Set-Content -Path (Join-Path $results "$name.summary.txt") -Value $summaryText
    Set-Content -Path (Join-Path $results "LATEST-RESULT.txt") -Value $summaryText

    Write-Host ""
    Write-Host "==================================================================" -ForegroundColor Cyan
    foreach ($s in $summary) { Write-Host $s }
    Write-Host "==================================================================" -ForegroundColor Cyan
    Write-Host "Copy the lines above and paste them to Claude."
    Write-Host "(They are also saved in sprt\results\LATEST-RESULT.txt)"
} catch {
    Write-UnexpectedError $_
    exit 1
} finally {
    Stop-KeepAwake $keepAwake
}
