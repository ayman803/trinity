# More data (Coder 2's second training run; owner's decision of 16
# October): main's 1024 recipe EXACTLY as it was (big.ps1: from scratch,
# 600 superbatches, LR 0.001 cosine, WDL 0.5, batch 16384, 8 king buckets,
# 8 output buckets, 1024 neurons) with ONE change: a second Leela file
# added to the data. The first file's four converted parts in
# sprt\coder2\data (the same positions the 1024 run used) are reused; the
# second file (put it in C:\Trinity\data\leela2) is converted into
# -SecondParts parts of a billion positions (default 2: disk budget).
# Publishes a net-data1024-<date> branch for an SPRT test (the engine is
# main's; no engine change). Double-click tools\net\New-Data-Network.bat.
#
# FOR THE SESSION'S REVIEW (owner's rule of 15 October): writes only inside
# sprt\coder2: data\second-<n>.raw and .data, checkpoints\<name>-<n>,
# results\train-<name>.log and results\newdata-<date>.txt, the trainer's
# source and build. DELETES one kind of file: its own data\second-<n>.raw
# after that part has been shuffled into second-<n>.data. Never touches
# C:\Trinity\data, C:\Trinity\checkpoints or anything else. Publishing
# writes git objects into C:\Trinity\.git and pushes a new branch.

param(
    [int]$Hidden = 1024,
    [string]$Branch = "main",         # trainer source and the base of the published branch (engine unchanged)
    [int]$SecondParts = 2,            # parts of a billion positions to convert from the second Leela file
    [long]$PerChunk = 1000000000,
    [int]$Superbatches = 600,
    [string]$LearningRate = "0.001",
    [string]$Wdl = "0.5",
    [int]$MemoryMB = 8000,
    [switch]$DryRun                   # every stage on a few million positions, about 2 minutes; publishes nothing
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
        # Same stages on a tiny sample, in a folder of its own (made fresh
        # each time; only this script's own dry-run files are removed).
        $workDir = Join-Path $script:Out "dry"
        if (Test-Path -LiteralPath $workDir) { Remove-Item -LiteralPath $workDir -Recurse -Force }
        New-Item -ItemType Directory -Force -Path $workDir | Out-Null
        $SecondParts = 1
        $PerChunk = 3000000
        $Superbatches = 1
        Write-Step "DRY RUN: convert 3 million positions, shuffle, train 1 superbatch, save; nothing is published"
    }
    # The first file's parts, made by Wide-Network.bat (same positions as the 1024 run).
    $firstParts = @(1..4 | ForEach-Object { Join-Path $dataDir "big-$_.data" } | Where-Object { Test-Path -LiteralPath $_ })
    if ($firstParts.Count -lt 4) { Stop-WithMessage "Expected the 4 converted parts big-1.data .. big-4.data in sprt\coder2\data (found $($firstParts.Count)). Wide-Network.bat makes them." }
    $binpacks = @(Find-Files (Join-Path (Join-Path $script:Root "data") "leela2") "*.binpack")
    if ($binpacks.Count -ne 1) { Stop-WithMessage "Expected exactly one .binpack file in data\leela2 (the second Leela file; found $($binpacks.Count))." }
    $utils = Join-Path (Join-Path (Join-Path (Join-Path (Join-Path $script:Root "sprt") "tools") "bullet") "bin") "bullet-utils$($script:ExeSuffix)"
    if (-not (Test-Path -LiteralPath $utils)) { Stop-WithMessage "$utils is missing (4-Train-Network.bat installs it)." }

    $Chunks = $SecondParts
    $secondDir = if ($DryRun) { $workDir } else { $dataDir }
    $parts = @(1..$Chunks | ForEach-Object { Join-Path $secondDir "second-$_.data" })
    $missing = @($parts | Where-Object { -not (Test-Path -LiteralPath $_) })
    $drive = [IO.DriveInfo]::new([IO.Path]::GetPathRoot($script:Root))
    $freeGB = [Math]::Floor($drive.AvailableFreeSpace / 1GB)
    if ($missing.Count -gt 0 -and -not $DryRun) {
        # 32 GB per part, 32 GB more while a part is shuffled, 20 GB spare.
        $needGB = 32 * $missing.Count + 52
        if ($freeGB -lt $needGB) {
            Stop-WithMessage "Only $freeGB GB free; about $needGB GB are needed for $($missing.Count) billion positions. Free some space (or ask Claude to use fewer parts)."
        }
    }
    $keepAwake = Start-KeepAwake
    $started = Get-Date

    # 1. Build the trainer from the branch.
    $trainer = Build-NetTrainer $Branch

    # 2. Convert (one pass over the Leela file), then shuffle each part.
    #    Parts already made by an earlier run are kept and reused.
    if ($missing.Count -gt 0) {
        $prefix = Join-Path $secondDir "second"
        Write-Step "Converting up to $Chunks x $PerChunk positions of the second Leela file (about 20 minutes per billion)"
        Invoke-Native $trainer @("convert-chunks", $binpacks[0], $prefix, "$Chunks", "$PerChunk")
        foreach ($i in 1..$Chunks) {
            $raw = "$prefix-$i.raw"
            $shuffledPart = $parts[$i - 1]
            if (-not (Test-Path -LiteralPath $raw)) { continue }
            if ((Get-Item -LiteralPath $raw).Length -lt 32MB) {
                # An empty tail part (the file ran out): the one deletion, of this script's own file.
                Remove-Item -LiteralPath $raw
                continue
            }
            Write-Step "Shuffling part $i of $Chunks"
            Invoke-Native $utils @("validate", "--input", $raw)
            Invoke-Native $utils @("shuffle", "--input", $raw, "--output", $shuffledPart, "--mem-used-mb", "$MemoryMB")
            # The shuffled copy replaces the raw one (this script's own file).
            Remove-Item -LiteralPath $raw
            Show-DiskSpace
        }
    }
    $secondFiles = @($parts | Where-Object { Test-Path -LiteralPath $_ })
    if ($secondFiles.Count -eq 0) { Stop-WithMessage "No converted data was produced from the second file." }
    # Interleaved (Session's review, 16 October): the trainer reads the files
    # in order, so alternating the months keeps the final low-learning-rate
    # stretch from being mostly one month: big-1, second-1, big-2, second-2, big-3, big-4.
    $files = @()
    if ($DryRun) {
        # The tiny new part first, then a real part, so one superbatch reads both.
        $files = @($secondFiles[0], $firstParts[0])
    } else {
    for ($k = 0; $k -lt [Math]::Max($firstParts.Count, $secondFiles.Count); $k++) {
        if ($k -lt $firstParts.Count) { $files += $firstParts[$k] }
        if ($k -lt $secondFiles.Count) { $files += $secondFiles[$k] }
    }
    }
    Write-Host "Data files in training order: $(($files | ForEach-Object { Split-Path $_ -Leaf }) -join ', ')"
    $positions = [Math]::Round((($files | ForEach-Object { (Get-Item -LiteralPath $_).Length } | Measure-Object -Sum).Sum) / 32 / 1e9, 2)
    $dataReady = Get-Date

    # 3. Train from scratch, keeping the trainer's output (loss per superbatch).
    #    The trainer writes checkpoints\<name>-<n> under the current folder.
    $stamp = Get-Date -Format "yyyyMMdd-HHmm"
    $netId = "trinity-$stamp-data$Hidden"
    $resultsDir = Get-OutDir "results"
    $log = Join-Path $resultsDir "train-$netId.log"
    Write-Step "Training $netId from scratch: $Hidden neurons, $Superbatches superbatches on $positions billion positions (the 1024 run took 5 hours)"
    Write-Host "Trainer output is also saved in $log"
    $env:TRINITY_HIDDEN = "$Hidden"
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
    }
    $final = Join-Path (Join-Path (Join-Path $workDir "checkpoints") "$netId-$Superbatches") "quantised.bin"
    if (-not (Test-Path $final)) { Stop-WithMessage "Training finished but $final was not found." }
    $trained = Get-Date
    $expected = 2 * (8 * 768 * $Hidden + $Hidden + 8 * (2 * $Hidden + 1))
    $size = (Get-Item -LiteralPath $final).Length
    if ($size -lt $expected -or $size -ge $expected + 64) { Stop-WithMessage "The saved network has $size bytes; a $Hidden-neuron network has $expected (plus up to 63 bytes of padding)." }
    if ($DryRun) {
        Write-Host ""
        Write-Good "DRY RUN OK: convert, shuffle, train and save all worked (network file $size bytes, the right size). Now run New-Data-Network.bat for real (double-click it again)."
        return
    }

    # 4. Publish on a new branch on top of the engine branch (which can load
    #    the wider network) so it can be SPRT-tested against main.
    $netBranch = "net-data$Hidden-$stamp"
    $commit = Publish-NetBranch $Branch $netBranch $final `
        "Network $netId ($Hidden neurons, king buckets; from scratch, $Superbatches superbatches on $positions billion Leela positions from two files, LR $LearningRate cosine, WDL $Wdl)"

    $fmt = { param($t) $t.ToString("yyyy-MM-dd HH:mm") }
    $summary = @(
        "NEW-DATA NETWORK DONE: branch $netBranch (commit $($commit.Substring(0, 10)); $Hidden neurons, $positions billion positions, $Superbatches superbatches)",
        "Started $(& $fmt $started); data ready $(& $fmt $dataReady); training finished $(& $fmt $trained) ($([Math]::Round(($trained - $dataReady).TotalHours, 1)) hours of training)",
        "Last lines of the trainer's log ($log):"
    ) + @(Get-Content -LiteralPath $log -Tail 6)
    Set-Content -Path (Join-Path $resultsDir "newdata-$stamp.txt") -Value ($summary -join "`n")
    Write-Host ""
    Write-Host "---------------- copy from here ----------------" -ForegroundColor Cyan
    $summary | ForEach-Object { Write-Host $_ }
    Write-Host "---------------- to here ----------------" -ForegroundColor Cyan
    Write-Good "Done. Paste the text between the lines to Claude (it is also saved in sprt\coder2\results\newdata-$stamp.txt)."
} catch {
    Write-UnexpectedError $_
    exit 1
} finally {
    Stop-KeepAwake $keepAwake
}
