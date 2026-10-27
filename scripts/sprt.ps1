# Run an SPRT test: does version "dev" play better than version "base"?
#
# Plays many fast games between the two versions and stops as soon as the
# statistics are clear (usually a few thousand to a few tens of thousands
# of games). Double-click 2-Run-SPRT-Test.bat to start it.

param(
    [string]$Dev = "",            # branch to test (asked interactively if empty)
    [string]$Base = "main",       # version to compare against
    [int]$Concurrency = 0,        # games at the same time; 0 = automatic (14 on the main PC)
    [string]$TC = "8+0.08",       # time control: 8 seconds + 0.08 seconds per move
    [int]$Hash = 16,
    [double]$Elo0 = 0,            # SPRT bounds (normalized Elo). [0, 5] tests a
    [double]$Elo1 = 5,            #  gain; use -Elo0 -5 -Elo1 0 for "no regression".
    [int]$MaxRounds = 50000,      # each round is 2 games (both colours)
    # Chunk mode (the runner's second test farm, 27 October): play exactly
    # this many rounds without stopping early, and report the counts so the
    # Session adds the chunks of one test up (scripts/sprt_combine.py).
    [int]$Rounds = 0,
    [string]$TestId = "",         # chunks of one test share it; default "<dev>-vs-<base>-<tc>"
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
        # Hide branches already merged into the base: testing them would
        # compare two identical engines.
        $old = $ErrorActionPreference; $ErrorActionPreference = "Continue"
        & git -C $script:Root merge-base --is-ancestor "origin/$name" "origin/$Base" 2>$null
        $merged = ($LASTEXITCODE -eq 0)
        $ErrorActionPreference = $old
        if ($merged) { continue }
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
    Invoke-Housekeeping
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

    if ($Concurrency -le 0) { $Concurrency = Get-DefaultConcurrency }
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
        "-repeat", "-games", "2", "-rounds", "$(if ($Rounds -gt 0) { $Rounds } else { $MaxRounds })",
        "-concurrency", "$Concurrency"
    )
    if ($Rounds -le 0) {
        $fcArgs += @("-sprt", "elo0=$Elo0", "elo1=$Elo1", "alpha=0.05", "beta=0.05", "model=normalized")
    }
    $fcArgs += @(
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
        "Machine: $(Get-MachineLabel) ($Concurrency games at a time)",
        "Test: $Dev (bench $devBench) vs $Base (bench $baseBench), tc $TC, bounds [$Elo0, $Elo1]",
        "$gamesLine".Trim(), "$eloLine".Trim(), "$llrLine".Trim(),
        "Duration: $hours hours. Full log: sprt\results\$name.log"
    ) | Where-Object { $_ }
    $summaryText = $summary -join "`r`n"
    Set-Content -Path (Join-Path $results "$name.summary.txt") -Value $summaryText
    Set-Content -Path (Join-Path $results "LATEST-RESULT.txt") -Value $summaryText

    # The counts in the format scripts/sprt_combine.py adds up (as each
    # GitHub job reports them), published under the runner.
    $lines = @($text -split "`n")
    $count = { param($pattern) @($lines | Where-Object { $_ -match $pattern }).Count }
    $ptnmlLine = ($lines | Where-Object { $_ -match "^Ptnml" } | Select-Object -Last 1)
    $chunk = @("$gamesLine".Trim(), "$ptnmlLine".Trim(), "tc=$TC", "dev_bench=$devBench", "base_bench=$baseBench",
        "time_losses=$(& $count '^Finished game.*loses on time')",
        "illegal_moves=$(& $count '^Finished game.*illegal move')",
        "crashes=$(& $count '^Finished game.*(disconnects|stalls)')",
        "machine=$(Get-MachineLabel) ($Concurrency games at a time)", "bounds=$Elo0 $Elo1") | Where-Object { $_ }
    if ($env:TRINITY_RESULTS_DIR) {
        if (-not $TestId) { $TestId = "$Dev-vs-$Base-$TC" }
        $safeId = $TestId -replace '[^A-Za-z0-9._-]', '_'
        Publish-ToResultsRepo "SPRT chunk: $Dev vs $Base ($stamp)" @{
            "sprt/$safeId/$stamp.txt" = ($chunk -join "`r`n")
        }
    }

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
