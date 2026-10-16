# Held-out positions file (Coder 2, owner's decision of 16 October): samples
# 150,000 positions from Trinity's calibration games (ply 16 on, not in
# check, spread evenly), each with the game's result, writes them to
# sprt\coder2\results\heldout-positions.txt and commits that file to main
# as tools/net/data/heldout-positions.txt (about 10 MB). With it, Coder 2
# can score any network file in the cloud without PC time.
# A minute or two. Double-click tools\net\Export-Heldout.bat.
#
# Writes only inside sprt\coder2 and the one repository file. Deletes nothing.

param(
    [string]$Branch = "net-quantcheck",   # branch holding the `netloss` engine command
    [int]$Positions = 150000
)

. (Join-Path $PSScriptRoot "net-common.ps1")

try {
    Assert-Tools
    Invoke-Native git @("-C", $script:Root, "fetch", "--prune", "origin")
    $pgns = @(Find-Files (Join-Path (Join-Path $script:Root "sprt") "results") "*_calibration_*.pgn")
    if ($pgns.Count -eq 0) { Stop-WithMessage "No calibration PGN files found in sprt\results." }
    Write-Host "Game files: $($pgns.Count)"
    $exe = Build-NetEngine $Branch
    $file = Join-Path (Get-OutDir "results") "heldout-positions.txt"
    Write-Step "Sampling $Positions positions"
    $inputs = @($pgns | ForEach-Object { "--games=$_" })
    $old = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $output = & $exe (@("netloss", "$Positions", "--export=$file") + $inputs) 2>&1
    $code = $LASTEXITCODE
    $ErrorActionPreference = $old
    if ($code -ne 0) { Stop-WithMessage "export failed: $($output -join "`n")" }
    $output | ForEach-Object { Write-Host $_ }
    $lines = (Get-Content -LiteralPath $file | Measure-Object -Line).Lines
    $sizeMB = [Math]::Round((Get-Item -LiteralPath $file).Length / 1MB, 1)
    if ($lines -lt 1000) { Stop-WithMessage "Only $lines positions were written; not published." }
    $stamp = Get-Date -Format "yyyy-MM-dd"
    $commit = Publish-FileToBranch "main" "tools/net/data/heldout-positions.txt" $file `
        "Held-out positions: $lines from Trinity's calibration games up to $stamp, with results (Coder 2's loss checks)"
    Write-Host ""
    Write-Good "Done: $lines positions ($sizeMB MB) committed to main as tools/net/data/heldout-positions.txt (commit $($commit.Substring(0, 10))). Tell Coder 2: held-out file published."
} catch {
    Write-UnexpectedError $_
    exit 1
}
