# Train a new NNUE network on the GPU with bullet, then publish it on a new
# branch so it can be SPRT-tested against the current version.
# Double-click 4-Train-Network.bat.
#
# Needs: an NVIDIA GPU and the CUDA Toolkit (see GUIDE.md).

param(
    [string]$Branch = "",        # engine code the network is for (asked if empty)
    [int]$Superbatches = 40,     # length of training (1 superbatch = ~100M positions)
    [int]$MemoryMB = 8000,       # RAM used for shuffling
    [switch]$SkipShuffle,        # reuse data\prepared\shuffled.data from last time
    [switch]$NoPublish           # don't create a GitHub branch at the end
)

. (Join-Path $PSScriptRoot "common.ps1")

# A network must match the engine code that reads it. Normally that is
# 'main'; a branch named arch-... holds a new network design.
function Select-CodeBranch {
    $refs = Get-NativeOutput git @("-C", $script:Root, "for-each-ref", "--sort=-committerdate",
        "--format=%(refname:short)", "refs/remotes/origin/arch-*")
    $choices = @()
    foreach ($r in ($refs -split "`n")) {
        if (-not $r) { continue }
        $old = $ErrorActionPreference; $ErrorActionPreference = "Continue"
        & git -C $script:Root merge-base --is-ancestor $r "origin/main" 2>$null
        $merged = ($LASTEXITCODE -eq 0)
        $ErrorActionPreference = $old
        if (-not $merged) { $choices += ($r -replace "^origin/", "") }
    }
    if ($choices.Count -eq 0) { return "main" }
    $choices += "main"
    Write-Host ""
    Write-Host "Which engine version is this network for?"
    for ($i = 0; $i -lt $choices.Count; $i++) { Write-Host ("  {0}) {1}" -f ($i + 1), $choices[$i]) }
    $answer = Read-Host "Type a number and press Enter (just Enter = 1)"
    if (-not $answer) { $answer = "1" }
    $n = 0
    if (-not [int]::TryParse($answer, [ref]$n) -or $n -lt 1 -or $n -gt $choices.Count) {
        Stop-WithMessage "'$answer' is not one of the numbers shown."
    }
    return $choices[$n - 1]
}

$keepAwake = $null
try {
    Assert-Tools
    if ($script:OnWindows -and -not $env:CUDA_PATH) {
        Stop-WithMessage "The NVIDIA CUDA Toolkit is not installed (or this window was opened before installing it). See GUIDE.md, 'Training a network'."
    }
    Invoke-Native git @("-C", $script:Root, "fetch", "--prune", "origin")
    if (-not $Branch) { $Branch = Select-CodeBranch }
    Write-Host "Training a network for the engine version '$Branch'."
    $dataDir = Join-Path $script:Root "data"
    $prepared = Join-Path $dataDir "prepared"
    $shuffled = Join-Path $prepared "shuffled.data"
    New-Item -ItemType Directory -Force -Path $prepared | Out-Null
    $keepAwake = Start-KeepAwake

    # bullet-utils: bullet's data tools (shuffle, interleave, validate).
    $toolsRoot = Join-Path (Join-Path $script:Work "tools") "bullet"
    $utils = Join-Path (Join-Path $toolsRoot "bin") "bullet-utils$($script:ExeSuffix)"
    if (-not (Test-Path $utils)) {
        Write-Step "Building bullet's data tools (one time)"
        Invoke-Native cargo @("install", "--locked", "--git", "https://github.com/jw1912/bullet", "--rev", $script:BulletRev,
            "bullet-utils", "--root", $toolsRoot)
    }

    $files = @(Find-Files $dataDir "*.data" |
        Where-Object { $_ -notlike "$prepared*" -and (Get-Item -LiteralPath $_).Length -gt 0 })
    # Reuse the shuffled file from a previous attempt if no data changed since.
    $upToDate = $false
    if (Test-Path -LiteralPath $shuffled) {
        $shuffledTime = (Get-Item -LiteralPath $shuffled).LastWriteTime
        $newest = $files | ForEach-Object { (Get-Item -LiteralPath $_).LastWriteTime } | Sort-Object | Select-Object -Last 1
        $upToDate = ($null -eq $newest) -or ($newest -lt $shuffledTime)
    }
    if ($upToDate -or ($SkipShuffle -and (Test-Path -LiteralPath $shuffled))) {
        Write-Host "Reusing the already shuffled data (no new data since last time)."
    } else {
        if (-not $files) { Stop-WithMessage "No training data found in data\. Run 3-Generate-Training-Data.bat first." }
        $total = ($files | ForEach-Object { (Get-Item -LiteralPath $_).Length } | Measure-Object -Sum).Sum
        Write-Step ("Preparing {0:N0} positions from {1} files" -f ($total / 32), @($files).Count)
        $combined = Join-Path $prepared "combined.data"
        Invoke-Native $utils (@("interleave") + $files + @("--output", $combined))
        Invoke-Native $utils @("validate", "--input", $combined)
        if (Test-Path $shuffled) { Remove-Item $shuffled }
        Invoke-Native $utils @("shuffle", "--input", $combined, "--output", $shuffled, "--mem-used-mb", "$MemoryMB")
        Remove-Item $combined
    }

    Write-Step "Building the trainer for '$Branch' (first time takes a few minutes)"
    $features = if ($script:OnWindows) { @("--features", "cuda") } else { @() }
    $codeTree = Join-Path $script:Work "trainer-src"
    if (Test-Path $codeTree) { Invoke-Native git @("-C", $script:Root, "worktree", "remove", "--force", $codeTree) }
    Invoke-Native git @("-C", $script:Root, "worktree", "prune")
    Invoke-Native git @("-C", $script:Root, "worktree", "add", "--detach", $codeTree, "origin/$Branch")
    try {
        Invoke-Native cargo (@("build", "--release", "--manifest-path", (Join-Path (Join-Path $codeTree "trainer") "Cargo.toml"),
                "--target-dir", (Join-Path (Join-Path $script:Root "trainer") "target")) + $features)
    } finally {
        Invoke-Native git @("-C", $script:Root, "worktree", "remove", "--force", $codeTree)
    }
    $trainer = Join-Path (Join-Path (Join-Path (Join-Path $script:Root "trainer") "target") "release") "trinity-trainer$($script:ExeSuffix)"

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
    $netBranch = "net-$stamp"
    Write-Step "Publishing the network on GitHub as branch '$netBranch'"
    $tree = Join-Path $script:Work "publish-$stamp"
    Invoke-Native git @("-C", $script:Root, "fetch", "origin")
    # -B: reuse the branch name if an earlier attempt left it behind locally.
    Invoke-Native git @("-C", $script:Root, "worktree", "add", "-B", $netBranch, $tree, "origin/$Branch")
    try {
        $nets = Join-Path $tree "nets"
        New-Item -ItemType Directory -Force -Path $nets | Out-Null
        Copy-Item $final (Join-Path $nets "default.nnue") -Force
        $identity = @()
        # Is a git identity configured? (`git config` exits with 1 when not.)
        # Note: Windows PowerShell 5.1 drops empty-string arguments to programs.
        $old = $ErrorActionPreference; $ErrorActionPreference = "Continue"
        $email = & git -C $tree config user.email 2>$null
        $ErrorActionPreference = $old
        if (-not $email) {
            $identity = @("-c", "user.name=Trinity trainer", "-c", "user.email=trainer@trinity.invalid")
        }
        Invoke-Native git @("-C", $tree, "add", "nets/default.nnue")
        Invoke-Native git ($identity + @("-C", $tree, "commit", "-m", "New network $netId ($Superbatches superbatches, for $Branch)"))
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
