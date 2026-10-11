# Threads test: Trinity with 4 threads against Trinity with 1 thread, same
# clock for both. Shows whether extra threads make it stronger (CCRL also
# tests engines on 4 CPUs) and counts crashes, illegal moves and time
# losses. Double-click 14-Threads-Test.bat on the PC or the laptop; close
# other programs first. Each machine saves its own result.

param(
    [string]$Ref = "main",
    [int]$Rounds = 0,             # each round is 2 games; 0 = automatic
    [string]$TC = "10+0.1",
    [int]$Threads = 4,
    [int]$Hash = 64,
    [int]$Concurrency = 0,        # 0 = automatic
    [switch]$NoUpdate
)

. (Join-Path $PSScriptRoot "common.ps1")

$keepAwake = $null
try {
    Assert-Tools
    Invoke-Housekeeping
    if (-not $NoUpdate) { Update-Repository }

    $running = @(Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.ProcessName -like "trinity*" })
    if ($running.Count -gt 0) {
        Write-Warn "Another Trinity job is still running. The threads test needs the whole processor."
        Read-Host "Close it and press Enter, or close this window to wait"
    }

    # Games at once: one game uses at most $Threads cores at a time (only the
    # side to move thinks). Allow one game per $Threads + 1 real cores, and
    # only one on processors that mix fast and slow cores (the laptop), where
    # threads would otherwise land on the slow ones.
    $logical = [Environment]::ProcessorCount
    $physical = $null
    try {
        $physical = (Get-CimInstance Win32_Processor -ErrorAction Stop | Measure-Object -Property NumberOfCores -Sum).Sum
    } catch { }
    if (-not $physical) { $physical = [Math]::Max(1, [int]($logical / 2)) }
    $hybrid = $logical -ne 2 * $physical -and $logical -ne $physical
    if ($Concurrency -le 0) {
        $Concurrency = if ($hybrid) { 1 } else { [Math]::Max(1, [Math]::Floor($physical / ($Threads + 1))) }
    }
    if ($Rounds -le 0) { $Rounds = if ($Concurrency -ge 2) { 500 } else { 250 } }
    Write-Host "Processor: $physical cores, $logical threads$(if ($hybrid) { ' (fast and slow cores mixed)' })."

    $trinity = Build-Trinity "origin/$Ref"
    $bench = Get-BenchNodes $trinity
    $fastchess = Get-Fastchess
    $book = Get-OpeningBook "8moves_v3.pgn"
    $results = Join-Path $script:Work "results"
    New-Item -ItemType Directory -Force -Path $results | Out-Null
    $stamp = Get-Date -Format "yyyy-MM-dd_HH-mm"
    $name = "${stamp}_threads_${Threads}v1"
    $log = Join-Path $results "$name.log"
    $pgn = Join-Path $results "$name.pgn"
    # Different openings on each machine, so the two results can be added up.
    $seed = Get-Random -Minimum 1 -Maximum 1000000

    Write-Step "Threads test: $Threads threads vs 1 thread, $(2 * $Rounds) games, $TC, $Concurrency game(s) at a time"
    Write-Host "This takes about 4-5 hours. Close other programs. To stop early, press Ctrl+C."
    $keepAwake = Start-KeepAwake
    $started = Get-Date
    $fcArgs = @(
        "-engine", "cmd=$trinity", "name=Trinity-${Threads}T", "option.Threads=$Threads",
        "-engine", "cmd=$trinity", "name=Trinity-1T", "option.Threads=1",
        "-each", "proto=uci", "tc=$TC", "option.Hash=$Hash",
        "-openings", "file=$book", "format=pgn", "order=random",
        "-srand", "$seed",
        "-repeat", "-games", "2", "-rounds", "$Rounds",
        "-concurrency", "$Concurrency",
        "-draw", "movenumber=40", "movecount=8", "score=10",
        "-resign", "movecount=3", "score=600", "twosided=true",
        "-ratinginterval", "50",
        "-pgnout", "file=$pgn"
    )
    $old = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        & $fastchess @fcArgs | Tee-Object -FilePath $log
    } finally {
        $ErrorActionPreference = $old
    }

    $text = Get-Content $log -Raw
    $lines = @($text -split "`n" | Where-Object { $_ -match "^(Elo|Games|Ptnml)" } | Select-Object -Last 3 | ForEach-Object { "$_".Trim() })
    $count = { param($pattern) if (Test-Path $pgn) { @(Select-String -Path $pgn -Pattern $pattern -SimpleMatch).Count } else { 0 } }
    $timeLosses = & $count "time forfeit"
    $illegal = & $count "illegal move"
    $crashes = (& $count "disconnect") + (& $count "stall")
    $hours = [Math]::Round(((Get-Date) - $started).TotalHours, 1)
    $summary = @("THREADS TEST: '$Ref' (bench $bench), $Threads threads vs 1 thread",
        "Machine: $([Environment]::MachineName), $physical cores / $logical threads, $Concurrency game(s) at a time, tc $TC, hash $Hash MB") +
        $lines + @(
        "Games lost on time: $timeLosses; illegal moves: $illegal; crashes or hangs: $crashes",
        "(Elo is from the $Threads-thread side's point of view.)",
        "Duration: $hours hours. Log: sprt\results\$name.log")
    $summaryText = $summary -join "`r`n"
    Set-Content -Path (Join-Path $results "LATEST-THREADS.txt") -Value $summaryText

    Write-Host ""
    Write-Host "==================================================================" -ForegroundColor Cyan
    foreach ($s in $summary) { Write-Host $s }
    Write-Host "==================================================================" -ForegroundColor Cyan
    Write-Host "Copy the lines above and paste them to Claude."
    Write-Host "(They are also saved in sprt\results\LATEST-THREADS.txt)"
} catch {
    Write-UnexpectedError $_
    exit 1
} finally {
    Stop-KeepAwake $keepAwake
}
