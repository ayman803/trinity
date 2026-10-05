# Drill training: refine the current network on the positions where it
# disagrees most with Leela (quiet positions only, where Leela's score
# agrees with the game result), mixed with as many ordinary positions.
# Publishes the result on a new net-drill-... branch for an SPRT test.
# Double-click 7-Drill-Network.bat.
#
# Needs: the converted Leela data (data\prepared\leela-shuffled.data, made by
# 4-Train-Network.bat) and the checkpoint of the network that is in main.

param(
    [double]$Fraction = 0.2,      # share of the filtered positions to drill
    [int]$Superbatches = 5,       # length of the drill stage (1 superbatch = ~100M positions)
    [string]$LearningRate = "0.0001",
    [int]$MemoryMB = 8000         # RAM used for shuffling
)

. (Join-Path $PSScriptRoot "common.ps1")

$keepAwake = $null
try {
    Assert-Tools
    Invoke-Housekeeping -Training
    if ($script:OnWindows -and -not $env:CUDA_PATH) {
        Stop-WithMessage "The NVIDIA CUDA Toolkit is not installed (or this window was opened before installing it)."
    }
    Invoke-Native git @("-C", $script:Root, "fetch", "--prune", "origin")
    $prepared = Join-Path (Join-Path $script:Root "data") "prepared"
    $leelaData = Join-Path $prepared "leela-shuffled.data"
    $freshBlock = Join-Path $prepared "leela-fresh.data"
    if (Test-Path -LiteralPath $freshBlock) { $leelaData = $freshBlock }
    if (-not (Test-Path -LiteralPath $leelaData)) {
        Stop-WithMessage "Converted Leela data not found ($leelaData). Run 4-Train-Network.bat with Leela data first."
    }

    # The checkpoint whose network is the one in main.
    $mainNet = Join-Path (Join-Path $script:Root "nets") "default.nnue"
    $mainHash = (Get-FileHash -LiteralPath $mainNet -Algorithm SHA256).Hash
    $checkpoint = $null
    foreach ($q in (Find-Files (Join-Path $script:Root "checkpoints") "quantised.bin")) {
        if ((Get-FileHash -LiteralPath $q -Algorithm SHA256).Hash -eq $mainHash) { $checkpoint = Split-Path $q -Parent }
    }
    if (-not $checkpoint) {
        Stop-WithMessage "No checkpoint matches the network in main (nets\default.nnue). Is this the PC that trained it?"
    }
    Write-Host "Starting from checkpoint $checkpoint"
    $keepAwake = Start-KeepAwake

    # 1. Select the drill positions with the engine (uses main's network).
    $engine = Build-Trinity "origin/main"
    $mix = Join-Path $prepared "drill-mix.data"
    $drillData = Join-Path $prepared "drill-shuffled.data"
    Write-Step "Selecting the positions Trinity gets most wrong (about 10-20 minutes)"
    Invoke-Native $engine @("select", $leelaData, $mix, "$Fraction")

    $utils = Join-Path (Join-Path (Join-Path (Join-Path $script:Work "tools") "bullet") "bin") "bullet-utils$($script:ExeSuffix)"
    if (Test-Path $drillData) { Remove-Item $drillData }
    Invoke-Native $utils @("shuffle", "--input", $mix, "--output", $drillData, "--mem-used-mb", "$MemoryMB")
    Remove-Item $mix

    # 2. Build the trainer from main and refine the network.
    Write-Step "Building the trainer"
    $features = if ($script:OnWindows) { @("--features", "cuda") } else { @() }
    $codeTree = Join-Path $script:Work "trainer-src"
    if (Test-Path $codeTree) { Invoke-Native git @("-C", $script:Root, "worktree", "remove", "--force", $codeTree) }
    Invoke-Native git @("-C", $script:Root, "worktree", "prune")
    Invoke-Native git @("-C", $script:Root, "worktree", "add", "--detach", $codeTree, "origin/main")
    try {
        Invoke-Native cargo (@("build", "--release", "--manifest-path", (Join-Path (Join-Path $codeTree "trainer") "Cargo.toml"),
                "--target-dir", (Join-Path (Join-Path $script:Root "trainer") "target")) + $features)
    } finally {
        Invoke-Native git @("-C", $script:Root, "worktree", "remove", "--force", $codeTree)
    }
    $trainer = Join-Path (Join-Path (Join-Path (Join-Path $script:Root "trainer") "target") "release") "trinity-trainer$($script:ExeSuffix)"

    $stamp = Get-Date -Format "yyyyMMdd-HHmm"
    $netId = "trinity-$stamp-drill"
    Write-Step "Drill training $netId ($Superbatches superbatches)"
    Push-Location $script:Root
    try {
        Invoke-Native $trainer @("finetune", $checkpoint, "$Superbatches", $netId, $drillData, $LearningRate)
    } finally {
        Pop-Location
    }
    $final = Join-Path (Join-Path (Join-Path $script:Root "checkpoints") "$netId-$Superbatches") "quantised.bin"
    if (-not (Test-Path $final)) { Stop-WithMessage "Training finished but $final was not found." }

    # 3. Publish on a new branch so it can be SPRT-tested against main.
    $netBranch = "net-drill-$stamp"
    Write-Step "Publishing the network on GitHub as branch '$netBranch'"
    $tree = Join-Path $script:Work "publish-$stamp"
    Invoke-Native git @("-C", $script:Root, "fetch", "origin")
    Invoke-Native git @("-C", $script:Root, "worktree", "add", "-B", $netBranch, $tree, "origin/main")
    try {
        Copy-Item $final (Join-Path (Join-Path $tree "nets") "default.nnue") -Force
        $identity = @()
        $old = $ErrorActionPreference; $ErrorActionPreference = "Continue"
        $email = & git -C $tree config user.email 2>$null
        $ErrorActionPreference = $old
        if (-not $email) {
            $identity = @("-c", "user.name=Trinity trainer", "-c", "user.email=trainer@trinity.invalid")
        }
        Invoke-Native git @("-C", $tree, "add", "nets/default.nnue")
        Invoke-Native git ($identity + @("-C", $tree, "commit", "-m",
                "Drill network $netId (main's network + $Superbatches superbatches on its biggest quiet disagreements with Leela)"))
        Invoke-Native git @("-C", $tree, "push", "-u", "origin", $netBranch)
    } finally {
        Invoke-Native git @("-C", $script:Root, "worktree", "remove", "--force", $tree)
    }
    Write-Host ""
    Write-Good "Done. Now double-click 2-Run-SPRT-Test.bat and choose '$netBranch'."
} catch {
    Write-UnexpectedError $_
    exit 1
} finally {
    Stop-KeepAwake $keepAwake
}
