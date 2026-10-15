# Quantisation check (Coder 2, Scout's idea of 14 October): how far the
# integer network built into the engine (nets\default.nnue) drifts from the
# float network it was rounded from (checkpoints\<run>\optimiser_state\
# weights.bin), on positions from Trinity's own calibration games.
# A few CPU minutes. Double-click tools\net\Quant-Check.bat.
#
# Writes only inside sprt\coder2 (the branch's source and build, and
# results\quantcheck-<date>.txt). Deletes nothing.
#
# The engine command `trinity quantcheck` lives on the branch
# net-quantcheck (engine code stays off main until the Session merges it);
# this script builds that branch and runs it against this PC's files.

param(
    [string]$Branch = "net-quantcheck",   # branch holding the `quantcheck` engine command
    [int]$Positions = 20000,
    [string]$Checkpoint = ""              # checkpoint folder; default: the one main's network came from
)

. (Join-Path $PSScriptRoot "net-common.ps1")

try {
    Assert-Tools
    Show-DiskSpace
    Invoke-Native git @("-C", $script:Root, "fetch", "--prune", "origin")

    # 1. The float checkpoint: the folder whose quantised.bin is main's network.
    $ckptRoot = Join-Path $script:Root "checkpoints"
    if ($Checkpoint -eq "") {
        $Checkpoint = Find-NetCheckpoint (Join-Path (Join-Path $script:Root "nets") "default.nnue")
        if (-not $Checkpoint) {
            $dirs = @(Get-ChildItem -LiteralPath $ckptRoot -Directory -ErrorAction SilentlyContinue | Where-Object { $_.Name -match '-big1024-600$' })
            if ($dirs.Count -gt 0) {
                $Checkpoint = ($dirs | Sort-Object LastWriteTime | Select-Object -Last 1).FullName
                Write-Warn "No checkpoint's quantised.bin matches nets\default.nnue; using the newest big1024-600 folder instead."
            }
        }
    }
    if (-not $Checkpoint) {
        $names = @(Get-ChildItem -LiteralPath $ckptRoot -Directory -ErrorAction SilentlyContinue | ForEach-Object { $_.Name })
        Stop-WithMessage "No checkpoint found for main's network. Folders in checkpoints\: $($names -join ', ')"
    }
    $weights = Join-Path (Join-Path $Checkpoint "optimiser_state") "weights.bin"
    if (-not (Test-Path -LiteralPath $weights)) { Stop-WithMessage "$weights is missing." }
    Write-Host "Float checkpoint: $Checkpoint"

    # 2. Training-speed record: when each checkpoint of that run was written
    #    (needed to plan the next run's length).
    $runName = (Split-Path $Checkpoint -Leaf) -replace '-\d+$', ''
    $siblings = @(Get-ChildItem -LiteralPath (Split-Path $Checkpoint -Parent) -Directory |
        Where-Object { $_.Name -match "^$([regex]::Escape($runName))-(\d+)$" } |
        Sort-Object { [int]($_.Name -replace '.*-', '') })
    $timeLines = @("Checkpoints of run ${runName}:")
    foreach ($s in $siblings) {
        $q = Join-Path $s.FullName "quantised.bin"
        $when = if (Test-Path -LiteralPath $q) { (Get-Item -LiteralPath $q).LastWriteTime } else { $s.LastWriteTime }
        $timeLines += "  superbatch $($s.Name -replace '.*-', '') written $($when.ToString('yyyy-MM-dd HH:mm'))"
    }

    # 3. Positions: Trinity's calibration games against rated engines.
    $pgns = @(Find-Files (Join-Path (Join-Path $script:Root "sprt") "results") "*_calibration_*.pgn")
    if ($pgns.Count -eq 0) { $pgns = @(Find-Files (Join-Path $script:Root "games") "*.pgn") }
    if ($pgns.Count -eq 0) { Stop-WithMessage "No PGN files found in sprt\results (calibration games) or games\." }
    Write-Host "Game files: $($pgns.Count)"

    # 4. Build the engine from the branch (its network is main's) and run.
    $exe = Build-NetEngine $Branch
    Write-Step "Comparing the integer and float networks on up to $Positions positions"
    $stamp = Get-Date -Format "yyyy-MM-dd_HH-mm"
    $resultFile = Join-Path (Get-OutDir "results") "quantcheck-$stamp.txt"
    $old = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $output = & $exe (@("quantcheck", $weights, "$Positions") + $pgns) 2>&1
    $code = $LASTEXITCODE
    $ErrorActionPreference = $old
    if ($code -ne 0) { Stop-WithMessage "quantcheck failed: $($output -join "`n")" }
    $text = @("Quantisation check of $stamp (branch $Branch, engine $(Split-Path $exe -Leaf))") + $output + $timeLines
    Set-Content -Path $resultFile -Value ($text -join "`n")
    Write-Host ""
    Write-Host "---------------- copy from here ----------------" -ForegroundColor Cyan
    $text | ForEach-Object { Write-Host $_ }
    Write-Host "---------------- to here ----------------" -ForegroundColor Cyan
    Write-Good "Done. Paste the text between the lines to Coder 2. (Also saved in $resultFile)"
} catch {
    Write-UnexpectedError $_
    exit 1
}
