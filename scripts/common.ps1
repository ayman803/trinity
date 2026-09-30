# Shared helpers for Trinity's Windows scripts.
# Written for Windows PowerShell 5.1 (built into Windows 10); also runs on
# PowerShell 7 on Linux, which is how these scripts are tested.

$ErrorActionPreference = "Stop"
$script:OnWindows = ($env:OS -eq "Windows_NT")
# Note: PowerShell variable names ignore case, so never name another
# script-level variable $exesuffix / $root / $work.
$script:ExeSuffix = if ($script:OnWindows) { ".exe" } else { "" }
$script:Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$script:Work = Join-Path $script:Root "sprt"   # all downloads and builds live here (ignored by git)

# Pinned downloads (update deliberately).
$script:FastchessVersion = "v1.8.2-alpha"
$script:FastchessUrl = "https://github.com/Disservin/fastchess/releases/download/$($script:FastchessVersion)/fastchess-windows-x86-64.zip"
$script:BookName = "UHO_Lichess_4852_v1.epd"
$script:BookUrl = "https://github.com/official-stockfish/books/raw/master/$($script:BookName).zip"
$script:BulletRev = "c004ebf04025edf91c6ccac47e72d561062a46b5"

# Older Windows PowerShell defaults to TLS 1.0, which GitHub refuses.
try { [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12 } catch { }

# Tools installed by rustup live here; make sure this session can find them
# even if the window was opened before Rust was installed.
$cargoBin = Join-Path $HOME ".cargo/bin"
if ((Test-Path $cargoBin) -and -not (($env:PATH -split [IO.Path]::PathSeparator) -contains $cargoBin)) {
    $env:PATH = $cargoBin + [IO.Path]::PathSeparator + $env:PATH
}

function Write-Step([string]$Text) {
    Write-Host ""
    Write-Host "==> $Text" -ForegroundColor Cyan
}

function Write-Good([string]$Text) { Write-Host $Text -ForegroundColor Green }
function Write-Warn([string]$Text) { Write-Host $Text -ForegroundColor Yellow }

function Stop-WithMessage([string]$Text) {
    Write-Host ""
    Write-Host "PROBLEM: $Text" -ForegroundColor Red
    throw "stopped"
}

# Run a program (git, cargo, ...) and stop if it fails. Native programs
# write progress to stderr, which must not be treated as an error.
function Invoke-Native {
    param([Parameter(Mandatory)][string]$Program, [string[]]$Arguments = @())
    $old = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        # Out-Host: show the output, but never return it as a function result.
        & $Program @Arguments | Out-Host
        $code = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $old
    }
    if ($code -ne 0) {
        Stop-WithMessage "'$Program $($Arguments -join ' ')' failed (exit code $code)."
    }
}

# Same, but return the program's output as text.
function Get-NativeOutput {
    param([Parameter(Mandatory)][string]$Program, [string[]]$Arguments = @())
    $old = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        $out = & $Program @Arguments 2>$null
        $code = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $old
    }
    if ($code -ne 0) {
        Stop-WithMessage "'$Program $($Arguments -join ' ')' failed (exit code $code)."
    }
    return ($out -join "`n").Trim()
}

# Report an unexpected error with enough detail to find its cause.
function Write-UnexpectedError($ErrorRecord) {
    if ($ErrorRecord.Exception.Message -eq "stopped") { return }
    Write-Host "Unexpected error: $($ErrorRecord.Exception.Message)" -ForegroundColor Red
    Write-Host "Where: $($ErrorRecord.InvocationInfo.PositionMessage)" -ForegroundColor Red
    Write-Host "Trace: $($ErrorRecord.ScriptStackTrace)" -ForegroundColor DarkGray
    Write-Host "Please send a screenshot of this window to Claude." -ForegroundColor Yellow
}

function Test-Command([string]$Name) {
    return [bool](Get-Command $Name -ErrorAction SilentlyContinue)
}

function Assert-Tools {
    if (-not (Test-Command "git")) { Stop-WithMessage "Git is not installed. See GUIDE.md, step 1." }
    if (-not (Test-Command "cargo")) { Stop-WithMessage "Rust is not installed. Double-click 1-Setup.bat first." }
    if ($script:Root -match "\s") {
        Stop-WithMessage "The Trinity folder path contains a space ($($script:Root)). Please move it to a path without spaces, e.g. C:\Trinity."
    }
}

function Save-Download([string]$Url, [string]$Path) {
    Write-Host "Downloading $Url"
    $old = $ProgressPreference
    $ProgressPreference = "SilentlyContinue"   # the progress bar makes downloads very slow in PS 5.1
    try {
        Invoke-WebRequest -Uri $Url -OutFile $Path -UseBasicParsing
    } finally {
        $ProgressPreference = $old
    }
}

# ---------------------------------------------------------------------------
# Keeping the PC awake.
#
# Two layers, both using Windows' standard SetThreadExecutionState call:
#  1. This window's own thread tells Windows "the system is required"
#     (ES_CONTINUOUS | ES_SYSTEM_REQUIRED) for as long as the job runs.
#  2. A background thread repeats a one-off "system is busy" signal every
#     30 seconds, which resets the idle-sleep countdown.
# (The engine itself also does (1) while generating data.)
# Nothing is changed permanently: when the job ends, or the window is
# closed, Windows automatically drops these requests and its normal sleep
# timer applies again. The screen may still turn off.
#
# This prevents *sleep*. It cannot stop a Windows Update restart, so we warn
# when an update restart is already pending.
# ---------------------------------------------------------------------------
$script:ES_CONTINUOUS = [uint32]"0x80000000"
$script:ES_SYSTEM_REQUIRED = [uint32]1
if ($script:OnWindows) {
    Add-Type -Namespace TrinityPower -Name Native -MemberDefinition @"
[System.Runtime.InteropServices.DllImport("kernel32.dll")]
public static extern uint SetThreadExecutionState(uint esFlags);
"@
}

function Test-RestartPending {
    if (-not $script:OnWindows) { return $false }
    return (Test-Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired") -or
        (Test-Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending")
}

function Start-KeepAwake {
    if (-not $script:OnWindows) { return $null }
    if (Test-RestartPending) {
        Write-Warn "WARNING: Windows has installed updates and is waiting to restart."
        Write-Warn "It may restart the PC during this job. To be safe: close this window, restart"
        Write-Warn "the PC, then start again. (Or pause updates: Settings > Update & Security.)"
        $answer = Read-Host "Type C and press Enter to continue anyway, or just press Enter to stop"
        if ($answer -ne "C" -and $answer -ne "c") { throw "stopped" }
    }
    $prev = [TrinityPower.Native]::SetThreadExecutionState($script:ES_CONTINUOUS -bor $script:ES_SYSTEM_REQUIRED)
    if ($prev -eq 0) { Write-Warn "Windows refused the keep-awake request; the PC might sleep." }

    $log = Join-Path $script:Work "keep-awake-error.txt"
    $ps = [PowerShell]::Create()
    [void]$ps.AddScript({
        param($LogFile)
        try {
            while ($true) {
                [void][TrinityPower.Native]::SetThreadExecutionState([uint32]1)
                Start-Sleep -Seconds 30
            }
        } catch {
            Set-Content -Path $LogFile -Value $_.Exception.Message
        }
    }).AddArgument($log)
    [void]$ps.BeginInvoke()
    Write-Host "(Sleep is blocked while this runs; normal sleep resumes automatically when finished.)" -ForegroundColor DarkGray
    return $ps
}

function Stop-KeepAwake($Handle) {
    if ($null -ne $Handle) {
        try { $Handle.Stop(); $Handle.Dispose() } catch { }
        try { [void][TrinityPower.Native]::SetThreadExecutionState($script:ES_CONTINUOUS) } catch { }
        Write-Host "(Normal sleep settings are active again.)" -ForegroundColor DarkGray
    }
}

# ---------------------------------------------------------------------------
# Building the engine.
# ---------------------------------------------------------------------------

# Build Trinity from a git ref (branch or commit) into sprt\engines and
# return the path of the executable. Builds are cached by commit.
function Build-Trinity {
    param([Parameter(Mandatory)][string]$Ref)
    $sha = Get-NativeOutput git @("-C", $script:Root, "rev-parse", "--short=10", $Ref)
    $safe = ($Ref -replace "^origin/", "") -replace "[^A-Za-z0-9._-]", "_"
    $engines = Join-Path $script:Work "engines"
    New-Item -ItemType Directory -Force -Path $engines | Out-Null
    $enginePath = Join-Path $engines "trinity-$safe-$sha$($script:ExeSuffix)"
    if (Test-Path $enginePath) {
        Write-Host "Using cached build of $Ref ($sha)."
        return $enginePath
    }
    Write-Step "Building $Ref ($sha)"
    $tree = Join-Path $script:Work "build-$safe"
    if (Test-Path $tree) {
        Invoke-Native git @("-C", $script:Root, "worktree", "remove", "--force", $tree)
    }
    Invoke-Native git @("-C", $script:Root, "worktree", "prune")
    Invoke-Native git @("-C", $script:Root, "worktree", "add", "--detach", $tree, $sha)
    try {
        $oldFlags = $env:RUSTFLAGS
        $env:RUSTFLAGS = "-C target-cpu=native"   # fastest code for this PC
        Invoke-Native cargo @("build", "--release", "--manifest-path", (Join-Path $tree "Cargo.toml"),
            "--target-dir", (Join-Path $script:Work "target"))
    } finally {
        $env:RUSTFLAGS = $oldFlags
    }
    $built = Join-Path (Join-Path (Join-Path $script:Work "target") "release") "trinity$($script:ExeSuffix)"
    Copy-Item $built $enginePath -Force
    Invoke-Native git @("-C", $script:Root, "worktree", "remove", "--force", $tree)
    return $enginePath
}

# Run `bench` and return the node count (the build's "fingerprint").
function Get-BenchNodes([string]$ExePath) {
    $out = Get-NativeOutput $ExePath @("bench")
    if ($out -match "(\d+) nodes (\d+) nps") { return $Matches[1] }
    Stop-WithMessage "bench failed for $ExePath : $out"
}

# ---------------------------------------------------------------------------
# Downloads used for testing.
# ---------------------------------------------------------------------------
# Find files by name below a folder.
function Find-Files([string]$Dir, [string]$Pattern) {
    if (-not [IO.Directory]::Exists($Dir)) { return @() }
    try {
        return [IO.Directory]::GetFiles($Dir, $Pattern, [IO.SearchOption]::AllDirectories)
    } catch {
        Stop-WithMessage "Could not search the folder '$Dir': $($_.Exception.Message)"
    }
}

function Find-File([string]$Dir, [string]$Name) {
    $all = @(Find-Files $Dir $Name)
    if ($all.Count -gt 0) { return $all[0] }
    return $null
}

function Get-Fastchess {
    $dir = Join-Path $script:Work "tools"
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    # The places the release zip unpacks to.
    $candidates = @(
        (Join-Path (Join-Path $dir "fastchess-windows-x86-64") "fastchess$($script:ExeSuffix)"),
        (Join-Path $dir "fastchess$($script:ExeSuffix)")
    )
    foreach ($c in $candidates) { if (Test-Path -LiteralPath $c) { return $c } }
    if (-not $script:OnWindows) { Stop-WithMessage "Put a fastchess binary in $dir (automatic download is Windows-only)." }
    Write-Step "Downloading fastchess (the program that runs test matches)"
    $zip = Join-Path $dir "fastchess.zip"
    Save-Download $script:FastchessUrl $zip
    Expand-Archive -Path $zip -DestinationPath $dir -Force
    Remove-Item $zip
    foreach ($c in $candidates) { if (Test-Path -LiteralPath $c) { return $c } }
    Stop-WithMessage "fastchess.exe not found after download (looked in $($candidates -join ', '))."
}

function Get-OpeningBook {
    $dir = Join-Path $script:Work "books"
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    $book = Join-Path $dir $script:BookName
    if (Test-Path $book) { return $book }
    Write-Step "Downloading the opening book"
    $zip = "$book.zip"
    Save-Download $script:BookUrl $zip
    Expand-Archive -Path $zip -DestinationPath $dir -Force
    Remove-Item $zip
    if (-not (Test-Path $book)) { Stop-WithMessage "Opening book not found after download." }
    return $book
}

# Bring the local copy of the repository up to date with GitHub.
function Update-Repository {
    Write-Step "Getting the latest code from GitHub"
    Invoke-Native git @("-C", $script:Root, "fetch", "--prune", "origin")
    $branch = Get-NativeOutput git @("-C", $script:Root, "rev-parse", "--abbrev-ref", "HEAD")
    $dirty = Get-NativeOutput git @("-C", $script:Root, "status", "--porcelain", "--untracked-files=no")
    if ($branch -eq "main" -and -not $dirty) {
        Invoke-Native git @("-C", $script:Root, "merge", "--ff-only", "origin/main")
    } else {
        Write-Warn "Your folder is not a clean copy of 'main', so it was not updated (tests still use GitHub's latest code)."
    }
}
