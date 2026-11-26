# Speed check (release checklist; advisor's note of 21 November 2026): how
# many positions per second Trinity searches on this PC in game-like
# conditions (one thread, Hash 256 MB as the CCRL testers use), for the
# builds a tester could run: the AVX2 file (x86-64-v3), the file for older
# processors (x86-64-v2), and this PC's own build for comparison. With
# -Release it also measures the Windows files of a published release (for
# 1.0.0, the files the CCRL tester has), downloaded from its release page.
#
# Method: every engine searches the same 12 middlegame positions from
# Trinity's own games for -Seconds each; the engines take turns position by
# position, so a slow minute on the PC hits all of them alike. The figure is
# the engine's own count (the "nps" of its last info line); the report gives
# the median over the positions. Double-click 20-Speed-Check.bat (about 10
# minutes; leave the PC alone meanwhile), or queue it for the runner.

param(
    [string]$Ref = "main",
    [int]$Hash = 256,
    [int]$Seconds = 8,
    [string]$Release = "",        # for example 1.0.0: also measure that release's Windows files
    [switch]$NoUpdate
)

. (Join-Path $PSScriptRoot "common.ps1")

$positions = @(
    "r5k1/3bbpp1/p3pn1p/1p2N3/n6B/P3P3/BPR1NPPP/6K1 b - - 0 20",
    "r4rk1/4qppp/b1p1p3/R1PnN3/8/Q5P1/4PPBP/4K2R b K - 0 20",
    "r4rk1/1q2bppp/p2pp3/8/1P2P3/3n2NP/R2B1PP1/3QR1K1 w - - 1 26",
    "4r3/1R3pbk/p6p/P4rpn/5PR1/3P1N2/1P1B3P/6K1 w - - 0 29",
    "5R2/1r1r4/3p1n2/3Pp1k1/p3p2p/2Nb3P/P4NP1/2R3K1 b - - 1 72",
    "r2q4/5pk1/r3pbp1/p1pn3p/4B2P/R1PP2P1/RP3PK1/Q1B5 w - - 21 40",
    "1r5k/4qp2/b5pp/p2Q4/Pp1P2B1/4P2P/5PP1/2R3K1 b - - 0 28",
    "rn3rk1/pp2ppbp/q1p1bnp1/6N1/Q2P4/2N3PP/PP2PPB1/R1B1K2R b KQ - 4 11",
    "r4rk1/1p3p2/2p4p/p1R1q1p1/P3P3/1P1PB2n/5P2/3RQ2K b - - 1 28",
    "1r1q1rk1/1p1b1ppp/p3p3/8/Pn1pPP2/1B1P3P/1P4P1/R2Q1RNK w - - 0 20",
    "5rk1/p2bqrbp/1pn1p1p1/1B1p4/3P4/P1NQPN2/1P3RPP/4R1K1 b - - 8 22",
    "r1bq1rk1/pp3pbp/6p1/2p5/5B2/3P2P1/PP3PBP/1R1Q1RK1 b - - 0 15"
)

# One running engine, spoken to in UCI.
function Start-Uci([string]$Path, [int]$HashMb) {
    $info = New-Object System.Diagnostics.ProcessStartInfo
    $info.FileName = $Path
    $info.UseShellExecute = $false
    $info.RedirectStandardInput = $true
    $info.RedirectStandardOutput = $true
    $info.CreateNoWindow = $true
    $proc = [System.Diagnostics.Process]::Start($info)
    $proc.StandardInput.WriteLine("uci")
    $name = ""
    while ($null -ne ($line = $proc.StandardOutput.ReadLine())) {
        if ($line -match "^id name (.*)$") { $name = $Matches[1].Trim() }
        if ($line -eq "uciok") { break }
    }
    $proc.StandardInput.WriteLine("setoption name Hash value $HashMb")
    $proc.StandardInput.WriteLine("setoption name Threads value 1")
    $proc.StandardInput.WriteLine("isready")
    while ($null -ne ($line = $proc.StandardOutput.ReadLine())) { if ($line -eq "readyok") { break } }
    if ($proc.HasExited) { throw "The engine $Path stopped while starting." }
    return @{ Process = $proc; Name = $name }
}

# Search one position for a fixed time; returns the nps, depth and hashfull
# of the last info line.
function Measure-Position($Engine, [string]$Fen, [int]$Ms) {
    $p = $Engine.Process
    $p.StandardInput.WriteLine("ucinewgame")
    $p.StandardInput.WriteLine("position fen $Fen")
    $p.StandardInput.WriteLine("go movetime $Ms")
    $nps = 0; $depth = 0; $full = 0
    while ($null -ne ($line = $p.StandardOutput.ReadLine())) {
        if ($line -match "^info .*\bdepth (\d+)\b.*\bnps (\d+)\b") {
            $depth = [int]$Matches[1]; $nps = [long]$Matches[2]
            if ($line -match "\bhashfull (\d+)\b") { $full = [int]$Matches[1] }
        }
        if ($line -match "^bestmove") { break }
    }
    if ($nps -le 0) { throw "No speed figure from $($Engine.Name) for position $Fen." }
    return @{ Nps = $nps; Depth = $depth; Hashfull = $full }
}

function Stop-Uci($Engine) {
    try {
        $Engine.Process.StandardInput.WriteLine("quit")
        if (-not $Engine.Process.WaitForExit(5000)) { $Engine.Process.Kill() }
    } catch { }
}

function Get-Median([long[]]$Values) {
    $s = @($Values | Sort-Object)
    $n = $s.Count
    if ($n % 2 -eq 1) { return [long]$s[($n - 1) / 2] }
    return [long](($s[$n / 2 - 1] + $s[$n / 2]) / 2)
}

$keepAwake = $null
$running = @()
try {
    Assert-Tools
    Invoke-Housekeeping
    if (-not $NoUpdate) { Update-Repository }
    if ($Seconds -lt 1 -or $Seconds -gt 60) { Stop-WithMessage "-Seconds must be between 1 and 60." }
    if ($Release -and $Release -notmatch '^\d+\.\d+\.\d+$') { Stop-WithMessage "-Release must look like 1.0.0." }

    $builds = [ordered]@{}
    $builds["$Ref, AVX2 file (x86-64-v3)"] = Build-Trinity "origin/$Ref" -Cpu "x86-64-v3"
    $builds["$Ref, older processors (x86-64-v2)"] = Build-Trinity "origin/$Ref" -Cpu "x86-64-v2"
    $builds["$Ref, this PC's own build"] = Build-Trinity "origin/$Ref"
    $bench = Get-BenchNodes $builds["$Ref, AVX2 file (x86-64-v3)"]
    if ($Release) {
        if (-not $script:OnWindows) { Stop-WithMessage "-Release measures Windows files; run it on Windows." }
        $dir = Join-Path (Join-Path $script:Work "release-files") $Release
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        foreach ($cpu in @("x86-64-v3", "x86-64-v2")) {
            $file = Join-Path $dir "trinity-$Release-windows-$cpu.exe"
            if (-not (Test-Path -LiteralPath $file)) {
                Save-Download "https://github.com/ayman803/trinity/releases/download/v$Release/trinity-$Release-windows-$cpu.exe" $file
            }
            $builds["release $Release, published file ($cpu)"] = $file
        }
    }

    $keepAwake = Start-KeepAwake
    Write-Step "Speed check: $($builds.Count) builds, $($positions.Count) positions, $Seconds s each, Hash $Hash MB (about $([Math]::Ceiling($builds.Count * $positions.Count * $Seconds / 60.0)) minutes)"
    $names = @($builds.Keys)
    foreach ($k in $names) { $running += , (Start-Uci $builds[$k] $Hash) }
    $nps = @{}; $depth = @{}; $full = @{}
    foreach ($k in $names) { $nps[$k] = @(); $depth[$k] = @(); $full[$k] = @() }
    for ($i = 0; $i -lt $positions.Count; $i++) {
        for ($j = 0; $j -lt $names.Count; $j++) {
            # A different engine goes first each round.
            $e = ($i + $j) % $names.Count
            $r = Measure-Position $running[$e] $positions[$i] (1000 * $Seconds)
            $nps[$names[$e]] += [long]$r.Nps; $depth[$names[$e]] += [long]$r.Depth; $full[$names[$e]] += [long]$r.Hashfull
        }
        Write-Host ("Position {0} of {1} done." -f ($i + 1), $positions.Count)
    }
    foreach ($e in $running) { Stop-Uci $e }
    $running = @()

    $base = Get-Median $nps[$names[0]]
    $stamp = Get-Date -Format "yyyy-MM-dd_HH-mm"
    $summary = @(
        "SPEED CHECK of '$Ref' (bench $bench) on $(Get-MachineLabel)",
        "One thread, Hash $Hash MB, $($positions.Count) middlegame positions, $Seconds s each; median positions per second (the engine's own count)")
    foreach ($k in $names) {
        $m = Get-Median $nps[$k]
        $lo = ($nps[$k] | Measure-Object -Minimum).Minimum
        $hi = ($nps[$k] | Measure-Object -Maximum).Maximum
        $summary += ("{0}: {1:N0} nps ({2:N0} to {3:N0}), {4:P0} of the AVX2 file; median depth {5}, hash used {6} per mille" -f
            $k, $m, $lo, $hi, ($m / [double]$base), (Get-Median $depth[$k]), (Get-Median $full[$k]))
    }
    $summaryText = $summary -join "`r`n"
    $results = Join-Path $script:Work "results"
    New-Item -ItemType Directory -Force -Path $results | Out-Null
    Set-Content -Path (Join-Path $results "LATEST-SPEEDCHECK.txt") -Value $summaryText
    try {
        Publish-RepoFiles "Speed check of $Ref" @{ "test-results/speedcheck-$stamp.txt" = $summaryText }
    } catch {
        Write-Warn "Could not save the result on GitHub: $($_.Exception.Message)"
    }
    Write-Host ""
    Write-Host "==================================================================" -ForegroundColor Cyan
    foreach ($s in $summary) { Write-Host $s }
    Write-Host "==================================================================" -ForegroundColor Cyan
    Write-Host "Copy the lines above and paste them to Claude (also saved in sprt\results\LATEST-SPEEDCHECK.txt)."
} catch {
    Write-UnexpectedError $_
    exit 1
} finally {
    foreach ($e in $running) { Stop-Uci $e }
    if ($keepAwake) { Stop-KeepAwake $keepAwake }
}
