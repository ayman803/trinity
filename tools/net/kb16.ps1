# 16 king buckets (owner's decision, 18 October 2026; NETWORK.md section 6):
# main's 1024 recipe EXACTLY as it was (from scratch, 600 superbatches, LR
# 0.001 cosine, WDL 0.5, batch 16384, 1024 neurons, 8 output buckets, the
# same 3.65 billion positions in sprt\coder2\data\big-1..4.data) with ONE
# change: 16 king buckets instead of 8 (layout in src/nnue.rs on branch
# net-kb16). Publishes a net-kb16-<date> branch for an SPRT test.
# Double-click tools\net\KB16-Network.bat (dry run first:
# tools\net\Dry-Run-KB16.bat, about 5 minutes).
# The same script makes the CONTROL run (advisor and owner, 18 October):
# -KingBuckets 8 -Seed <n> -Tag control is main's recipe unchanged with a
# new random start, on the same data files as the 16-bucket run, to
# measure how much two from-scratch runs differ
# (tools\net\Control-Network.bat).
#
# Writes only inside sprt\coder2: checkpoints\<name>-<n>, results\
# (train-<name>.log, kb16-<date>.txt), the branch's source and builds.
# Deletes one thing: its own dry-run folder sprt\coder2\dry-kb16 before a
# new dry run. Converts nothing (the data is already there). Publishing
# writes git objects into C:\Trinity\.git and pushes a new branch.

param(
    [int]$Hidden = 1024,
    [string]$Branch = "net-kb16",     # engine and trainer with the 16-bucket layout
    [ValidateSet(8, 16)][int]$KingBuckets = 16,
    [string]$Seed = "",               # weight seed; empty = bullet's default (as main's network)
    [string]$Tag = "kb16",            # name of the run and of the published branch (net-<tag>-<date>)
    [int]$Superbatches = 600,
    [string]$LearningRate = "0.001",
    [string]$Wdl = "0.5",
    [switch]$DryRun                   # 1 superbatch, save, check the file loads in the engine; publishes nothing
)

. (Join-Path $PSScriptRoot "net-common.ps1")

$keepAwake = $null
try {
    Assert-Tools
    Show-DiskSpace
    if ($script:OnWindows -and -not $env:CUDA_PATH) {
        Stop-WithMessage "The NVIDIA CUDA Toolkit is not installed (or this window was opened before installing it)."
    }
    Invoke-Native git @("-C", $script:Root, "fetch", "--prune", "origin")
    $dataDir = Get-OutDir "data"
    $workDir = $script:Out
    if ($DryRun) {
        $workDir = Join-Path $script:Out "dry-$Tag"
        if (Test-Path -LiteralPath $workDir) { Remove-Item -LiteralPath $workDir -Recurse -Force }
        New-Item -ItemType Directory -Force -Path $workDir | Out-Null
        $Superbatches = 1
        Write-Step "DRY RUN: train 1 superbatch with $KingBuckets king buckets, save, load it in the engine; nothing is published"
    }
    # The same converted positions main's network was trained on (made by Wide-Network.bat).
    $files = @(1..4 | ForEach-Object { Join-Path $dataDir "big-$_.data" } | Where-Object { Test-Path -LiteralPath $_ })
    if ($files.Count -lt 4) { Stop-WithMessage "Expected the 4 converted parts big-1.data .. big-4.data in sprt\coder2\data (found $($files.Count))." }
    if ($DryRun) { $files = @($files[0]) }
    $drive = [IO.DriveInfo]::new([IO.Path]::GetPathRoot($script:Root))
    $freeGB = [Math]::Floor($drive.AvailableFreeSpace / 1GB)
    if ($freeGB -lt 10) { Stop-WithMessage "Only $freeGB GB free; the checkpoints need about 3 GB and 10 GB are asked for as spare." }
    $positions = [Math]::Round((($files | ForEach-Object { (Get-Item -LiteralPath $_).Length } | Measure-Object -Sum).Sum) / 32 / 1e9, 2)
    $keepAwake = Start-KeepAwake
    $started = Get-Date

    # 1. Build the trainer from the branch.
    $trainer = Build-NetTrainer $Branch

    # 2. Train from scratch with 16 king buckets, keeping the trainer's output.
    $stamp = Get-Date -Format "yyyyMMdd-HHmm"
    $netId = "trinity-$stamp-$Tag"
    $resultsDir = Get-OutDir "results"
    $log = Join-Path $resultsDir "train-$netId.log"
    Write-Step "Training $netId from scratch: $Hidden neurons, $KingBuckets king buckets, $Superbatches superbatches on $positions billion positions (16 buckets: about 7 hours; 8 buckets: about 5)"
    Write-Host "Trainer output is also saved in $log"
    $env:TRINITY_HIDDEN = "$Hidden"
    $env:TRINITY_KING_BUCKETS = "$KingBuckets"
    if ($Seed) { $env:TRINITY_SEED = $Seed }
    Push-Location $workDir
    try {
        $old = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        & $trainer (@("scratch", "$Superbatches", $netId, $LearningRate, $Wdl) + $files) 2>&1 | Tee-Object -FilePath $log | Out-Host
        $code = $LASTEXITCODE
        $ErrorActionPreference = $old
        if ($code -ne 0) { Stop-WithMessage "The trainer failed (exit code $code). See $log" }
    } finally {
        Pop-Location
        Remove-Item Env:TRINITY_HIDDEN -ErrorAction SilentlyContinue
        Remove-Item Env:TRINITY_KING_BUCKETS -ErrorAction SilentlyContinue
        Remove-Item Env:TRINITY_SEED -ErrorAction SilentlyContinue
    }
    $final = Join-Path (Join-Path (Join-Path $workDir "checkpoints") "$netId-$Superbatches") "quantised.bin"
    if (-not (Test-Path $final)) { Stop-WithMessage "Training finished but $final was not found." }
    $trained = Get-Date
    $expected = 2 * ($KingBuckets * 768 * $Hidden + $Hidden + 8 * (2 * $Hidden + 1))
    $size = (Get-Item -LiteralPath $final).Length
    if ($size -lt $expected -or $size -ge $expected + 64) { Stop-WithMessage "The saved network has $size bytes; a $Hidden-neuron network with $KingBuckets king buckets has $expected (plus up to 63 bytes of padding)." }

    # 3. Load it in the engine of the branch (it picks the 16-bucket layout
    #    from the file's size) and run the bench: proves the file and the
    #    engine's layout fit together before any game is played.
    $src = Export-Branch $Branch
    Write-Step "Checking the network in the engine (bench)"
    $oldFlags = $env:RUSTFLAGS
    $env:RUSTFLAGS = "-C target-cpu=native"
    $env:EVALFILE = $final
    try {
        Invoke-Native cargo @("build", "--release", "--manifest-path", (Join-Path $src "Cargo.toml"),
            "--target-dir", (Get-OutDir "target-$Tag"))
    } finally {
        $env:RUSTFLAGS = $oldFlags
        Remove-Item Env:EVALFILE -ErrorAction SilentlyContinue
    }
    $engine = Join-Path (Join-Path (Get-OutDir "target-$Tag") "release") "trinity$($script:ExeSuffix)"
    $bench = (Get-NativeOutput $engine @("bench")) | Select-Object -Last 1
    Write-Host "Bench with the new network: $bench"
    if ($bench -notmatch "nodes") { Stop-WithMessage "The engine did not run its bench with the new network." }
    if ($DryRun) {
        Write-Host ""
        Write-Good "DRY RUN OK: training with $KingBuckets king buckets, saving ($size bytes, the right size) and loading in the engine all worked. Now run the real run (double-click its .bat)."
        return
    }

    # 4. Publish on a new branch on top of net-kb16 (whose engine loads both layouts).
    $netBranch = "net-$Tag-$stamp"
    $seedNote = if ($Seed) { ", weight seed $Seed" } else { "" }
    $commit = Publish-NetBranch $Branch $netBranch $final `
        "Network $netId ($Hidden neurons, $KingBuckets king buckets; from scratch, $Superbatches superbatches on $positions billion Leela positions, LR $LearningRate cosine, WDL $Wdl$seedNote)"

    $fmt = { param($t) $t.ToString("yyyy-MM-dd HH:mm") }
    $summary = @(
        "$($Tag.ToUpper()) NETWORK DONE: branch $netBranch (commit $($commit.Substring(0, 10)); $Hidden neurons, $KingBuckets king buckets, $positions billion positions, $Superbatches superbatches$seedNote)",
        "Started $(& $fmt $started); training finished $(& $fmt $trained) ($([Math]::Round(($trained - $started).TotalHours, 1)) hours)",
        "Bench with the new network: $bench",
        "Last lines of the trainer's log ($log):"
    ) + @(Get-Content -LiteralPath $log -Tail 6)
    Set-Content -Path (Join-Path $resultsDir "$Tag-$stamp.txt") -Value ($summary -join "`n")
    # Under the runner the network itself went to trinity-results
    # (Publish-NetBranch); its summary goes next to it.
    if ($env:TRINITY_RESULTS_DIR) {
        Publish-ToResultsRepo "Network ${netBranch}: summary" @{ "nets/$netBranch.txt" = ($summary -join "`n") }
    }
    Write-Host ""
    Write-Host "---------------- copy from here ----------------" -ForegroundColor Cyan
    $summary | ForEach-Object { Write-Host $_ }
    Write-Host "---------------- to here ----------------" -ForegroundColor Cyan
    Write-Good "Done. Paste the text between the lines to Claude (it is also saved in sprt\coder2\results\$Tag-$stamp.txt)."
} catch {
    Write-UnexpectedError $_
    exit 1
} finally {
    Stop-KeepAwake $keepAwake
}
