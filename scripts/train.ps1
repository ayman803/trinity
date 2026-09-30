# Train a new NNUE network on the GPU with bullet, then publish it on a new
# branch so it can be SPRT-tested against the current version.
# Double-click 4-Train-Network.bat.
#
# Needs: an NVIDIA GPU and the CUDA Toolkit (see GUIDE.md).

param(
    [int]$Superbatches = 40,     # length of training (1 superbatch = ~100M positions)
    [int]$MemoryMB = 8000,       # RAM used for shuffling
    [switch]$SkipShuffle,        # reuse data\prepared\shuffled.data from last time
    [switch]$NoPublish           # don't create a GitHub branch at the end
)

. (Join-Path $PSScriptRoot "common.ps1")

$keepAwake = $null
try {
    Assert-Tools
    if ($script:OnWindows -and -not $env:CUDA_PATH) {
        Stop-WithMessage "The NVIDIA CUDA Toolkit is not installed (or this window was opened before installing it). See GUIDE.md, 'Training a network'."
    }
    $dataDir = Join-Path $script:Root "data"
    $prepared = Join-Path $dataDir "prepared"
    $shuffled = Join-Path $prepared "shuffled.data"
    New-Item -ItemType Directory -Force -Path $prepared | Out-Null
    $keepAwake = Start-KeepAwake

    # bullet-utils: bullet's data tools (shuffle, interleave, validate).
    $toolsRoot = Join-Path (Join-Path $script:Work "tools") "bullet"
    $utils = Join-Path (Join-Path $toolsRoot "bin") "bullet-utils$($script:Exe)"
    if (-not (Test-Path $utils)) {
        Write-Step "Building bullet's data tools (one time)"
        Invoke-Native cargo @("install", "--locked", "--git", "https://github.com/jw1912/bullet", "--rev", $script:BulletRev,
            "bullet-utils", "--root", $toolsRoot)
    }

    if (-not ($SkipShuffle -and (Test-Path $shuffled))) {
        $files = Get-ChildItem -Path $dataDir -Recurse -Filter "*.data" |
            Where-Object { $_.FullName -notlike "$prepared*" -and $_.Length -gt 0 } |
            ForEach-Object { $_.FullName }
        if (-not $files) { Stop-WithMessage "No training data found in data\. Run 3-Generate-Training-Data.bat first." }
        $total = ($files | ForEach-Object { (Get-Item $_).Length } | Measure-Object -Sum).Sum
        Write-Step ("Preparing {0:N0} positions from {1} files" -f ($total / 32), @($files).Count)
        $combined = Join-Path $prepared "combined.data"
        Invoke-Native $utils (@("interleave") + $files + @("--output", $combined))
        Invoke-Native $utils @("validate", "--input", $combined)
        if (Test-Path $shuffled) { Remove-Item $shuffled }
        Invoke-Native $utils @("shuffle", "--input", $combined, "--output", $shuffled, "--mem-used-mb", "$MemoryMB")
        Remove-Item $combined
    }

    Write-Step "Building the trainer (first time takes a few minutes)"
    $features = if ($script:OnWindows) { @("--features", "cuda") } else { @() }
    Invoke-Native cargo (@("build", "--release", "--manifest-path", (Join-Path (Join-Path $script:Root "trainer") "Cargo.toml")) + $features)
    $trainer = Join-Path (Join-Path (Join-Path (Join-Path $script:Root "trainer") "target") "release") "trinity-trainer$($script:Exe)"

    $stamp = Get-Date -Format "yyyyMMdd-HHmm"
    $netId = "trinity-$stamp"
    Write-Step "Training network $netId ($Superbatches superbatches)"
    Push-Location $script:Root
    try {
        Invoke-Native $trainer @($shuffled, "$Superbatches", $netId)
    } finally {
        Pop-Location
    }

    $final = Join-Path (Join-Path (Join-Path $script:Root "checkpoints") "$netId-$Superbatches") "quantised.bin"
    if (-not (Test-Path $final)) { Stop-WithMessage "Training finished but $final was not found." }
    Write-Good "Network trained: $final"

    if ($NoPublish) { return }

    # Publish on a new branch (main is not touched) so it can be SPRT-tested.
    $branch = "net-$stamp"
    Write-Step "Publishing the network on GitHub as branch '$branch'"
    $tree = Join-Path $script:Work "publish-$stamp"
    Invoke-Native git @("-C", $script:Root, "fetch", "origin")
    Invoke-Native git @("-C", $script:Root, "worktree", "add", "-b", $branch, $tree, "origin/main")
    try {
        $nets = Join-Path $tree "nets"
        New-Item -ItemType Directory -Force -Path $nets | Out-Null
        Copy-Item $final (Join-Path $nets "default.nnue") -Force
        $identity = @()
        if (-not (Get-NativeOutput git @("-C", $tree, "config", "--default", "", "user.email"))) {
            $identity = @("-c", "user.name=Trinity trainer", "-c", "user.email=trainer@trinity.invalid")
        }
        Invoke-Native git @("-C", $tree, "add", "nets/default.nnue")
        Invoke-Native git ($identity + @("-C", $tree, "commit", "-m", "New network $netId ($Superbatches superbatches)"))
        Invoke-Native git @("-C", $tree, "push", "-u", "origin", $branch)
    } finally {
        Invoke-Native git @("-C", $script:Root, "worktree", "remove", "--force", $tree)
    }
    Write-Host ""
    Write-Good "Done. Now double-click 2-Run-SPRT-Test.bat and choose '$branch'."
} catch {
    if ($_.Exception.Message -ne "stopped") { Write-Host "Unexpected error: $($_.Exception.Message)" -ForegroundColor Red }
    exit 1
} finally {
    Stop-KeepAwake $keepAwake
}
