# Loss of the saved checkpoints (Coder 2): scores every checkpoint of the
# 1024 run (C:\Trinity\checkpoints) and of Coder 2's runs
# (sprt\coder2\checkpoints) on the same positions, so the two runs can be
# compared at the same superbatch. Two kinds of positions:
# - held-out: Trinity's own calibration games (sprt\results\*_calibration_*.pgn),
#   label = game result; no network was trained on them (the fair test);
# - training: the first positions of sprt\coder2\data\big-1.data, if it
#   exists (fit only; the gap to the held-out number is the memorisation).
# About a minute. Double-click tools\net\Net-Loss.bat.
#
# Writes only inside sprt\coder2 (the branch's build and
# results\netloss-<date>.txt). Deletes nothing.
#
# The engine command `trinity netloss` lives on the branch net-quantcheck.

param(
    [string]$Branch = "net-quantcheck",
    [int]$Positions = 1000000,   # per kind of position
    [string]$Data = ""           # shuffled training file; default: sprt\coder2\data\big-1.data if it exists
)

. (Join-Path $PSScriptRoot "net-common.ps1")

try {
    Assert-Tools
    Invoke-Native git @("-C", $script:Root, "fetch", "--prune", "origin")

    # Held-out positions: Trinity's calibration games.
    $pgns = @(Find-Files (Join-Path (Join-Path $script:Root "sprt") "results") "*_calibration_*.pgn")
    if ($pgns.Count -eq 0) { $pgns = @(Find-Files (Join-Path $script:Root "games") "*.pgn") }
    if ($pgns.Count -eq 0) { Stop-WithMessage "No PGN files found in sprt\results (calibration games) or games\." }
    Write-Host "Held-out positions: up to $Positions from $($pgns.Count) game files"
    # Training positions, if the data is there (optional).
    if ($Data -eq "") {
        $candidate = Join-Path (Join-Path $script:Out "data") "big-1.data"
        if (Test-Path -LiteralPath $candidate) { $Data = $candidate }
    }
    $inputs = @($pgns | ForEach-Object { "--games=$_" })
    if ($Data -ne "") {
        if (-not (Test-Path -LiteralPath $Data)) { Stop-WithMessage "$Data does not exist." }
        Write-Host "Training positions: the first $Positions of $Data"
        $inputs += "--train=$Data"
    } else {
        Write-Warn "No training data file found (sprt\coder2\data\big-1.data); held-out positions only."
    }

    # Every checkpoint folder of every run, sorted by run and superbatch.
    $dirs = @()
    foreach ($root in @((Join-Path $script:Root "checkpoints"), (Join-Path $script:Out "checkpoints"))) {
        foreach ($w in (Find-Files $root "weights.bin")) {
            $d = Split-Path (Split-Path $w -Parent) -Parent
            if ((Split-Path (Split-Path $w -Parent) -Leaf) -eq "optimiser_state") { $dirs += $d }
        }
    }
    $dirs = @($dirs | Sort-Object { ($_ -replace '-\d+$', '') }, { [int](($_ -replace '.*-', '')) })
    if ($dirs.Count -eq 0) { Stop-WithMessage "No checkpoints with optimiser_state\weights.bin found." }
    Write-Host "Checkpoints: $($dirs.Count)"

    $exe = Build-NetEngine $Branch
    Write-Step "Scoring $($dirs.Count) checkpoints (a minute or two)"
    $stamp = Get-Date -Format "yyyy-MM-dd_HH-mm"
    $resultFile = Join-Path (Get-OutDir "results") "netloss-$stamp.txt"
    $old = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $output = & $exe (@("netloss", "$Positions") + $dirs + $inputs) 2>&1
    $code = $LASTEXITCODE
    $ErrorActionPreference = $old
    if ($code -ne 0) { Stop-WithMessage "netloss failed: $($output -join "`n")" }
    $text = @("Network loss check of $stamp (branch $Branch, engine $(Split-Path $exe -Leaf))") + $output
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
