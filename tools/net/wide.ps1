# Wider network (Coder 2's first training run): the same recipe that made
# main's 1024-neuron network (scripts\big.ps1: convert the whole Leela file,
# shuffle, train from scratch, 600 superbatches, LR 0.001 cosine, WDL 0.5),
# with ONE change: a wider hidden layer (2048 neurons by default; 1536 with
# -Hidden 1536). Publishes a net-wide<N>-<date> branch for an SPRT test.
# Double-click tools\net\Wide-Network.bat (runs about a day; see NETWORK.md).
#
# FOR THE SESSION'S REVIEW (owner's rule of 15 October): this script
# writes only inside sprt\coder2: data\big-<n>.raw and .data (32 GB per
# billion positions), checkpoints\<name>-<n>, results\train-<name>.log and
# results\wide-<date>.txt, the trainer's source and build. It DELETES one
# kind of file: its own data\big-<n>.raw after that part has been shuffled
# into big-<n>.data (otherwise the data takes twice the space). It never
# touches C:\Trinity\data, C:\Trinity\checkpoints or anything else.
# Publishing writes git objects into C:\Trinity\.git and pushes a new
# branch (no worktree, nothing checked out).
#
# Differences from scripts\big.ps1: the trainer and the published branch
# come from the branch net-wide (engine support for the wider network); the
# trainer's output (the loss per superbatch) is saved; start and end times
# are printed; no housekeeping (nothing of the Session's is deleted).

param(
    [int]$Hidden = 2048,
    [string]$Branch = "net-wide",     # engine support for the wider network; the trained net is published on top of it
    [int]$Chunks = 4,
    [long]$PerChunk = 1000000000,
    [int]$Superbatches = 600,
    [string]$LearningRate = "0.001",
    [string]$Wdl = "0.5",
    [int]$MemoryMB = 8000
)

. (Join-Path $PSScriptRoot "net-common.ps1")

$keepAwake = $null
try {
    Assert-Tools
    Show-DiskSpace
    if ($script:OnWindows -and -not $env:CUDA_PATH) {
        Stop-WithMessage "The NVIDIA CUDA Toolkit is not installed (or this window was opened before installing it)."
    }
    if ($Hidden -notin @(1024, 1536, 2048)) { Stop-WithMessage "Hidden must be 1024, 1536 or 2048 (the sizes the engine on branch $Branch supports)." }
    Invoke-Native git @("-C", $script:Root, "fetch", "--prune", "origin")
    $binpacks = @(Find-Files (Join-Path (Join-Path $script:Root "data") "leela") "*.binpack")
    if ($binpacks.Count -ne 1) { Stop-WithMessage "Expected exactly one .binpack file in data\leela (found $($binpacks.Count))." }
    $utils = Join-Path (Join-Path (Join-Path (Join-Path (Join-Path $script:Root "sprt") "tools") "bullet") "bin") "bullet-utils$($script:ExeSuffix)"
    if (-not (Test-Path -LiteralPath $utils)) { Stop-WithMessage "$utils is missing (4-Train-Network.bat installs it)." }

    $dataDir = Get-OutDir "data"
    $parts = @(1..$Chunks | ForEach-Object { Join-Path $dataDir "big-$_.data" })
    $missing = @($parts | Where-Object { -not (Test-Path -LiteralPath $_) })
    $drive = [IO.DriveInfo]::new([IO.Path]::GetPathRoot($script:Root))
    $freeGB = [Math]::Floor($drive.AvailableFreeSpace / 1GB)
    if ($missing.Count -gt 0) {
        # 32 GB per part, 32 GB more while a part is shuffled, 20 GB spare.
        $needGB = 32 * $missing.Count + 52
        if ($freeGB -lt $needGB) {
            Stop-WithMessage "Only $freeGB GB free; about $needGB GB are needed for $($missing.Count) billion positions. Free some space (or ask Coder 2 to use fewer parts)."
        }
    }
    $keepAwake = Start-KeepAwake
    $started = Get-Date

    # 1. Build the trainer from the branch.
    $trainer = Build-NetTrainer $Branch

    # 2. Convert (one pass over the Leela file), then shuffle each part.
    #    Parts already made by an earlier run are kept and reused.
    if ($missing.Count -gt 0) {
        $prefix = Join-Path $dataDir "big"
        Write-Step "Converting up to $Chunks x $PerChunk Leela positions (about 20 minutes per billion)"
        Invoke-Native $trainer @("convert-chunks", $binpacks[0], $prefix, "$Chunks", "$PerChunk")
        foreach ($i in 1..$Chunks) {
            $raw = "$prefix-$i.raw"
            $out = $parts[$i - 1]
            if (-not (Test-Path -LiteralPath $raw)) { continue }
            if ((Get-Item -LiteralPath $raw).Length -lt 32MB) {
                # An empty tail part (the file ran out): the one deletion, of this script's own file.
                Remove-Item -LiteralPath $raw
                continue
            }
            Write-Step "Shuffling part $i of $Chunks"
            Invoke-Native $utils @("validate", "--input", $raw)
            Invoke-Native $utils @("shuffle", "--input", $raw, "--output", $out, "--mem-used-mb", "$MemoryMB")
            # The shuffled copy replaces the raw one (this script's own file).
            Remove-Item -LiteralPath $raw
            Show-DiskSpace
        }
    }
    $files = @($parts | Where-Object { Test-Path -LiteralPath $_ })
    if ($files.Count -eq 0) { Stop-WithMessage "No converted data was produced." }
    $positions = [Math]::Round((($files | ForEach-Object { (Get-Item -LiteralPath $_).Length } | Measure-Object -Sum).Sum) / 32 / 1e9, 2)
    $dataReady = Get-Date

    # 3. Train from scratch, keeping the trainer's output (loss per superbatch).
    #    The trainer writes checkpoints\<name>-<n> under the current folder.
    $stamp = Get-Date -Format "yyyyMMdd-HHmm"
    $netId = "trinity-$stamp-wide$Hidden"
    $resultsDir = Get-OutDir "results"
    $log = Join-Path $resultsDir "train-$netId.log"
    Write-Step "Training $netId from scratch: $Hidden neurons, $Superbatches superbatches on $positions billion positions (the 1024 run took under 10 hours; expect about twice that)"
    Write-Host "Trainer output is also saved in $log"
    $env:TRINITY_HIDDEN = "$Hidden"
    Push-Location $script:Out
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
    }
    $final = Join-Path (Join-Path (Join-Path $script:Out "checkpoints") "$netId-$Superbatches") "quantised.bin"
    if (-not (Test-Path $final)) { Stop-WithMessage "Training finished but $final was not found." }
    $trained = Get-Date

    # 4. Publish on a new branch on top of the engine branch (which can load
    #    the wider network) so it can be SPRT-tested against main.
    $netBranch = "net-wide$Hidden-$stamp"
    $commit = Publish-NetBranch $Branch $netBranch $final `
        "Network $netId ($Hidden neurons, king buckets; from scratch, $Superbatches superbatches on $positions billion Leela positions, LR $LearningRate cosine, WDL $Wdl)"

    $fmt = { param($t) $t.ToString("yyyy-MM-dd HH:mm") }
    $summary = @(
        "WIDE NETWORK DONE: branch $netBranch (commit $($commit.Substring(0, 10)); $Hidden neurons, $positions billion positions, $Superbatches superbatches)",
        "Started $(& $fmt $started); data ready $(& $fmt $dataReady); training finished $(& $fmt $trained) ($([Math]::Round(($trained - $dataReady).TotalHours, 1)) hours of training)",
        "Last lines of the trainer's log ($log):"
    ) + @(Get-Content -LiteralPath $log -Tail 6)
    Set-Content -Path (Join-Path $resultsDir "wide-$stamp.txt") -Value ($summary -join "`n")
    Write-Host ""
    Write-Host "---------------- copy from here ----------------" -ForegroundColor Cyan
    $summary | ForEach-Object { Write-Host $_ }
    Write-Host "---------------- to here ----------------" -ForegroundColor Cyan
    Write-Good "Done. Paste the text between the lines to Coder 2 (it is also saved in sprt\coder2\results\wide-$stamp.txt)."
} catch {
    Write-UnexpectedError $_
    exit 1
} finally {
    Stop-KeepAwake $keepAwake
}
