# Calibration: estimate Trinity's real CCRL rating.
#
# Self-play tests (SPRT) only say "better or worse than the last version".
# To know where Trinity stands on the CCRL list, it plays matches against
# engines that have a known CCRL rating. Double-click 5-Calibrate.bat.
#
# The opponents are built from their authors' public source code at the
# exact version that CCRL rated, so no download links can go stale.

param(
    [string]$Ref = "main",        # Trinity version to measure
    [int]$Rounds = 500,           # per opponent; each round is 2 games (both colours)
    [string]$TC = "10+0.1",
    [int]$Hash = 16,
    [int]$Concurrency = 0,        # 0 = automatic (14 on the main PC)
    [switch]$NoUpdate,
    # One-off match instead of calibration: the name of an unrated opponent
    # whose Windows build is saved as sprt\opponents\<name>.exe.
    [string]$Vs = ""
)

. (Join-Path $PSScriptRoot "common.ps1")

# CCRL 40/15 ratings (single CPU), list of 2 October 2026.
$opponents = @(
    @{ Name = "akimbo"; Version = "1.0.0"; Rating = 3474; Url = "https://github.com/jw1912/akimbo";
        Tag = "v1.0.0"; Binary = "akimbo"; CargoArgs = @(); Env = @{ EVALFILE = "resources/net.bin" } },
    # Downloaded by hand (Windows release builds): see GUIDE.md.
    @{ Name = "Patricia"; Version = "5.1"; Rating = 3487; Download = $true },
    @{ Name = "Bread"; Version = "4.0.0"; Rating = 3522; Download = $true },
    @{ Name = "Prune"; Version = "4.0.1"; Rating = 3543; Download = $true }
)
if ($Vs) { $opponents = @(@{ Name = $Vs; Version = ""; Rating = $null; Download = $true }) }
# The CCRL top-20 cut-off (Halogen 16, 4 CPUs) on the same list.
$top20 = 3625

function Build-Opponent($Opp) {
    $dir = Join-Path $script:Work "opponents"
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    $oppExe = Join-Path $dir "$($Opp.Name)-$($Opp.Version)$($script:ExeSuffix)"
    if (Test-Path $oppExe) { return $oppExe }

    Write-Step "Building opponent $($Opp.Name) $($Opp.Version) from its source code"
    $src = Join-Path $dir "src-$($Opp.Name)"
    if (Test-Path $src) { Remove-Item -Recurse -Force $src }
    Invoke-Native git @("clone", "--quiet", "--depth", "1", "--branch", $Opp.Tag, $Opp.Url, $src)
    $saved = @{}
    $vars = @{ RUSTFLAGS = "-C target-cpu=native" } + $Opp.Env
    foreach ($k in $vars.Keys) {
        $saved[$k] = [Environment]::GetEnvironmentVariable($k)
        [Environment]::SetEnvironmentVariable($k, $vars[$k])
    }
    try {
        $target = Join-Path $dir "target-$($Opp.Name)"
        Invoke-Native cargo (@("build", "--release", "--manifest-path", (Join-Path $src "Cargo.toml"),
                "--target-dir", $target) + $Opp.CargoArgs)
    } finally {
        foreach ($k in $saved.Keys) { [Environment]::SetEnvironmentVariable($k, $saved[$k]) }
    }
    $output = Join-Path (Join-Path $target "release") "$($Opp.Binary)$($script:ExeSuffix)"
    if (-not (Test-Path $output)) { Stop-WithMessage "Building $($Opp.Name) did not produce $output." }
    Copy-Item $output $oppExe -Force
    Remove-Item -Recurse -Force $src
    return $oppExe
}

$keepAwake = $null
try {
    Assert-Tools
    Invoke-Housekeeping
    if (-not $NoUpdate) { Update-Repository }

    $running = @(Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.ProcessName -like "trinity*" })
    if ($running.Count -gt 0) {
        Write-Warn "Another Trinity job is still running (data generation or a test)."
        Write-Warn "Running both at once makes both slower and this measurement less accurate."
        Read-Host "Press Enter to continue anyway, or close this window to wait"
    }

    $trinity = Build-Trinity "origin/$Ref"
    $bench = Get-BenchNodes $trinity
    $built = @()
    $opponentDir = Join-Path $script:Work "opponents"
    foreach ($o in $opponents) {
        if ($o.Download) {
            $file = Join-Path $opponentDir "$($o.Name).exe"
            if (Test-Path -LiteralPath $file) {
                $built += , @($o, $file)
            } else {
                Write-Warn "Skipping $($o.Name) $($o.Version): save its Windows .exe as $file to include it."
            }
        } else {
            $built += , @($o, (Build-Opponent $o))
        }
    }

    if ($Concurrency -le 0) { $Concurrency = Get-DefaultConcurrency }
    $fastchess = Get-Fastchess
    # A balanced book, closer to how rating lists start their games than the
    # unbalanced book used for SPRT.
    $book = Get-OpeningBook "8moves_v3.pgn"
    $results = Join-Path $script:Work "results"
    New-Item -ItemType Directory -Force -Path $results | Out-Null
    $stamp = Get-Date -Format "yyyy-MM-dd_HH-mm"

    Write-Step "Calibration of '$Ref' ($($built.Count) opponents x $(2 * $Rounds) games, $TC, $Concurrency games at a time)"
    Write-Host "This takes about 30 minutes per opponent. To stop early, press Ctrl+C."
    $keepAwake = Start-KeepAwake
    $started = Get-Date

    $lines = @()
    $sumW = 0.0; $sumWR = 0.0
    foreach ($pair in $built) {
        $o = $pair[0]; $oppExe = $pair[1]
        $name = "${stamp}_calibration_$($Ref -replace '[^A-Za-z0-9._-]', '_')_vs_$($o.Name)"
        $log = Join-Path $results "$name.log"
        $pgn = Join-Path $results "$name.pgn"
        $fcArgs = @(
            "-engine", "cmd=$trinity", "name=Trinity",
            "-engine", "cmd=$oppExe", "name=$($o.Name)",
            "-each", "proto=uci", "tc=$TC", "option.Hash=$Hash", "option.Threads=1",
            "-openings", "file=$book", "format=pgn", "order=random",
            "-repeat", "-games", "2", "-rounds", "$Rounds",
            "-concurrency", "$Concurrency",
            "-draw", "movenumber=40", "movecount=8", "score=10",
            "-resign", "movecount=3", "score=600", "twosided=true",
            "-ratinginterval", "50",
            "-pgnout", "file=$pgn"
        )
        Write-Step "Trinity vs $($o.Name) $($o.Version) (CCRL $($o.Rating))"
        $old = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        try {
            & $fastchess @fcArgs | Tee-Object -FilePath $log
        } finally {
            $ErrorActionPreference = $old
        }

        $text = Get-Content $log -Raw
        $eloLine = ($text -split "`n" | Where-Object { $_ -match "^Elo:" } | Select-Object -Last 1)
        $gamesLine = ($text -split "`n" | Where-Object { $_ -match "^Games:" } | Select-Object -Last 1)
        $timeLosses = 0
        if (Test-Path $pgn) { $timeLosses = @(Select-String -Path $pgn -Pattern "time forfeit" -SimpleMatch).Count }
        $inv = [Globalization.CultureInfo]::InvariantCulture
        if ("$eloLine" -match "^Elo:\s*(-?[0-9.]+)\s*\+/-\s*([0-9.]+)") {
            $diff = [double]::Parse($Matches[1], $inv)
            $margin = [double]::Parse($Matches[2], $inv)
            if ($null -eq $o.Rating) {
                $lines += ("vs {0}: Elo difference {1:+0;-0;0} +/- {2:0} (Trinity's point of view)" -f
                    $o.Name, [Math]::Round($diff), $margin)
                if ($gamesLine) { $lines += "   $("$gamesLine".Trim())" }
                continue
            }
            $est = $o.Rating + $diff
            $lines += ("vs {0} {1} (CCRL {2}): Elo difference {3:+0;-0;0} +/- {4:0} -> Trinity about {5:0}" -f
                $o.Name, $o.Version, $o.Rating, [Math]::Round($diff), $margin, $est)
            if ($margin -gt 0) {
                $w = 1.0 / ($margin * $margin)
                $sumW += $w; $sumWR += $w * $est
            }
        } else {
            $label = if ($null -eq $o.Rating) { $o.Name } else { "$($o.Name) $($o.Version) (CCRL $($o.Rating))" }
            $lines += "vs ${label}: no usable result (too few games, or one side won everything)"
        }
        if ($gamesLine) { $lines += "   $("$gamesLine".Trim())" }
        if ($timeLosses -gt 0) { $lines += "   Games lost on time (either side): $timeLosses" }
    }

    $hours = [Math]::Round(((Get-Date) - $started).TotalHours, 1)
    $title = if ($Vs) { "MATCH RESULT" } else { "CALIBRATION RESULT" }
    $summary = @("$title for '$Ref' (bench $bench)",
        "Machine: $([Environment]::MachineName) ($Concurrency games at a time), tc $TC, 1 thread each") + $lines
    if ($sumW -gt 0) {
        $estimate = $sumWR / $sumW
        $error95 = 1.0 / [Math]::Sqrt($sumW)
        $summary += ("ESTIMATED CCRL RATING (1 CPU): about {0:0} (statistical +/- {1:0}; real uncertainty about +/- 50-100)" -f $estimate, $error95)
        $summary += ("Top-20 cut-off: {0} (4 CPUs). Gap: about {1:0} Elo" -f $top20, ($top20 - $estimate))
    }
    $summary += "Duration: $hours hours. Logs: sprt\results\${stamp}_calibration_*"
    $summaryText = $summary -join "`r`n"
    Set-Content -Path (Join-Path $results "${stamp}_calibration.summary.txt") -Value $summaryText
    $latest = if ($Vs) { "LATEST-MATCH.txt" } else { "LATEST-CALIBRATION.txt" }
    Set-Content -Path (Join-Path $results $latest) -Value $summaryText

    Write-Host ""
    Write-Host "==================================================================" -ForegroundColor Cyan
    foreach ($s in $summary) { Write-Host $s }
    Write-Host "==================================================================" -ForegroundColor Cyan
    Write-Host "Copy the lines above and paste them to Claude."
    Write-Host "(They are also saved in sprt\results\$latest)"
} catch {
    Write-UnexpectedError $_
    exit 1
} finally {
    Stop-KeepAwake $keepAwake
}
