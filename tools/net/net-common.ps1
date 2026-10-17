# Shared helpers for Coder 2's PC scripts (tools\net\*.ps1).
#
# Owner's rule (15 October 2026): these scripts write only inside their own
# output folder, C:\Trinity\sprt\coder2 (builds, data, checkpoints, results),
# and never delete or move anything. A script that must delete or move a
# file says so at its top and is reviewed by the Session before Ayman runs
# it. The main folder's own helpers (scripts\common.ps1) are used only for
# messages, running programs, keeping the PC awake and finding files; their
# housekeeping, worktree and publishing functions are not called.
#
# Git writes: `git archive` reads the repository; publishing a network
# writes objects into C:\Trinity\.git and pushes a new branch, with a
# private index file in the output folder (no worktree, nothing checked out).

$ErrorActionPreference = "Stop"
# PowerShell ignores case in variable names, and code outside functions
# shares the script scope: never use $out, $root, $work, $exesuffix or
# $onwindows as a variable name in these scripts (it would overwrite
# $script:Out etc.; this broke the first wide-network run on 15 October).
$netToolsRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path   # C:\Trinity
. (Join-Path (Join-Path $netToolsRoot "scripts") "common.ps1")
$script:Root = $netToolsRoot
$script:Out = Join-Path (Join-Path $script:Root "sprt") "coder2"
# common.ps1's keep-awake helper writes its (error-only) log under $script:Work.
$script:Work = $script:Out
New-Item -ItemType Directory -Force -Path $script:Out | Out-Null

function Get-OutDir([string]$Name) {
    $d = Join-Path $script:Out $Name
    New-Item -ItemType Directory -Force -Path $d | Out-Null
    return $d
}

# Short commit id of a branch on GitHub (after `git fetch`).
function Get-BranchSha([string]$Branch) {
    return Get-NativeOutput git @("-C", $script:Root, "rev-parse", "--short=10", "origin/$Branch")
}

# The source of a branch, extracted (not checked out) into
# sprt\coder2\src\<branch>-<sha>. Reused when it already exists.
function Export-Branch([string]$Branch) {
    $sha = Get-BranchSha $Branch
    $dir = Join-Path (Get-OutDir "src") "$Branch-$sha"
    if (Test-Path -LiteralPath (Join-Path $dir "Cargo.toml")) { return $dir }
    Write-Step "Getting the source of branch $Branch ($sha)"
    $zip = "$dir.zip"
    Invoke-Native git @("-C", $script:Root, "archive", "--format=zip", "--output=$zip", "origin/$Branch")
    Expand-Archive -Path $zip -DestinationPath $dir -Force
    if (-not (Test-Path -LiteralPath (Join-Path $dir "Cargo.toml"))) { Stop-WithMessage "Could not extract branch $Branch into $dir." }
    return $dir
}

# Build the engine of a branch (for this PC's processor) into
# sprt\coder2\engines and return the executable's path. Cached by commit.
function Build-NetEngine([string]$Branch) {
    $sha = Get-BranchSha $Branch
    $exe = Join-Path (Get-OutDir "engines") "trinity-$Branch-$sha$($script:ExeSuffix)"
    if (Test-Path -LiteralPath $exe) {
        Write-Host "Using the cached build of $Branch ($sha)."
        return $exe
    }
    $src = Export-Branch $Branch
    Write-Step "Building the engine of branch $Branch ($sha)"
    $oldFlags = $env:RUSTFLAGS
    $env:RUSTFLAGS = "-C target-cpu=native"
    try {
        Invoke-Native cargo @("build", "--release", "--manifest-path", (Join-Path $src "Cargo.toml"),
            "--target-dir", (Get-OutDir "target-engine"))
    } finally {
        $env:RUSTFLAGS = $oldFlags
    }
    Copy-Item (Join-Path (Join-Path (Get-OutDir "target-engine") "release") "trinity$($script:ExeSuffix)") $exe -Force
    return $exe
}

# Build the trainer of a branch (with CUDA on Windows) and return its path.
function Build-NetTrainer([string]$Branch) {
    $src = Export-Branch $Branch
    Write-Step "Building the trainer of branch $Branch (the first build takes a few minutes)"
    $features = if ($script:OnWindows) { @("--features", "cuda") } else { @() }
    $target = Get-OutDir "target-trainer"
    Invoke-Native cargo (@("build", "--release", "--manifest-path", (Join-Path (Join-Path $src "trainer") "Cargo.toml"),
            "--target-dir", $target) + $features)
    return Join-Path (Join-Path $target "release") "trinity-trainer$($script:ExeSuffix)"
}

# The checkpoint folder whose quantised.bin is exactly `NetFile`, searched
# in C:\Trinity\checkpoints and in sprt\coder2\checkpoints; or $null.
function Find-NetCheckpoint([string]$NetFile) {
    if (-not (Test-Path -LiteralPath $NetFile)) { return $null }
    $hash = (Get-FileHash -LiteralPath $NetFile -Algorithm SHA256).Hash
    foreach ($ckptDir in @((Join-Path $script:Root "checkpoints"), (Join-Path $script:Out "checkpoints"))) {
        foreach ($q in (Find-Files $ckptDir "quantised.bin")) {
            if ((Get-FileHash -LiteralPath $q -Algorithm SHA256).Hash -eq $hash) { return (Split-Path $q -Parent) }
        }
    }
    return $null
}

# Publish a network file as nets/default.nnue on a new branch that starts
# from origin/<BaseBranch>, without checking anything out: a private index
# file in the output folder, git's own object store, one commit, one push.
function Publish-NetBranch([string]$BaseBranch, [string]$NewBranch, [string]$NetFile, [string]$Message) {
    if ($env:TRINITY_RESULTS_DIR) {
        # Under the runner: the network file goes to trinity-results
        # (nets/<branch>.nnue); the Session makes the branch in trinity.
        Publish-ToResultsRepo $Message @{} @{ "nets/$NewBranch.nnue" = $NetFile }
        return "trinity-results:nets/$NewBranch.nnue"
    }
    Write-Step "Publishing the network on GitHub as branch '$NewBranch' (from $BaseBranch)"
    Invoke-Native git @("-C", $script:Root, "fetch", "origin", $BaseBranch)
    $index = Join-Path (Get-OutDir "publish") "$NewBranch.index"
    $oldIndex = $env:GIT_INDEX_FILE
    $env:GIT_INDEX_FILE = $index
    try {
        Invoke-Native git @("-C", $script:Root, "read-tree", "origin/$BaseBranch")
        $blob = Get-NativeOutput git @("-C", $script:Root, "hash-object", "-w", $NetFile)
        Invoke-Native git @("-C", $script:Root, "update-index", "--add", "--cacheinfo", "100644,$blob,nets/default.nnue")
        $tree = Get-NativeOutput git @("-C", $script:Root, "write-tree")
        $parent = Get-NativeOutput git @("-C", $script:Root, "rev-parse", "origin/$BaseBranch")
        $commit = Get-NativeOutput git ($script:CommitIdentity + @("-C", $script:Root, "commit-tree", $tree, "-p", $parent, "-m", $Message))
    } finally {
        if ($null -eq $oldIndex) { Remove-Item Env:GIT_INDEX_FILE -ErrorAction SilentlyContinue } else { $env:GIT_INDEX_FILE = $oldIndex }
    }
    Invoke-Native git @("-C", $script:Root, "push", "-u", "origin", "${commit}:refs/heads/$NewBranch")
    return $commit
}

# One file from a commit in the repository's history (for example an old
# nets/default.nnue), extracted (not checked out) into
# sprt\coder2\history\<label>.<extension>. Reused when it already exists.
function Export-HistoricFile([string]$Sha, [string]$RepoPath, [string]$Label) {
    $dir = Get-OutDir "history"
    $ext = [IO.Path]::GetExtension($RepoPath)
    $file = Join-Path $dir "$Label$ext"
    if (Test-Path -LiteralPath $file) { return $file }
    $zip = Join-Path $dir "$Label.zip"
    Invoke-Native git @("-C", $script:Root, "archive", "--format=zip", "--output=$zip", $Sha, "--", $RepoPath)
    $unpack = Join-Path $dir "$Label-unpacked"
    Expand-Archive -Path $zip -DestinationPath $unpack -Force
    $found = Join-Path $unpack $RepoPath
    if (-not (Test-Path -LiteralPath $found)) { Stop-WithMessage "Could not extract $RepoPath from commit $Sha." }
    Copy-Item -LiteralPath $found -Destination $file -Force
    return $file
}

# Commit one file to a branch on GitHub (for example main) without checking
# anything out: a private index file, git's object store, one commit on top
# of the branch's current tip, one push. Coder 2 may push to main only
# NETWORK.md and files under tools/net (CLAUDE.md), so RepoPath must be
# such a path. Fails if the branch moved between fetch and push (rerun).
function Publish-FileToBranch([string]$TargetBranch, [string]$RepoPath, [string]$File, [string]$Message) {
    if ($TargetBranch -eq "main" -and -not ($RepoPath -eq "NETWORK.md" -or $RepoPath -like "tools/net/*")) {
        Stop-WithMessage "Coder 2's scripts may write only NETWORK.md and tools/net/* to main (asked: $RepoPath)."
    }
    Write-Step "Publishing $RepoPath on GitHub ($TargetBranch)"
    Invoke-Native git @("-C", $script:Root, "fetch", "origin", $TargetBranch)
    $index = Join-Path (Get-OutDir "publish") "$($RepoPath -replace '[^A-Za-z0-9._-]', '_').index"
    $oldIndex = $env:GIT_INDEX_FILE
    $env:GIT_INDEX_FILE = $index
    try {
        Invoke-Native git @("-C", $script:Root, "read-tree", "origin/$TargetBranch")
        $blob = Get-NativeOutput git @("-C", $script:Root, "hash-object", "-w", $File)
        Invoke-Native git @("-C", $script:Root, "update-index", "--add", "--cacheinfo", "100644,$blob,$RepoPath")
        $tree = Get-NativeOutput git @("-C", $script:Root, "write-tree")
        $parent = Get-NativeOutput git @("-C", $script:Root, "rev-parse", "origin/$TargetBranch")
        $commit = Get-NativeOutput git ($script:CommitIdentity + @("-C", $script:Root, "commit-tree", $tree, "-p", $parent, "-m", $Message))
    } finally {
        if ($null -eq $oldIndex) { Remove-Item Env:GIT_INDEX_FILE -ErrorAction SilentlyContinue } else { $env:GIT_INDEX_FILE = $oldIndex }
    }
    Invoke-Native git @("-C", $script:Root, "push", "origin", "${commit}:refs/heads/$TargetBranch")
    return $commit
}
