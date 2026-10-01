# Generate NNUE training data by letting Trinity play itself.
# Double-click 3-Generate-Training-Data.bat. Output goes to data\.
#
# Rough guide: 14 threads produce very roughly 3-5 million positions per
# hour on an i9-10900KF, so the default 30 million is about one night. The
# window shows progress and an estimate of the time left.

param(
    [long]$Positions = 30000000,
    [int]$Threads = 0,           # 0 = automatic (14 on the main PC)
    [int]$Nodes = 5000,          # search effort per move (higher = better data, slower)
    [switch]$NoUpdate
)

. (Join-Path $PSScriptRoot "common.ps1")

$keepAwake = $null
try {
    Assert-Tools
    if (-not $NoUpdate) { Update-Repository }
    $enginePath = Build-Trinity "origin/main"
    if ($Threads -le 0) { $Threads = Get-DefaultConcurrency }

    $stamp = Get-Date -Format "yyyy-MM-dd_HH-mm"
    $out = Join-Path (Join-Path $script:Root "data") "gen_$stamp"
    Write-Step "Generating $Positions positions with $Threads threads into data\gen_$stamp"
    Write-Host "You can stop at any time with Ctrl+C; everything written so far is kept."
    $keepAwake = Start-KeepAwake
    Invoke-Native $enginePath @("datagen", "threads=$Threads", "positions=$Positions", "nodes=$Nodes", "out=$out")
    Write-Host ""
    Write-Good "Done. Next: double-click 4-Train-Network.bat"
} catch {
    Write-UnexpectedError $_
    exit 1
} finally {
    Stop-KeepAwake $keepAwake
}
