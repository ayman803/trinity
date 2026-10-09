# Big network: a 1024-neuron king-bucket network trained from scratch on
# all the Leela data we can convert (up to 4 x 1 billion positions), in one
# long run with a smoothly falling learning rate. Publishes a net-big-...
# branch for an SPRT test. Double-click 11-Big-Network.bat (runs overnight).
#
# Disk: each billion positions takes 32 GB, plus 32 GB while it is
# shuffled. The number of parts is lowered automatically if space is short.

param(
    [int]$Chunks = 4,
    [long]$PerChunk = 1000000000,
    [int]$Superbatches = 600,
    [string]$LearningRate = "0.001",
    [string]$Wdl = "0.5",
    [int]$Hidden = 1024,
    [int]$MemoryMB = 8000
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
    New-Item -ItemType Directory -Force -Path $prepared | Out-Null
    $binpacks = @(Find-Files (Join-Path (Join-Path $script:Root "data") "leela") "*.binpack")
    if ($binpacks.Count -ne 1) { Stop-WithMessage "Expected exactly one .binpack file in data\leela (found $($binpacks.Count))." }

    # Earlier Leela blocks are not needed any more (this run converts the
    # file from the start); free their space.
    foreach ($old in @("leela-fresh.data", "leela-shuffled.data")) {
        $f = Join-Path $prepared $old
        if (Test-Path -LiteralPath $f) { Write-Host "Removing $f (no longer needed)"; Remove-Item -LiteralPath $f }
    }
    $parts = @(1..$Chunks | ForEach-Object { Join-Path $prepared "big-$_.data" })
    $ready = @($parts | Where-Object { Test-Path -LiteralPath $_ })
    $drive = [IO.DriveInfo]::new([IO.Path]::GetPathRoot($script:Root))
    $freeGB = [Math]::Floor($drive.AvailableFreeSpace / 1GB)
    if ($ready.Count -lt $Chunks) {
        # Need 32 GB per part plus 32 GB for shuffling, plus 20 GB spare.
        $fit = [Math]::Floor(($freeGB - 52) / 32)
        if ($fit -lt $Chunks) {
            if ($fit -lt 2) { Stop-WithMessage "Only $freeGB GB free; at least about 116 GB are needed. Free some space and try again." }
            Write-Warn "Only $freeGB GB free: converting $fit billion positions instead of $Chunks."
            $Chunks = [int]$fit
            $parts = $parts[0..($Chunks - 1)]
        }
    }
    $keepAwake = Start-KeepAwake

    # 1. Build the trainer from main.
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
    $utils = Join-Path (Join-Path (Join-Path (Join-Path $script:Work "tools") "bullet") "bin") "bullet-utils$($script:ExeSuffix)"

    # 2. Convert (one pass over the Leela file), then shuffle each part.
    #    Parts already made by an earlier, interrupted run are kept.
    if (@($parts | Where-Object { -not (Test-Path -LiteralPath $_) }).Count -gt 0) {
        $prefix = Join-Path $prepared "big"
        Write-Step "Converting up to $Chunks x $PerChunk Leela positions (about 20 minutes per billion)"
        Invoke-Native $trainer @("convert-chunks", $binpacks[0], $prefix, "$Chunks", "$PerChunk")
        foreach ($i in 1..$Chunks) {
            $raw = "$prefix-$i.raw"
            $out = $parts[$i - 1]
            if (-not (Test-Path -LiteralPath $raw)) { continue }
            if ((Get-Item -LiteralPath $raw).Length -lt 32MB) { Remove-Item -LiteralPath $raw; continue }
            Write-Step "Shuffling part $i of $Chunks"
            Invoke-Native $utils @("validate", "--input", $raw)
            Invoke-Native $utils @("shuffle", "--input", $raw, "--output", $out, "--mem-used-mb", "$MemoryMB")
            Remove-Item -LiteralPath $raw
            Show-DiskSpace
        }
    }
    $files = @($parts | Where-Object { Test-Path -LiteralPath $_ })
    if ($files.Count -eq 0) { Stop-WithMessage "No converted data was produced." }
    $positions = [Math]::Round((($files | ForEach-Object { (Get-Item -LiteralPath $_).Length } | Measure-Object -Sum).Sum) / 32 / 1e9, 2)

    # 3. Train from scratch.
    $stamp = Get-Date -Format "yyyyMMdd-HHmm"
    $netId = "trinity-$stamp-big$Hidden"
    Write-Step "Training $netId from scratch: $Hidden neurons, $Superbatches superbatches on $positions billion positions (several hours)"
    $env:TRINITY_HIDDEN = "$Hidden"
    Push-Location $script:Root
    try {
        Invoke-Native $trainer (@("scratch", "$Superbatches", $netId, $LearningRate, $Wdl) + $files)
    } finally {
        Pop-Location
        Remove-Item Env:TRINITY_HIDDEN -ErrorAction SilentlyContinue
    }
    $final = Join-Path (Join-Path (Join-Path $script:Root "checkpoints") "$netId-$Superbatches") "quantised.bin"
    if (-not (Test-Path $final)) { Stop-WithMessage "Training finished but $final was not found." }

    # 4. Publish on a new branch so it can be SPRT-tested against main.
    $netBranch = "net-big-$stamp"
    Write-Step "Publishing the network on GitHub as branch '$netBranch'"
    $tree = Join-Path $script:Work "publish-$stamp"
    Invoke-Native git @("-C", $script:Root, "fetch", "origin")
    Invoke-Native git @("-C", $script:Root, "worktree", "add", "-B", $netBranch, $tree, "origin/main")
    try {
        Copy-Item $final (Join-Path (Join-Path $tree "nets") "default.nnue") -Force
        # Fixed identity: keeps personal e-mail addresses out of the public history.
        $identity = $script:CommitIdentity
        Invoke-Native git @("-C", $tree, "add", "nets/default.nnue")
        Invoke-Native git ($identity + @("-C", $tree, "commit", "-m",
                "Network $netId ($Hidden neurons, king buckets; from scratch, $Superbatches superbatches on $positions billion Leela positions, LR $LearningRate cosine, WDL $Wdl)"))
        Invoke-Native git @("-C", $tree, "push", "-u", "origin", $netBranch)
    } finally {
        Invoke-Native git @("-C", $script:Root, "worktree", "remove", "--force", $tree)
    }
    Write-Host ""
    Write-Good "Done. Tell Claude: big network done, branch '$netBranch'."
} catch {
    Write-UnexpectedError $_
    exit 1
} finally {
    Stop-KeepAwake $keepAwake
}
