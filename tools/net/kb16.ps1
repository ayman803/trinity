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
# Resumable training (advisor and Scout, 12 November): a run longer than a
# night is queued as parts with the same arguments and -Resume, each part
# ending at its -StopAt superbatch; a part that is stopped (DISTURBED) or
# run again continues from the run's newest complete checkpoint (saved
# every 50 superbatches), and the parts follow one uninterrupted run (the
# same data order and learning rate). -From <tag> starts from another
# run's newest checkpoint instead of from scratch: with -Superbatches equal
# to that run's length and -StopAt beyond it, the extra superbatches run at
# the final learning rate (lowlr-tail). Dry run with -Resume checks the
# resume on the GPU (Dry-Run-Resume.bat, about 10 minutes).
#
# Writes only inside sprt\coder2: checkpoints\<name>-<n>, results\
# (train-<name>.log, kb16-<date>.txt), the branch's source and builds.
# Deletes one thing: its own dry-run folder sprt\coder2\dry-kb16 before a
# new dry run. Converts nothing (the data is already there). Publishing
# writes git objects into C:\Trinity\.git and pushes a new branch.

param(
    [int]$Hidden = 1024,
    [string]$Branch = "net-kb16-resume", # engine and trainer (16-bucket layout, resumable training)
    [ValidateSet(8, 16)][int]$KingBuckets = 16,
    [string]$Seed = "",               # weight seed; empty = bullet's default (as main's network)
    [string]$Tag = "kb16",            # name of the run and of the published branch (net-<tag>-<date>)
    [int]$Superbatches = 600,         # the run's planned length (the learning rate reaches its floor there)
    [string]$LearningRate = "0.001",
    [string]$Wdl = "0.5",
    [switch]$Resume,                  # continue this run (same -Tag) from its newest complete checkpoint, if there is one
    [int]$StopAt = 0,                 # end this job at this superbatch (one part of a longer run); 0 = -Superbatches
    [string]$From = "",               # start from the newest checkpoint of another run (its -Tag), e.g. control2
    [switch]$DryRun                   # 1 superbatch, save, check the file loads in the engine; publishes nothing
)

. (Join-Path $PSScriptRoot "net-common.ps1")

# The approval covers this script, not the branch it builds, so the branch
# is pinned here: a newer commit on it needs a new version of this script
# (and Ayman's approval of it).
$PinnedBranches = @{ "net-kb16-resume" = "6c4e8af396" }

# Complete checkpoints of a run (its tag), oldest first: folders
# trinity-<yyyyMMdd>-<HHmm>-<tag>-<N> whose optimiser state and network were
# both written (quantised.bin is the last file bullet saves).
function Find-RunCheckpoints([string[]]$Dirs, [string]$RunTag) {
    $pattern = '^(trinity-\d{8}-\d{4}-' + [regex]::Escape($RunTag) + ')-(\d+)$'
    $found = @()
    foreach ($d in $Dirs) {
        if (-not (Test-Path -LiteralPath $d)) { continue }
        foreach ($c in Get-ChildItem -LiteralPath $d -Directory) {
            if ($c.Name -notmatch $pattern) { continue }
            $id = $Matches[1]
            $n = [int]$Matches[2]
            if (-not (Test-Path -LiteralPath (Join-Path $c.FullName "quantised.bin"))) { continue }
            $state = Join-Path $c.FullName "optimiser_state"
            $missing = @("weights.bin", "momentum.bin", "velocity.bin" | Where-Object { -not (Test-Path -LiteralPath (Join-Path $state $_)) })
            if ($missing.Count) { continue }
            $found += [pscustomobject]@{ Path = $c.FullName; NetId = $id; Superbatch = $n }
        }
    }
    return @($found | Sort-Object NetId, Superbatch)
}

# Run the trainer with the run's settings, keeping its output in a log.
function Invoke-Trainer([string]$Exe, [string[]]$TrainArgs, [string]$LogFile, [string]$InDir) {
    $env:TRINITY_HIDDEN = "$Hidden"
    $env:TRINITY_KING_BUCKETS = "$KingBuckets"
    if ($Seed) { $env:TRINITY_SEED = $Seed }
    Push-Location $InDir
    try {
        $old = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        & $Exe $TrainArgs 2>&1 | Tee-Object -FilePath $LogFile | Out-Host
        $code = $LASTEXITCODE
        $ErrorActionPreference = $old
        if ($code -ne 0) { Stop-WithMessage "The trainer failed (exit code $code). See $LogFile" }
    } finally {
        Pop-Location
        Remove-Item Env:TRINITY_HIDDEN -ErrorAction SilentlyContinue
        Remove-Item Env:TRINITY_KING_BUCKETS -ErrorAction SilentlyContinue
        Remove-Item Env:TRINITY_SEED -ErrorAction SilentlyContinue
    }
}

# Mean training loss of one superbatch, from a checkpoint's log.txt
# (lines "superbatch,batch,loss" of the part that saved it).
function Get-SuperbatchLoss([string]$CheckpointDir, [int]$Number) {
    $vals = @(Get-Content -LiteralPath (Join-Path $CheckpointDir "log.txt") | ForEach-Object {
            $f = $_ -split ','
            if ($f.Count -eq 3 -and [int]$f[0] -eq $Number) { [double]::Parse($f[2], [Globalization.CultureInfo]::InvariantCulture) }
        })
    if (-not $vals.Count) { Stop-WithMessage "No losses for superbatch $Number in $CheckpointDir\log.txt." }
    return ($vals | Measure-Object -Average).Average
}

$keepAwake = $null
try {
    Assert-Tools
    Show-DiskSpace
    if ($script:OnWindows -and -not $env:CUDA_PATH) {
        Stop-WithMessage "The NVIDIA CUDA Toolkit is not installed (or this window was opened before installing it)."
    }
    Invoke-Native git @("-C", $script:Root, "fetch", "--prune", "origin")
    if ($PinnedBranches.ContainsKey($Branch)) {
        $head = Get-BranchSha $Branch
        if ($head.Substring(0, 10) -ne $PinnedBranches[$Branch]) {
            Stop-WithMessage "Branch $Branch is at $head, but this script was approved for $($PinnedBranches[$Branch])."
        }
    }
    $target = if ($StopAt -gt 0) { $StopAt } else { $Superbatches }
    if ($target -lt 1 -or $Superbatches -lt 1) { Stop-WithMessage "-Superbatches and -StopAt must be at least 1." }
    $dataDir = Get-OutDir "data"
    $workDir = $script:Out
    if ($DryRun) {
        $workDir = Join-Path $script:Out "dry-$Tag"
        if (Test-Path -LiteralPath $workDir) { Remove-Item -LiteralPath $workDir -Recurse -Force }
        New-Item -ItemType Directory -Force -Path $workDir | Out-Null
        $Superbatches = 1
        $target = 1
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
    $resultsDir = Get-OutDir "results"
    $ckptDir = Join-Path $workDir "checkpoints"
    # Checkpoints another run (-From) may come from: this script's own, and
    # those of runs Ayman started by hand in C:\Trinity (Control2).
    $fromDirs = @((Join-Path $script:Out "checkpoints"))
    $handRuns = "C:\Trinity\sprt\coder2\checkpoints"
    if ($script:OnWindows -and (Test-Path -LiteralPath $handRuns) -and ($fromDirs -notcontains $handRuns)) { $fromDirs += $handRuns }

    # 1. Build the trainer from the branch.
    $trainer = Build-NetTrainer $Branch

    $summaryExtra = @()
    if ($DryRun -and $Resume) {
        # Resume check on the GPU, 3 superbatches planned on one data file:
        # A and A2 uninterrupted (A2 shows the GPU's own run-to-run noise), B
        # stopped after superbatch 1 and resumed to 3, then 1 more superbatch
        # past the plan (the final learning rate, as lowlr-tail uses).
        Write-Step "DRY RUN with -Resume: runs A and A2 (3 superbatches each), B (1, then resumed to 3, then 1 more at the final rate)"
        $common = @($LearningRate, $Wdl)
        Invoke-Trainer $trainer (@("part", "3", "3", "dryA") + $common + @("-") + $files) (Join-Path $workDir "dryA.log") $workDir
        Invoke-Trainer $trainer (@("part", "3", "3", "dryA2") + $common + @("-") + $files) (Join-Path $workDir "dryA2.log") $workDir
        Invoke-Trainer $trainer (@("part", "3", "1", "dryB") + $common + @("-") + $files) (Join-Path $workDir "dryB1.log") $workDir
        Invoke-Trainer $trainer (@("part", "3", "3", "dryB") + $common + @((Join-Path $ckptDir "dryB-1")) + $files) (Join-Path $workDir "dryB2.log") $workDir
        Invoke-Trainer $trainer (@("part", "3", "4", "dryB") + $common + @((Join-Path $ckptDir "dryB-3")) + $files) (Join-Path $workDir "dryB3.log") $workDir
        foreach ($c in @("dryA-3", "dryA2-3", "dryB-3", "dryB-4")) {
            if (-not (Test-Path -LiteralPath (Join-Path (Join-Path $ckptDir $c) "quantised.bin"))) { Stop-WithMessage "Checkpoint $c was not saved." }
        }
        $lines = @()
        $worst = 0.0
        $noise = 0.0
        foreach ($sb in @(2, 3)) {
            $a = Get-SuperbatchLoss (Join-Path $ckptDir "dryA-3") $sb
            $a2 = Get-SuperbatchLoss (Join-Path $ckptDir "dryA2-3") $sb
            $b = Get-SuperbatchLoss (Join-Path $ckptDir "dryB-3") $sb
            $noise = [Math]::Max($noise, [Math]::Abs($a2 - $a) / $a)
            $worst = [Math]::Max($worst, [Math]::Abs($b - $a) / $a)
            $lines += ("superbatch {0}: loss A {1:F7}, A2 {2:F7}, B (resumed) {3:F7}" -f $sb, $a, $a2, $b)
        }
        $wAA2 = @(Get-NativeOutput $trainer @("compare", (Join-Path $ckptDir "dryA-3"), (Join-Path $ckptDir "dryA2-3")) | Where-Object { $_ -like "WEIGHTS:*" })
        $wAB = @(Get-NativeOutput $trainer @("compare", (Join-Path $ckptDir "dryA-3"), (Join-Path $ckptDir "dryB-3")) | Where-Object { $_ -like "WEIGHTS:*" })
        $lines += "A vs A2 $($wAA2 -join ' ')"
        $lines += "A vs B  $($wAB -join ' ')"
        $tail = Get-SuperbatchLoss (Join-Path $ckptDir "dryB-4") 4
        $lines += ("superbatch 4 (past the plan, final rate): loss {0:F7}" -f $tail)
        $limit = [Math]::Max(3 * $noise, 0.002)
        $lossLine = "largest loss difference: resumed {0:P3}, run-to-run {1:P3}; allowed {2:P3}" -f $worst, $noise, $limit
        $lines += $lossLine
        if ($From) {
            $src = @(Find-RunCheckpoints $fromDirs $From)
            if (-not $src.Count) {
                $lines += "-From ${From}: NO complete checkpoint found"
            } else {
                $probe = Join-Path (Join-Path $src[-1].Path "optimiser_state") "weights.bin"
                $readable = $true
                try { $fs = [IO.File]::OpenRead($probe); $fs.Close() } catch { $readable = $false }
                $lines += "-From ${From}: newest checkpoint $($src[-1].NetId) at superbatch $($src[-1].Superbatch), readable: $readable"
            }
        }
        $lines | ForEach-Object { Write-Host $_ }
        $summaryExtra = $lines
        if ($worst -gt $limit) { Stop-WithMessage "The resumed run differs from the uninterrupted one by more than allowed ($lossLine)." }
        $final = Join-Path (Join-Path $ckptDir "dryB-3") "quantised.bin"
    } else {
        # 2. Train, keeping the trainer's output: from scratch, from this run's
        #    newest checkpoint (-Resume), or from another run's (-From).
        $stamp = Get-Date -Format "yyyyMMdd-HHmm"
        $netId = "trinity-$stamp-$Tag"
        $startFrom = "-"
        $done = 0
        $own = @(Find-RunCheckpoints @($ckptDir) $Tag)
        if ($Resume -and $own.Count) {
            $netId = $own[-1].NetId
            $startFrom = $own[-1].Path
            $done = $own[-1].Superbatch
            Write-Step "Resuming ${netId}: superbatch $done of $Superbatches done"
        } elseif ($From) {
            $src = @(Find-RunCheckpoints $fromDirs $From)
            if (-not $src.Count) { Stop-WithMessage "No complete checkpoint of run '$From' found in $($fromDirs -join ', ')." }
            $startFrom = $src[-1].Path
            $done = $src[-1].Superbatch
            Write-Step "Starting from $($src[-1].NetId) at superbatch $done"
        }
        $stamp = ($netId -split '-')[1..2] -join '-'
        $log = Join-Path $resultsDir "train-$netId.log"
        if ($done -gt 0) { $log = Join-Path $resultsDir "train-$netId-from-$done.log" }
        $finalDir = Join-Path $ckptDir "$netId-$target"
        if ($done -ge $target) {
            Write-Host "Nothing to train: superbatch $done is done (this part ends at $target)."
            $finalDir = $startFrom
        } else {
            Write-Step "Training $netId, superbatches $($done + 1) to $target of a $Superbatches-superbatch plan: $Hidden neurons, $KingBuckets king buckets, $positions billion positions (about 30 seconds per superbatch with 8 buckets)"
            Write-Host "Trainer output is also saved in $log"
            if ($startFrom -eq "-" -and $target -eq $Superbatches) {
                $trainArgs = @("scratch", "$Superbatches", $netId, $LearningRate, $Wdl) + $files
            } else {
                $trainArgs = @("part", "$Superbatches", "$target", $netId, $LearningRate, $Wdl, $startFrom) + $files
            }
            Invoke-Trainer $trainer $trainArgs $log $workDir
        }
        $final = Join-Path $finalDir "quantised.bin"
        if (-not (Test-Path $final)) { Stop-WithMessage "Training finished but $final was not found." }
        if ($target -lt $Superbatches -and $done -ge $target) {
            Write-Good "Nothing to do: $netId is already at superbatch $done (this part ends at $target)."
            return
        }
        if ($target -lt $Superbatches) {
            # One part of a longer run: the next part (same arguments, a later
            # -StopAt) continues from this checkpoint. Nothing to publish yet.
            $fmt = { param($t) $t.ToString("yyyy-MM-dd HH:mm") }
            $part = @(
                "$($Tag.ToUpper()) PART DONE: $netId at superbatch $target of $Superbatches (started at $done)",
                "Started $(& $fmt $started); finished $(& $fmt (Get-Date)) ($([Math]::Round(((Get-Date) - $started).TotalHours, 1)) hours)",
                "Last lines of the trainer's log:"
            ) + @(if (Test-Path -LiteralPath $log) { Get-Content -LiteralPath $log -Tail 4 })
            Set-Content -Path (Join-Path $resultsDir "$Tag-$stamp-part-$target.txt") -Value ($part -join "`n")
            if ($env:TRINITY_RESULTS_DIR) {
                Publish-ToResultsRepo "Network $Tag-${stamp}: part to superbatch $target" @{ "nets/net-$Tag-$stamp-part-$target.txt" = ($part -join "`n") }
            }
            $part | ForEach-Object { Write-Host $_ }
            Write-Good "Part done. The next part continues from superbatch $target."
            return
        }
    }
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
        if ($Resume) {
            $report = @("DRY RUN RESUME OK ($(Get-Date -Format 'yyyy-MM-dd HH:mm'), branch $Branch $(Get-BranchSha $Branch))") + $summaryExtra + @("Bench with the resumed network: $bench")
            Set-Content -Path (Join-Path $resultsDir "dry-resume.txt") -Value ($report -join "`n")
            if ($env:TRINITY_RESULTS_DIR) {
                Publish-ToResultsRepo "Dry run: resumable training" @{ "nets/dry-resume-$(Get-Date -Format 'yyyyMMdd-HHmm').txt" = ($report -join "`n") }
            }
        }
        Write-Host ""
        Write-Good "DRY RUN OK: training with $KingBuckets king buckets, saving ($size bytes, the right size) and loading in the engine all worked. Now run the real run (double-click its .bat)."
        return
    }

    # 4. Publish on a new branch on top of net-kb16 (whose engine loads both layouts).
    $netBranch = "net-$Tag-$stamp"
    $seedNote = if ($Seed) { ", weight seed $Seed" } else { "" }
    $howNote = if ($From) { "continued from run $From at superbatch $done to $target (plan $Superbatches, final rate past it)" } else { "from scratch, $Superbatches superbatches" }
    $commit = Publish-NetBranch $Branch $netBranch $final `
        "Network $netId ($Hidden neurons, $KingBuckets king buckets; $howNote on $positions billion Leela positions, LR $LearningRate cosine, WDL $Wdl$seedNote)"

    $fmt = { param($t) $t.ToString("yyyy-MM-dd HH:mm") }
    $summary = @(
        "$($Tag.ToUpper()) NETWORK DONE: branch $netBranch (commit $($commit.Substring(0, 10)); $Hidden neurons, $KingBuckets king buckets, $positions billion positions, $howNote$seedNote)",
        "Started $(& $fmt $started); training finished $(& $fmt $trained) ($([Math]::Round(($trained - $started).TotalHours, 1)) hours)",
        "Bench with the new network: $bench",
        "Last lines of the trainer's log ($log):"
    ) + @(if (Test-Path -LiteralPath $log) { Get-Content -LiteralPath $log -Tail 6 })
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
