# Shared helpers for Trinity's Windows scripts.
# Written for Windows PowerShell 5.1 (built into Windows 10); also runs on
# PowerShell 7 on Linux, which is how these scripts are tested.

$ErrorActionPreference = "Stop"
$script:OnWindows = ($env:OS -eq "Windows_NT")
$script:Exe = if ($script:OnWindows) { ".exe" } else { "" }
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
        & $Program @Arguments
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
# While a long job runs, a background thread tells Windows every 30 seconds
# that the system is busy (SetThreadExecutionState with ES_SYSTEM_REQUIRED).
# Each call only resets the idle-sleep countdown; nothing is changed
# permanently. When the job ends - or the window is closed - the calls
# stop, and Windows goes back to its normal sleep timer by itself. The
# screen is still allowed to turn off.
# ---------------------------------------------------------------------------
function Start-KeepAwake {
    if (-not $script:OnWindows) { return $null }
    $ps = [PowerShell]::Create()
    [void]$ps.AddScript({
        Add-Type -Namespace TrinityPower -Name Native -MemberDefinition @"
[System.Runtime.InteropServices.DllImport("kernel32.dll")]
public static extern uint SetThreadExecutionState(uint esFlags);
"@
        $ES_SYSTEM_REQUIRED = [uint32]1
        while ($true) {
            [void][TrinityPower.Native]::SetThreadExecutionState($ES_SYSTEM_REQUIRED)
            Start-Sleep -Seconds 30
        }
    })
    [void]$ps.BeginInvoke()
    Write-Host "(Sleep is paused while this runs; it resumes automatically when finished.)" -ForegroundColor DarkGray
    return $ps
}

function Stop-KeepAwake($Handle) {
    if ($null -ne $Handle) {
        try { $Handle.Stop(); $Handle.Dispose() } catch { }
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
    $exe = Join-Path $engines "trinity-$safe-$sha$($script:Exe)"
    if (Test-Path $exe) {
        Write-Host "Using cached build of $Ref ($sha)."
        return $exe
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
    $built = Join-Path (Join-Path (Join-Path $script:Work "target") "release") "trinity$($script:Exe)"
    Copy-Item $built $exe -Force
    Invoke-Native git @("-C", $script:Root, "worktree", "remove", "--force", $tree)
    return $exe
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
function Get-Fastchess {
    $dir = Join-Path $script:Work "tools"
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    $found = Get-ChildItem -Path $dir -Recurse -Filter "fastchess$($script:Exe)" -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($found) { return $found.FullName }
    if (-not $script:OnWindows) { Stop-WithMessage "Put a fastchess binary in $dir (automatic download is Windows-only)." }
    Write-Step "Downloading fastchess (the program that runs test matches)"
    $zip = Join-Path $dir "fastchess.zip"
    Save-Download $script:FastchessUrl $zip
    Expand-Archive -Path $zip -DestinationPath $dir -Force
    Remove-Item $zip
    $found = Get-ChildItem -Path $dir -Recurse -Filter "fastchess.exe" | Select-Object -First 1
    if (-not $found) { Stop-WithMessage "fastchess.exe not found after download." }
    return $found.FullName
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
