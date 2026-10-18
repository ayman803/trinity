# Trinity job runner (CLAUDE.md, "PC job runner"; docs/runner-design.md).
#
# The copy that runs is C:\TrinityRunner\runner.ps1, installed by Ayman with
# Install-Runner.bat; changes to this file on GitHub reach the machine only
# when he installs again. It runs as the standard account trinity-runner,
# started by Windows at boot, in its own clone C:\TrinityRunner\repo.
# Every few minutes it reads runner/queue.txt from the latest main, checks
# out exactly the commit Ayman approved (C:\TrinityRunner\approved.txt) and
# verifies its fingerprint, runs the first queued job for this machine with
# self-update off, and logs the result in the trinity-results repository.
# It writes only in C:\TrinityRunner\repo and C:\TrinityRunner\state.

param(
    [string]$RunnerDir = "C:\TrinityRunner",
    [string]$Data = "C:\Trinity",       # Ayman's folder: read-only for this account
    [int]$PollSeconds = 300,
    [switch]$Once                       # one cycle, then exit (for testing)
)

$ErrorActionPreference = "Stop"
$StateDir = Join-Path $RunnerDir "state"
$Clone = Join-Path $RunnerDir "repo"
$TrinityUrl = "https://github.com/ayman803/trinity.git"
$ResultsUrl = "https://github.com/ayman803/trinity-results.git"
$ResultsDir = Join-Path $StateDir "results"
$Fingerprinted = @("scripts", "tools", "runner/jobs")
# Scripts that publish into trinity-results under the runner (the others
# would fail at the end, after hours of work, pushing to trinity).
$Ready = @("runner/jobs/hello.ps1", "scripts/calibrate.ps1", "scripts/timeuse.ps1", "scripts/forensics.ps1",
    "scripts/v2check.ps1", "scripts/threads.ps1", "tools/net/quantcheck.ps1", "tools/net/netloss.ps1")
# Read-only links from the clone to Ayman's data (reads only; Windows refuses writes).
$Links = @("data", "checkpoints", "syzygy", "sprt\coder2\data")
$ArgPattern = '^[A-Za-z0-9_.+/:=-]+$'
$IdPattern = '^[A-Za-z0-9_.-]+$'
$MinFreeStartGB = 50
$MinFreeRunGB = 30
$env:GIT_TERMINAL_PROMPT = "0"    # git never waits for typed input: nobody is there
$env:GCM_INTERACTIVE = "never"
$Identity = @("-c", "user.name=ayman803", "-c", "user.email=7761579+ayman803@users.noreply.github.com")
New-Item -ItemType Directory -Force -Path $StateDir | Out-Null
$Machine = (Get-Content -LiteralPath (Join-Path $RunnerDir "machine.txt") -TotalCount 1).Trim()
$JobsFile = Join-Path $StateDir "jobs.txt"      # "<id> <STATUS ...>", one line per change
$script:LastNote = ""

function Write-Log([string]$Text) {
    $line = "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') $Text"
    Add-Content -LiteralPath (Join-Path $StateDir "runner.log") -Value $line
    Write-Host $line
}

# Log a waiting reason once, not every cycle.
function Write-Note([string]$Text) {
    if ($Text -ne $script:LastNote) { Write-Log $Text; $script:LastNote = $Text }
}

function Invoke-Git([string]$Dir, [string[]]$GitArgs) {
    $old = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        $out = & git -C $Dir @GitArgs 2>&1 | ForEach-Object { "$_" }
        $code = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $old
    }
    if ($code -ne 0) { throw "git $($GitArgs -join ' ') failed: $($out -join ' ')" }
    return $out
}

function Get-FreeGB {
    return [Math]::Floor(([IO.DriveInfo]::new([IO.Path]::GetPathRoot($RunnerDir))).AvailableFreeSpace / 1GB)
}

# Fingerprint of everything a job can run at a commit: the list of files
# and their contents (git object ids) under scripts/, tools/, runner/jobs/.
# Approve-Scripts.ps1 computes it the same way.
function Get-Fingerprint([string]$Dir, [string]$Commit) {
    $list = Invoke-Git $Dir (@("ls-tree", "-r", $Commit, "--") + $Fingerprinted)
    $sha = [Security.Cryptography.SHA256]::Create()
    $hash = $sha.ComputeHash([Text.Encoding]::UTF8.GetBytes(($list -join "`n")))
    return ([BitConverter]::ToString($hash) -replace "-", "")
}

# approved.txt: "commit | <sha> | <fingerprint>" and "limit | <script> | <hours>".
function Get-Approval {
    $a = [pscustomobject]@{ Commit = ""; Fingerprint = ""; Limits = @{} }
    $file = Join-Path $RunnerDir "approved.txt"
    if (-not (Test-Path -LiteralPath $file)) { return $a }
    foreach ($l in Get-Content -LiteralPath $file) {
        $f = @($l -split '\|' | ForEach-Object { $_.Trim() })
        if ($f.Count -lt 3) { continue }
        if ($f[0] -eq "commit") { $a.Commit = $f[1]; $a.Fingerprint = $f[2] }
        elseif ($f[0] -eq "limit") { $a.Limits[$f[1]] = [double]$f[2] }
    }
    return $a
}

# The queue comes from the latest main: "id | machine | script | arguments".
function Get-Queue {
    $jobs = @()
    $text = Invoke-Git $Clone @("show", "origin/main:runner/queue.txt")
    foreach ($l in $text) {
        if ($l -match '^\s*#' -or -not $l.Trim()) { continue }
        $f = @($l -split '\|' | ForEach-Object { $_.Trim() })
        if ($f.Count -lt 3) { continue }
        $jobArgs = @()
        if ($f.Count -ge 4 -and $f[3]) { $jobArgs = @($f[3] -split '\s+' | Where-Object { $_ }) }
        $jobs += [pscustomobject]@{ Id = $f[0]; Machine = $f[1]; Script = $f[2]; Args = $jobArgs }
    }
    return $jobs
}

function Get-JobStatus {
    $status = @{}
    if (Test-Path -LiteralPath $JobsFile) {
        foreach ($l in Get-Content -LiteralPath $JobsFile) {
            $id, $rest = $l -split ' ', 2
            if ($id) { $status[$id] = $rest }
        }
    }
    return $status
}

function Set-JobStatus([string]$Id, [string]$Status) {
    Add-Content -LiteralPath $JobsFile -Value "$Id $Status"
}

# Append a line to runner/log/<machine>.txt in trinity-results (and the
# job's output to runner/output/<id>.txt), then push.
function Publish-Log([string]$Line, [string]$JobId = "", [string[]]$Output = @()) {
    for ($try = 1; $try -le 3; $try++) {
        try {
            if (-not (Test-Path -LiteralPath (Join-Path $ResultsDir ".git"))) { Invoke-Git $StateDir @("clone", "-q", $ResultsUrl, $ResultsDir) | Out-Null }
            Invoke-Git $ResultsDir @("pull", "-q", "--rebase", "origin", "main") | Out-Null
            $logRel = "runner/log/$Machine.txt"
            $logPath = Join-Path $ResultsDir $logRel
            New-Item -ItemType Directory -Force -Path (Split-Path $logPath) | Out-Null
            Add-Content -LiteralPath $logPath -Value "$(Get-Date -Format 'yyyy-MM-dd HH:mm') $Line"
            Invoke-Git $ResultsDir @("add", $logRel) | Out-Null
            if ($JobId -and $Output.Count -gt 0) {
                $outRel = "runner/output/$JobId.txt"
                $outPath = Join-Path $ResultsDir $outRel
                New-Item -ItemType Directory -Force -Path (Split-Path $outPath) | Out-Null
                Set-Content -LiteralPath $outPath -Value $Output
                Invoke-Git $ResultsDir @("add", $outRel) | Out-Null
            }
            Invoke-Git $ResultsDir ($Identity + @("commit", "-q", "-m", "Runner ($Machine): $Line")) | Out-Null
            Invoke-Git $ResultsDir @("push", "-q", "origin", "HEAD:main") | Out-Null
            return
        } catch {
            Write-Log "Could not save the log in trinity-results (try $try of 3): $($_.Exception.Message)"
            try { Invoke-Git $ResultsDir @("reset", "-q", "--hard", "origin/main") | Out-Null } catch { }
            Start-Sleep -Seconds 20
        }
    }
}

# Windows wants to restart (an update): start nothing until Ayman restarts.
function Test-RestartPending {
    return (Test-Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired") -or
        (Test-Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending")
}

# Ayman's own Trinity windows (or anything else of Trinity's) running.
function Test-Busy {
    return [bool](Get-Process -Name @("trinity*", "fastchess*", "bullet-utils*", "cargo", "rustc") -ErrorAction SilentlyContinue)
}

# The runner's own clone, with read-only links to Ayman's big folders.
function Initialize-Clone {
    if (-not (Test-Path -LiteralPath (Join-Path $Clone ".git"))) {
        Write-Log "Making the runner's own copy of trinity (one time)"
        Invoke-Git $RunnerDir @("clone", "-q", $TrinityUrl, $Clone) | Out-Null
    }
    foreach ($l in $Links) {
        $link = Join-Path $Clone $l
        $target = Join-Path $Data $l
        if ((Test-Path -LiteralPath $target) -and -not (Test-Path -LiteralPath $link)) {
            New-Item -ItemType Directory -Force -Path (Split-Path $link) | Out-Null
            & cmd.exe /c mklink /J "$link" "$target" | Out-Null
        }
    }
}

# Check out exactly the approved commit. Returns $false (and says why) if
# there is no valid approval.
function Select-ApprovedCommit($Approval) {
    if (-not $Approval.Commit) { Write-Note "Waiting: nothing approved yet; Ayman runs Approve-Scripts."; return $false }
    $fp = Get-Fingerprint $Clone $Approval.Commit
    if ($fp -ne $Approval.Fingerprint) { Write-Note "Waiting: the approved commit's fingerprint does not match; Ayman runs Approve-Scripts again."; return $false }
    Invoke-Git $Clone @("checkout", "-q", "--force", "--detach", $Approval.Commit) | Out-Null
    return $true
}

function Stop-Tree([int]$ProcessId) {
    $old = $ErrorActionPreference; $ErrorActionPreference = "Continue"
    try { & taskkill.exe /PID $ProcessId /T /F 2>&1 | Out-Null } finally { $ErrorActionPreference = $old }
}

function Invoke-Job($Job, $Hours) {
    $scriptPath = Join-Path $Clone $Job.Script
    $runArgs = @($Job.Args)
    if ((Get-Content -LiteralPath $scriptPath -Raw) -match '\[switch\]\$NoUpdate') { $runArgs += "-NoUpdate" }
    $ps = Join-Path $env:SystemRoot "System32\WindowsPowerShell\v1.0\powershell.exe"
    $out = Join-Path $StateDir "job-$($Job.Id).out.txt"
    $err = Join-Path $StateDir "job-$($Job.Id).err.txt"
    $psArgs = @("-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File", $scriptPath) + $runArgs
    Set-JobStatus $Job.Id "STARTED"
    Write-Log "Starting $($Job.Id): $($Job.Script) $($runArgs -join ' ') (limit $Hours h)"
    Publish-Log "STARTED $($Job.Id): $($Job.Script) $($runArgs -join ' ')"
    $started = Get-Date
    $env:TRINITY_RESULTS_DIR = $ResultsDir   # job scripts publish into trinity-results (scripts/common.ps1)
    $env:TRINITY_NO_UPDATE = "1"             # and never pull newer code mid-job
    $p = Start-Process -FilePath $ps -ArgumentList $psArgs -WorkingDirectory $Clone -PassThru -NoNewWindow `
        -RedirectStandardOutput $out -RedirectStandardError $err
    $null = $p.Handle   # keeps the exit code available after the process ends
    $deadline = $started.AddHours($Hours)
    $reason = ""
    while (-not $p.HasExited) {
        Start-Sleep -Seconds 30
        if ((Get-Date) -gt $deadline) { $reason = "STOPPED (over the $Hours h limit)"; Stop-Tree $p.Id; break }
        if ((Get-FreeGB) -lt $MinFreeRunGB) { $reason = "STOPPED (under $MinFreeRunGB GB free)"; Stop-Tree $p.Id; break }
    }
    $p.WaitForExit()
    $minutes = [Math]::Round(((Get-Date) - $started).TotalMinutes)
    $status = if ($reason) { $reason } elseif ($p.ExitCode -eq 0) { "DONE" } else { "FAILED (exit code $($p.ExitCode))" }
    Set-JobStatus $Job.Id $status
    Write-Log "$status $($Job.Id) after $minutes min"
    $tail = @()
    foreach ($f in @($out, $err)) { if (Test-Path -LiteralPath $f) { $tail += @(Get-Content -LiteralPath $f -Tail 80) } }
    Publish-Log "$status $($Job.Id) after $minutes min" $Job.Id $tail
}

function Invoke-Cycle {
    if (Test-Path -LiteralPath (Join-Path $RunnerDir "STOP")) { Write-Note "Stopped by the STOP switch; starting nothing new."; return $false }
    if (Test-RestartPending) { Write-Note "Waiting: Windows wants to restart; restart the machine when convenient."; return $false }
    if ((Get-FreeGB) -lt $MinFreeStartGB) { Write-Note "Waiting: under $MinFreeStartGB GB free on the disk."; return $false }
    if (Test-Busy) { Write-Note "Waiting: another Trinity program is running on this machine."; return $false }
    Initialize-Clone
    Invoke-Git $Clone @("fetch", "-q", "--prune", "origin") | Out-Null
    $status = Get-JobStatus
    $next = $null
    foreach ($job in Get-Queue) {
        if ($job.Machine -ne $Machine -or $status.ContainsKey($job.Id)) { continue }
        $next = $job
        break
    }
    if (-not $next) { Write-Note "Queue empty for '$Machine'."; return $false }
    $approval = Get-Approval
    if (-not (Select-ApprovedCommit $approval)) { return $false }
    $problem = ""
    if ($next.Id -notmatch $IdPattern) { $problem = "bad job id" }
    elseif ($Ready -notcontains $next.Script) { $problem = "script not cleared for the runner: $($next.Script)" }
    elseif (-not (Test-Path -LiteralPath (Join-Path $Clone $next.Script))) { $problem = "script not in the approved commit: $($next.Script)" }
    else { foreach ($a in $next.Args) { if ($a -notmatch $ArgPattern) { $problem = "argument not allowed: $a" } } }
    if ($problem) {
        Set-JobStatus $next.Id "REFUSED ($problem)"
        Write-Log "REFUSED $($next.Id): $problem"
        Publish-Log "REFUSED $($next.Id): $problem"
        return $true
    }
    if (-not $approval.Limits.ContainsKey($next.Script)) {
        Write-Note "Waiting: no time limit approved for $($next.Script) (job $($next.Id)); Ayman runs Approve-Scripts."
        return $false
    }
    $script:LastNote = ""
    Invoke-Job $next $approval.Limits[$next.Script]
    return $true
}

# --- first start for this account: tools and git settings -----------------
function Initialize-Account {
    $creds = (Join-Path $RunnerDir "secret\git-credentials") -replace '\\', '/'
    # Only the stored key: an empty entry first switches off Git for Windows'
    # own credential manager, which would wait for a sign-in window nobody sees.
    & git config --global --unset-all credential.helper
    $empty = if ($PSVersionTable.PSVersion.Major -le 5) { '""' } else { "" }   # Windows PowerShell drops a bare empty argument
    & git config --global --add credential.helper $empty
    & git config --global --add credential.helper "store --file=$creds"
    & git config --global core.autocrlf true
    # The installer (an administrator) created the clone's folder, so git
    # would refuse to work in it without this.
    $safe = $Clone -replace '\\', '/'
    if (@(& git config --global --get-all safe.directory) -notcontains $safe) { & git config --global --add safe.directory $safe }
    $cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
    if (-not ($env:PATH -split ';' -contains $cargoBin)) { $env:PATH = "$cargoBin;$env:PATH" }
    if (-not (Test-Cargo)) {
        Write-Log "Installing Rust for this account (one time)"
        $init = Join-Path $StateDir "rustup-init.exe"
        Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $init -UseBasicParsing
        $old = $ErrorActionPreference
        $ErrorActionPreference = "Continue"     # rustup reports progress on stderr
        try { & $init -y --profile minimal --default-toolchain stable 2>&1 | Out-Null } finally { $ErrorActionPreference = $old }
        if (-not (Test-Cargo)) { throw "Rust could not be installed for $env:USERNAME." }
        Write-Log "Rust installed"
    }
}

# True if cargo runs (a half-finished install has cargo but no toolchain).
function Test-Cargo {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { return $false }
    $old = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try { & cargo --version 2>&1 | Out-Null; return ($LASTEXITCODE -eq 0) } finally { $ErrorActionPreference = $old }
}

# Any error that would stop the runner is written to the log first.
trap { Write-Log "Runner stopped by an error: $($_.Exception.Message)"; break }
Write-Log "Runner started on '$Machine' as $env:USERNAME"
# Keep the machine awake while the runner is on (ES_CONTINUOUS | ES_SYSTEM_REQUIRED).
if ($env:OS -eq "Windows_NT") {
    try {
        Add-Type -Namespace TrinityRunner -Name Power -MemberDefinition '[DllImport("kernel32.dll")] public static extern uint SetThreadExecutionState(uint esFlags);'
        [TrinityRunner.Power]::SetThreadExecutionState([uint32]2147483649) | Out-Null
    } catch { Write-Log "Could not keep the machine awake: $($_.Exception.Message)" }
}
try { Initialize-Account } catch { Write-Log "Setup problem: $($_.Exception.Message)" }
# A job that was running when the machine went down is not restarted by
# itself; it is reported, and the Session decides whether to queue it again.
$status = Get-JobStatus
foreach ($id in @($status.Keys)) {
    if ($status[$id] -eq "STARTED") {
        Set-JobStatus $id "INTERRUPTED"
        Write-Log "INTERRUPTED $id (the runner stopped while it ran)"
        Publish-Log "INTERRUPTED $id (the runner stopped while it ran)"
    }
}
while ($true) {
    $ran = $false
    try { $ran = Invoke-Cycle } catch { Write-Log "Cycle error: $($_.Exception.Message)" }
    if ($Once) { break }
    Start-Sleep -Seconds $(if ($ran) { 30 } else { $PollSeconds })
}
