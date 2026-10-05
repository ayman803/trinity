# Fresh-data training: continue training main's network on Leela positions
# it has never seen (the next part of the Leela file after the positions
# already converted). Publishes a net-fresh-... branch for an SPRT test.
# Double-click 8-Fresh-Leela.bat.

param(
    [long]$Positions = 1000000000, # fresh positions to convert (32 bytes each on disk)
    [int]$Superbatches = 40,       # length of the refinement (1 superbatch = ~100M positions)
    [string]$LearningRate = "0.0002",
    [int]$MemoryMB = 8000          # RAM used for shuffling
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
    $binpacks = @(Find-Files (Join-Path (Join-Path $script:Root "data") "leela") "*.binpack")
    if ($binpacks.Count -ne 1) { Stop-WithMessage "Expected exactly one .binpack file in data\leela (found $($binpacks.Count))." }
    # How far into the Leela file earlier conversions got (raw entries read).
    # The first conversion (4 October 2026) stopped at entry 1746514449.
    $positionFile = Join-Path $prepared "leela-file-position.txt"
    $skip = if (Test-Path -LiteralPath $positionFile) { [long](Get-Content -LiteralPath $positionFile -Raw).Trim() } else { [long]1746514449 }

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

    # 1. Build the trainer from main (it also converts the data).
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

    # 2. Convert the next unused part of the Leela file, then shuffle it.
    $fresh = Join-Path $prepared "leela-fresh.data"
    $raw = Join-Path $prepared "leela-fresh-raw.data"
    if (Test-Path $fresh) { Remove-Item $fresh }
    Write-Step "Converting $Positions fresh Leela positions (skipping the $skip already used; 20-40 minutes)"
    $old = $ErrorActionPreference; $ErrorActionPreference = "Continue"
    $out = & $trainer convert $binpacks[0] $raw "$Positions" "$skip" 2>&1 | Tee-Object -Variable lines | Out-Host
    $code = $LASTEXITCODE
    $ErrorActionPreference = $old
    if ($code -ne 0) { Stop-WithMessage "Converting the Leela data failed (exit code $code)." }
    $done = ($lines | Where-Object { "$_" -match "stopped at file position (\d+)" } | Select-Object -Last 1)
    if (-not ("$done" -match "stopped at file position (\d+)")) { Stop-WithMessage "Could not read where the conversion stopped." }
    Set-Content -LiteralPath $positionFile -Value $Matches[1]
    Invoke-Native $utils @("validate", "--input", $raw)
    Invoke-Native $utils @("shuffle", "--input", $raw, "--output", $fresh, "--mem-used-mb", "$MemoryMB")
    Remove-Item $raw

    $stamp = Get-Date -Format "yyyyMMdd-HHmm"
    $netId = "trinity-$stamp-fresh"
    Write-Step "Training $netId on the fresh positions ($Superbatches superbatches)"
    Push-Location $script:Root
    try {
        Invoke-Native $trainer @("finetune", $checkpoint, "$Superbatches", $netId, $fresh, $LearningRate)
    } finally {
        Pop-Location
    }
    $final = Join-Path (Join-Path (Join-Path $script:Root "checkpoints") "$netId-$Superbatches") "quantised.bin"
    if (-not (Test-Path $final)) { Stop-WithMessage "Training finished but $final was not found." }

    # 3. Publish on a new branch so it can be SPRT-tested against main.
    $netBranch = "net-fresh-$stamp"
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
                "Fresh-data network $netId (main's network + $Superbatches superbatches on $Positions unseen Leela positions)"))
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
