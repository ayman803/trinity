# Older-processor check: builds a branch twice, for this PC (with AVX2) and
# as the x86-64-v2 release build (no AVX2, the plain code path), checks that
# both give the same bench node count, and plays games between the two to
# catch crashes, illegal moves and time losses in the v2 build. Double-click
# 19-V2-Check.bat (about 15 minutes for 20 games).

param(
    [string]$Ref = "combo",
    [int]$Rounds = 10,            # each round is 2 games
    [string]$TC = "10+0.1",
    [switch]$NoUpdate
)

. (Join-Path $PSScriptRoot "common.ps1")

try {
    Assert-Tools
    Invoke-Housekeeping
    if (-not $NoUpdate) { Update-Repository }

    $fast = Build-Trinity "origin/$Ref"
    $v2 = Build-Trinity "origin/$Ref" -Cpu "x86-64-v2"
    $builds = [ordered]@{ "AVX2" = $fast; "v2" = $v2 }
    $benchText = @{}
    $nodes = @{}
    foreach ($k in @($builds.Keys)) {
        $out = Get-NativeOutput $builds[$k] @("bench")
        if (-not ($out -match "(\d+) nodes (\d+) nps")) { Stop-WithMessage "bench failed for the $k build: $out" }
        $nodes[$k] = $Matches[1]
        $benchText[$k] = "$($Matches[1]) nodes, $($Matches[2]) nps"
    }
    $sameBench = $nodes["AVX2"] -eq $nodes["v2"]

    $fastchess = Get-Fastchess
    $book = Get-OpeningBook "8moves_v3.pgn"
    $results = Join-Path $script:Work "results"
    New-Item -ItemType Directory -Force -Path $results | Out-Null
    $stamp = Get-Date -Format "yyyy-MM-dd_HH-mm"
    $name = "${stamp}_v2check_$($Ref -replace '[^A-Za-z0-9._-]', '_')"
    $log = Join-Path $results "$name.log"
    $pgn = Join-Path $results "$name.pgn"
    Write-Step "Playing $(2 * $Rounds) games: v2 build vs AVX2 build, $TC"
    $fcArgs = @(
        "-engine", "cmd=$v2", "name=Trinity-v2",
        "-engine", "cmd=$fast", "name=Trinity-AVX2",
        "-each", "proto=uci", "tc=$TC", "option.Hash=16", "option.Threads=1",
        "-openings", "file=$book", "format=pgn", "order=random",
        "-srand", "$(Get-Random -Minimum 1 -Maximum 1000000)",
        "-repeat", "-games", "2", "-rounds", "$Rounds",
        "-concurrency", "2",
        "-draw", "movenumber=40", "movecount=8", "score=10",
        "-resign", "movecount=3", "score=600", "twosided=true",
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
    $games = & $count "[Result "
    $timeLosses = & $count "time forfeit"
    $illegal = & $count "illegal move"
    $crashes = (& $count "disconnect") + (& $count "stall")
    $passed = $sameBench -and $games -ge 2 * $Rounds -and $timeLosses -eq 0 -and $illegal -eq 0 -and $crashes -eq 0
    $summary = @(
        "V2 CHECK: '$Ref' on $([Environment]::MachineName) - $(if ($passed) { 'PASSED' } else { 'FAILED' })",
        "bench AVX2 build: $($benchText['AVX2'])",
        "bench v2 build:   $($benchText['v2'])",
        "Same bench node count: $(if ($sameBench) { 'yes' } else { 'NO' })",
        "Games finished: $games of $(2 * $Rounds), tc $TC") + $lines + @(
        "Games lost on time: $timeLosses; illegal moves: $illegal; crashes or hangs: $crashes")
    $summaryText = $summary -join "`r`n"
    Set-Content -Path (Join-Path $results "LATEST-V2CHECK.txt") -Value $summaryText
    try {
        Publish-RepoFiles "v2 check of $Ref" @{ "test-results/v2check-$stamp.txt" = $summaryText }
    } catch {
        Write-Warn "Could not save the result on GitHub: $($_.Exception.Message)"
    }

    Write-Host ""
    Write-Host "==================================================================" -ForegroundColor Cyan
    foreach ($s in $summary) { Write-Host $s }
    Write-Host "==================================================================" -ForegroundColor Cyan
    Write-Host "Copy the lines above and paste them to Claude (also saved in sprt\results\LATEST-V2CHECK.txt)."
} catch {
    Write-UnexpectedError $_
    exit 1
}
