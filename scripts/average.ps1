# Weight averaging: two new networks made by averaging the last checkpoints
# of the 1024-neuron training run (no training, a few minutes). Publishes
# each as a branch for an SPRT test against main. Double-click
# 17-Weight-Average.bat.

param([switch]$NoUpdate)

. (Join-Path $PSScriptRoot "common.ps1")

try {
    Assert-Tools
    Invoke-Housekeeping
    if ($script:OnWindows -and -not $env:CUDA_PATH) {
        Stop-WithMessage "The NVIDIA CUDA Toolkit is not installed (or this window was opened before installing it)."
    }
    if (-not $NoUpdate) { Update-Repository }

    # The newest 1024-neuron run: folders named <run>-big1024-<superbatch>.
    $ckptRoot = Join-Path $script:Root "checkpoints"
    $dirs = @(Get-ChildItem -LiteralPath $ckptRoot -Directory | Where-Object { $_.Name -match '^(.*-big1024)-(\d+)$' })
    if ($dirs.Count -eq 0) { Stop-WithMessage "No 1024-neuron checkpoints found in $ckptRoot." }
    $run = ($dirs | ForEach-Object { ($_.Name -replace '-\d+$', '') } | Sort-Object | Select-Object -Last 1)
    Write-Host "Training run: $run"
    $variants = @(
        @{ Label = "550-600"; Steps = @(550, 600) },
        @{ Label = "450-600"; Steps = @(450, 500, 550, 600) }
    )
    foreach ($v in $variants) {
        foreach ($s in $v.Steps) {
            $d = Join-Path $ckptRoot "$run-$s"
            if (-not (Test-Path -LiteralPath (Join-Path (Join-Path $d "optimiser_state") "weights.bin"))) {
                Stop-WithMessage "Checkpoint $d is missing (or has no optimiser_state\weights.bin)."
            }
        }
    }

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
    $branches = @()
    $env:TRINITY_HIDDEN = "1024"
    try {
        foreach ($v in $variants) {
            $out = Join-Path $ckptRoot "$run-avg-$($v.Label).nnue"
            $dirsArg = @($v.Steps | ForEach-Object { Join-Path $ckptRoot "$run-$_" })
            Write-Step "Averaging checkpoints $($v.Steps -join ', ')"
            Invoke-Native $trainer (@("average", $out) + $dirsArg)
            $size = (Get-Item -LiteralPath $out).Length
            # 12617744 bytes of weights, padded to a multiple of 64 (as main's network is).
            if ($size -lt 12617744 -or $size -ge 12617744 + 64) { Stop-WithMessage "$out has an unexpected size ($size bytes)." }

            $main = Join-Path (Join-Path $script:Root "nets") "default.nnue"
            $hashOut = (Get-FileHash -LiteralPath $out -Algorithm SHA256).Hash
            $hashMain = (Get-FileHash -LiteralPath $main -Algorithm SHA256).Hash
            Write-Host "Fingerprint of the average: $($hashOut.Substring(0, 16)); of main's network: $($hashMain.Substring(0, 16))"
            if ($hashOut -eq $hashMain) {
                Write-Warn "The average $($v.Label) is identical to main's network; not published."
                continue
            }
            $branch = "net-avg-$($v.Label)-$stamp"
            $tree = Join-Path $script:Work "publish-$branch"
            Invoke-Native git @("-C", $script:Root, "fetch", "origin")
            Invoke-Native git @("-C", $script:Root, "worktree", "add", "-B", $branch, $tree, "origin/main")
            try {
                Copy-Item $out (Join-Path (Join-Path $tree "nets") "default.nnue") -Force
                Invoke-Native git @("-C", $tree, "add", "nets/default.nnue")
                Invoke-Native git ($script:CommitIdentity + @("-C", $tree, "commit", "-m",
                        "Network: average of checkpoints $($v.Steps -join ', ') of $run (weight averaging, no training)"))
                Invoke-Native git @("-C", $tree, "push", "-u", "origin", $branch)
            } finally {
                Invoke-Native git @("-C", $script:Root, "worktree", "remove", "--force", $tree)
            }
            $branches += $branch
        }
    } finally {
        Remove-Item Env:TRINITY_HIDDEN -ErrorAction SilentlyContinue
    }
    Write-Host ""
    Write-Good "Done. Tell Claude: weight averaging done, branches: $($branches -join ' and ') (send a screenshot of this window)."
} catch {
    Write-UnexpectedError $_
    exit 1
}
