# Setup-drift check (advisor, 19 October 2026; NETWORK.md section 6): does
# today's trainer still train exactly like the one that made main's network
# (commit 8b4d54f, 9 October)? Trains ONE superbatch from scratch with each
# (1024 neurons, 8 king buckets, bullet's default seed, LR 0.001, WDL 0.5,
# the same data file big-1.data) and compares the loss and the saved files.
# Equal: the control's -10 is not a setup change. Different: it may be.
# About 10 minutes. Double-click tools\net\Drift-Check.bat.
#
# Writes only inside sprt\coder2 (drift\, the two trainers' sources and
# builds). Deletes one thing: its own drift\ folder from an earlier run.

param(
    [string]$OldCommit = "8b4d54f",       # the trainer that made main's network (11-Big-Network.bat, 9 October)
    [string]$NewBranch = "net-kb16"       # today's trainer (the control and 16-bucket runs)
)

. (Join-Path $PSScriptRoot "net-common.ps1")

# Build the trainer of any commit (with CUDA on Windows) in its own target folder.
function Build-TrainerAt([string]$Ref, [string]$Label) {
    $sha = Get-NativeOutput git @("-C", $script:Root, "rev-parse", "--short=10", $Ref)
    $dir = Join-Path (Get-OutDir "src") "drift-$Label-$sha"
    if (-not (Test-Path -LiteralPath (Join-Path $dir "trainer\Cargo.toml"))) {
        Write-Step "Getting the trainer source of $Label ($sha)"
        $zip = "$dir.zip"
        Invoke-Native git @("-C", $script:Root, "archive", "--format=zip", "--output=$zip", $sha)
        Expand-Archive -Path $zip -DestinationPath $dir -Force
    }
    Write-Step "Building the $Label trainer ($sha)"
    $features = if ($script:OnWindows) { @("--features", "cuda") } else { @() }
    # A target folder per trainer, so cargo cannot mistake one for the other.
    $target = Get-OutDir "target-drift-$Label"
    Invoke-Native cargo (@("build", "--release", "--manifest-path", (Join-Path (Join-Path $dir "trainer") "Cargo.toml"),
            "--target-dir", $target) + $features)
    $built = Join-Path (Join-Path $target "release") "trinity-trainer$($script:ExeSuffix)"
    $copy = Join-Path (Get-OutDir "drift-bin") "trainer-$Label$($script:ExeSuffix)"
    Copy-Item -LiteralPath $built -Destination $copy -Force
    return $copy
}

try {
    Assert-Tools
    if ($script:OnWindows -and -not $env:CUDA_PATH) { Stop-WithMessage "The NVIDIA CUDA Toolkit is not installed." }
    Invoke-Native git @("-C", $script:Root, "fetch", "--prune", "origin")
    $data = Join-Path (Get-OutDir "data") "big-1.data"
    if (-not (Test-Path -LiteralPath $data)) { Stop-WithMessage "sprt\coder2\data\big-1.data is missing." }
    $workDir = Join-Path $script:Out "drift"
    if (Test-Path -LiteralPath $workDir) { Remove-Item -LiteralPath $workDir -Recurse -Force }
    New-Item -ItemType Directory -Force -Path $workDir | Out-Null

    $trainers = [ordered]@{ old = (Build-TrainerAt $OldCommit "old"); new = (Build-TrainerAt "origin/$NewBranch" "new") }
    $losses = @{}
    $hashes = @{}
    $env:TRINITY_HIDDEN = "1024"
    Remove-Item Env:TRINITY_KING_BUCKETS -ErrorAction SilentlyContinue
    Remove-Item Env:TRINITY_SEED -ErrorAction SilentlyContinue
    try {
        foreach ($k in $trainers.Keys) {
            $dir = Join-Path $workDir $k
            New-Item -ItemType Directory -Force -Path $dir | Out-Null
            Write-Step "Training 1 superbatch with the $k trainer"
            Push-Location $dir
            try {
                $old = $ErrorActionPreference
                $ErrorActionPreference = "Continue"
                $text = (& $trainers[$k] @("scratch", "1", "drift", "0.001", "0.5", $data) 2>&1 | ForEach-Object { "$_" })
                $ErrorActionPreference = $old
            } finally { Pop-Location }
            # The trainer colours its numbers: strip the colour codes before reading them.
            $plain = (@($text) -join "`n") -replace "\x1b\[[0-9;]*m", ""
            $plain | Set-Content -LiteralPath (Join-Path $workDir "$k.log")
            $m = [regex]::Matches($plain, 'running loss ([0-9.]+)')
            if ($m.Count -eq 0) {
                Write-Host ($plain -split "`n" | Select-Object -Last 15 | Out-String)
                Stop-WithMessage "The $k trainer printed no loss (its output is above and in sprt\coder2\drift\$k.log)."
            }
            $losses[$k] = [double]::Parse($m[$m.Count - 1].Groups[1].Value, [Globalization.CultureInfo]::InvariantCulture)
            $q = Join-Path (Join-Path (Join-Path $dir "checkpoints") "drift-1") "quantised.bin"
            $hashes[$k] = if (Test-Path -LiteralPath $q) { (Get-FileHash -LiteralPath $q -Algorithm SHA256).Hash } else { "missing" }
        }
    } finally {
        Remove-Item Env:TRINITY_HIDDEN -ErrorAction SilentlyContinue
    }
    $rel = [Math]::Abs($losses["old"] - $losses["new"]) / $losses["old"]
    $verdict = if ($hashes["old"] -eq $hashes["new"]) { "IDENTICAL networks: no setup drift" }
        elseif ($rel -lt 0.0005) { "SAME loss within 0.05% (files differ only by GPU rounding): no setup drift" }
        else { "DIFFERENT: setup drift is possible; tell Claude" }
    $summary = @(
        "DRIFT CHECK: old trainer ($OldCommit) loss $($losses['old']), new trainer ($NewBranch) loss $($losses['new']), relative difference $([Math]::Round(100 * $rel, 4))%",
        "Saved networks: $(if ($hashes['old'] -eq $hashes['new']) { 'identical' } else { 'different' })",
        "Verdict: $verdict"
    )
    Set-Content -LiteralPath (Join-Path (Get-OutDir "results") "drift-$(Get-Date -Format 'yyyy-MM-dd_HH-mm').txt") -Value $summary
    Write-Host ""
    Write-Host "---------------- copy from here ----------------" -ForegroundColor Cyan
    $summary | ForEach-Object { Write-Host $_ }
    Write-Host "---------------- to here ----------------" -ForegroundColor Cyan
} catch {
    Write-UnexpectedError $_
    exit 1
}
