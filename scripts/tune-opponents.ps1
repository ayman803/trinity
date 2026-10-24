# Re-tuning against the rated opponents (docs/spsa-runner-design.md;
# Ayman with the advisor, 20 October 2026: 4 settings, the unseen-opponent
# check decides). Used by calibrate.ps1 -Tune; not run on its own.
#
# SPSA, as scripts/spsa.py does on GitHub, with one difference: the "up"
# and "down" versions do not play each other. Each plays the same openings
# (same seed, both colours) against the same rated opponent, and the
# difference of their two scores moves the settings. One opponent per
# step, in turn. Two independent chains (A and B, own openings; C and D
# since razoring was removed on 21 October) run as separate jobs; at night 3 they must agree in direction on at least 3 of
# the 4 settings, or the experiment stops.
#
# Progress is saved after every step (sprt\spsa\state-<chain>.json), so a
# job stopped because Ayman came back loses at most one step. Under the
# runner the state and a summary are published in trinity-results (spsa\).

# The 4 settings that moved most, in the same direction, in both earlier
# tunes. RAZOR_MUL (third) was replaced by the fifth, LMR_HIST_DIV, when
# razoring was removed (21 October; RAZOR_BASE, fourth, went with it), and
# QS_FUTILITY by FUT_MUL when quiescence futility was removed (22 October;
# widening futility then gained +5.8, so its margin is worth tuning).
$script:TuneNames = @("FUT_MUL", "LMP_BASE", "LMR_HIST_DIV", "RFP_MARGIN")
# Planned pairs per chain (about 6 nights of one 4.5-hour job per chain);
# sets the step-size schedule, as --total-pairs does in spsa.py.
$script:TuneTotalPairs = 11000
$script:TuneAlpha = 0.602
$script:TuneGamma = 0.101
$script:TuneREnd = 0.002

function Get-TuneSettings([string]$Sha) {
    $text = (Get-NativeOutput git @("-C", $script:Root, "show", "${Sha}:src/tune.rs")) -join "`n"
    $inv = [Globalization.CultureInfo]::InvariantCulture
    $out = [ordered]@{}
    foreach ($name in $script:TuneNames) {
        if ($text -notmatch "(?m)^\s*$name = (-?\d+), (-?\d+), (-?\d+), ([\d.]+);") {
            Stop-WithMessage "Setting $name not found in src/tune.rs at $Sha."
        }
        $out[$name] = @{ Default = [int]$Matches[1]; Min = [int]$Matches[2]; Max = [int]$Matches[3];
            Step = [double]::Parse($Matches[4], $inv) }
    }
    return $out
}

# The saved state with the most pairs: this machine's own copy, or the one
# published in trinity-results (the newer job may have run elsewhere).
function Read-TuneState([string]$Chain, $Settings) {
    $best = $null
    $files = @(Join-Path (Join-Path $script:Work "spsa") "state-$Chain.json")
    if ($env:TRINITY_RESULTS_DIR) {
        Invoke-Native git @("-C", $env:TRINITY_RESULTS_DIR, "pull", "-q", "--rebase", "origin", "main")
        $files += Join-Path (Join-Path $env:TRINITY_RESULTS_DIR "spsa") "state-$Chain.json"
    }
    foreach ($f in $files) {
        if (-not (Test-Path -LiteralPath $f)) { continue }
        $s = Get-Content -LiteralPath $f -Raw | ConvertFrom-Json
        if ($null -eq $best -or [int]$s.pairs -gt [int]$best.pairs) { $best = $s }
    }
    $theta = [ordered]@{}
    foreach ($name in $Settings.Keys) {
        $theta[$name] = [double]$Settings[$name].Default
        if ($best -and $null -ne $best.theta.$name) { $theta[$name] = [double]$best.theta.$name }
    }
    $state = [ordered]@{ chain = $Chain; pairs = 0; steps = 0; games = 0; timeLosses = 0; theta = $theta }
    if ($best) {
        $state.pairs = [int]$best.pairs; $state.steps = [int]$best.steps
        $state.games = [int]$best.games; $state.timeLosses = [int]$best.timeLosses
    }
    return $state
}

function Save-TuneState($State) {
    $dir = Join-Path $script:Work "spsa"
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    $file = Join-Path $dir "state-$($State.chain).json"
    $tmp = "$file.tmp"
    Set-Content -LiteralPath $tmp -Value ($State | ConvertTo-Json -Depth 4)
    Move-Item -LiteralPath $tmp -Destination $file -Force
    return $file
}

# One match of Trinity (with $Values) against one opponent. Returns
# Trinity's wins, losses, games and the games lost on time.
function Invoke-TuneMatch($Fastchess, $Engine, $Values, $Opp, $Common) {
    $fcArgs = @("-engine", "cmd=$Engine", "name=Trinity")
    foreach ($k in $Values.Keys) { $fcArgs += "option.$k=$($Values[$k])" }
    $fcArgs += @("-engine", "cmd=$($Opp.Exe)", "name=$($Opp.Name)") + $Common
    $old = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try { $text = (& $Fastchess @fcArgs 2>&1 | Out-String) } finally { $ErrorActionPreference = $old }
    $m = [regex]::Matches($text, 'Games: (\d+), Wins: (\d+), Losses: (\d+), Draws: (\d+)')
    if ($m.Count -eq 0) {
        $tail = $text.Substring([Math]::Max(0, $text.Length - 1500))
        Stop-WithMessage "fastchess gave no result:`n$tail"
    }
    $g = $m[$m.Count - 1].Groups
    $onTime = [regex]::Matches($text, '(?m)^Finished game.*loses on time').Count
    return @{ Games = [int]$g[1].Value; Wins = [int]$g[2].Value; Losses = [int]$g[3].Value; OnTime = $onTime }
}

function Invoke-OpponentTune {
    param($Engine, [string]$Sha, [string]$Ref, $Opponents, $Fastchess, $Book, [string]$TC, [string]$SyzygyArg,
        [int]$Concurrency, [string]$Chain, [int]$Minutes)
    $settings = Get-TuneSettings $Sha
    $state = Read-TuneState $Chain $settings
    $n = $script:TuneTotalPairs
    $bigA = 0.1 * $n
    $pairs = [Math]::Max(1, $Concurrency)        # pairs per side per step: one game per slot, both colours
    $chainSeed = @{ A = 1; B = 2; C = 3; D = 4 }[$Chain]
    $rng = New-Object System.Random ($chainSeed * 1000003 + $state.steps)
    $common = @("-each", "proto=uci", "tc=$TC", "option.Hash=16", "option.Threads=1", $SyzygyArg,
        "-openings", "file=$Book", "format=pgn", "order=random",
        "-repeat", "-games", "2", "-rounds", "$pairs", "-concurrency", "$Concurrency",
        "-draw", "movenumber=40", "movecount=8", "score=10",
        "-resign", "movecount=3", "score=600", "twosided=true")
    $started = Get-Date
    $deadline = $started.AddMinutes($Minutes)
    $stepSeconds = 0.0
    $firstPairs = $state.pairs; $firstGames = $state.games
    Write-Step "Tuning chain $Chain of '$Ref' against $($Opponents.Count) rated opponents, $TC, $Minutes minutes (starting at $($state.pairs) of $n planned pairs)"
    while ((Get-Date).AddSeconds(1.5 * $stepSeconds) -lt $deadline) {
        $t0 = Get-Date
        $kk = $state.pairs + 1
        $plus = [ordered]@{}; $minus = [ordered]@{}; $flips = @{}; $ck = @{}
        foreach ($name in $settings.Keys) {
            $p = $settings[$name]
            $ck[$name] = $p.Step * [Math]::Pow($n / $kk, $script:TuneGamma)
            $flips[$name] = if ($rng.Next(2) -eq 0) { -1 } else { 1 }
            $up = $state.theta[$name] + $ck[$name] * $flips[$name]
            $down = $state.theta[$name] - $ck[$name] * $flips[$name]
            $plus[$name] = [int][Math]::Round([Math]::Min([Math]::Max($up, $p.Min), $p.Max))
            $minus[$name] = [int][Math]::Round([Math]::Min([Math]::Max($down, $p.Min), $p.Max))
        }
        $opp = $Opponents[$state.steps % $Opponents.Count]
        # The same seed: both versions get the same openings and colours.
        $seed = $chainSeed * 1000003 + $state.steps
        $srand = @("-srand", "$seed")
        $a = Invoke-TuneMatch $Fastchess $Engine $plus $opp ($common + $srand)
        $b = Invoke-TuneMatch $Fastchess $Engine $minus $opp ($common + $srand)
        $result = ($a.Wins - $a.Losses) - ($b.Wins - $b.Losses)
        foreach ($name in $settings.Keys) {
            $p = $settings[$name]
            $aEnd = $script:TuneREnd * $p.Step * $p.Step
            $ak = $aEnd * [Math]::Pow($bigA + $n, $script:TuneAlpha) / [Math]::Pow($bigA + $kk, $script:TuneAlpha)
            $v = $state.theta[$name] + $ak * $result * $flips[$name] / $ck[$name]
            $state.theta[$name] = [Math]::Min([Math]::Max($v, $p.Min), $p.Max)
        }
        $state.pairs += $pairs
        $state.steps += 1
        $state.games += $a.Games + $b.Games
        $state.timeLosses += $a.OnTime + $b.OnTime
        [void](Save-TuneState $state)
        $stepSeconds = ((Get-Date) - $t0).TotalSeconds
        $shown = ($settings.Keys | ForEach-Object { "{0}={1:0.0}" -f $_, $state.theta[$_] }) -join " "
        Write-Host ("step {0} vs {1}: up {2}-{3}, down {4}-{5}; {6}" -f $state.steps, $opp.Name,
            $a.Wins, $a.Losses, $b.Wins, $b.Losses, $shown)
    }

    $hours = [Math]::Round(((Get-Date) - $started).TotalHours, 1)
    $lines = @(
        "TUNE chain $Chain of '$Ref' (commit $($Sha.Substring(0, 10))) against the rated opponents, $TC",
        "Machine: $(Get-MachineLabel) ($Concurrency games at a time); $hours hours",
        "This job: $($state.games - $firstGames) games, $($state.pairs - $firstPairs) pairs per side; chain so far: $($state.pairs) of $n planned pairs, $($state.games) games, $($state.steps) steps",
        "Games lost on time (either side, whole chain): $($state.timeLosses)",
        "setting: default -> chain $Chain now"
    )
    foreach ($name in $settings.Keys) {
        $lines += ("{0}: {1} -> {2:0.0}" -f $name, $settings[$name].Default, $state.theta[$name])
    }
    $text = $lines -join "`r`n"
    $stamp = Get-Date -Format "yyyy-MM-dd_HH-mm"
    $local = Save-TuneState $state
    Set-Content -LiteralPath (Join-Path (Split-Path $local) "tune-$stamp-$Chain.txt") -Value $text
    if ($env:TRINITY_RESULTS_DIR) {
        Publish-ToResultsRepo "Tuning chain $Chain ($stamp): $($state.pairs) of $n pairs" @{
            "spsa/state-$Chain.json" = ($state | ConvertTo-Json -Depth 4)
            "spsa/tune-$stamp-$Chain.txt" = $text
        }
    }
    Write-Host ""
    Write-Host "==================================================================" -ForegroundColor Cyan
    foreach ($l in $lines) { Write-Host $l }
    Write-Host "==================================================================" -ForegroundColor Cyan
}
