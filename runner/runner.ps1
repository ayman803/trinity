# Trinity job runner (CLAUDE.md, "PC job runner"; docs/runner-design.md).
#
# The copy that runs is C:\TrinityRunner\runner.ps1, installed by Ayman with
# Install-Runner.bat; changes to this file on GitHub reach the machine only
# when he installs again. It runs as the standard account trinity-runner,
# started by Windows at boot, in its own clone C:\TrinityRunner\repo.
# Every few minutes while the PC is awake it reads runner/queue.txt from the latest main, checks
# out exactly the commit Ayman approved (C:\TrinityRunner\approved.txt) and
# verifies its fingerprint, runs the first queued job for this machine with
# self-update off, and logs the result in the trinity-results repository.
# It writes only in C:\TrinityRunner\repo and C:\TrinityRunner\state.

param(
    [string]$RunnerDir = "C:\TrinityRunner",
    [string]$Data = "C:\Trinity",       # Ayman's folder: read-only for this account
    [int]$PollSeconds = 300,
    # Ayman needs the PC at 7 am every day (owner, 21 October): a job starts
    # only if it should end by this time; resumable tuning fills the gap.
    [string]$MorningDeadline = "06:55",
    [string]$MorningBack = "07:30",     # no new job between the deadline and this time
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
    "scripts/v2check.ps1", "scripts/threads.ps1", "tools/net/quantcheck.ps1", "tools/net/netloss.ps1",
    "scripts/sprt.ps1", "tools/net/kb16.ps1")
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
    $script:Disturbed = @{}
    if (Test-Path -LiteralPath $JobsFile) {
        foreach ($l in Get-Content -LiteralPath $JobsFile) {
            $id, $rest = $l -split ' ', 2
            if ($id) { $status[$id] = $rest }
            if ($id -and $rest -like "DISTURBED*") { $script:Disturbed[$id] = 1 + $(if ($script:Disturbed.ContainsKey($id)) { $script:Disturbed[$id] } else { 0 }) }
        }
    }
    return $status
}

# A job still to run: never started, or stopped because Ayman came back
# (at most 3 such stops; after that it is reported and left).
function Test-JobPending($Status, [string]$Id) {
    if (-not $Status.ContainsKey($Id)) { return $true }
    return ($Status[$Id] -like "DISTURBED*" -and $script:Disturbed[$Id] -lt 3)
}

function Set-JobStatus([string]$Id, [string]$Status) {
    Add-Content -LiteralPath $JobsFile -Value "$Id $Status"
}

# Append a line to runner/log/<machine>.txt in trinity-results (and the
# job's output to runner/output/<id>.txt), then push.
# trinity-results is public: no user names, personal paths or device ids.
function Get-Scrubbed([string]$Text) {
    $t = $Text -replace '(?i)C:\\Users\\[^\\\s]+', 'C:\Users\<user>' -replace '(?i)GPU-[0-9a-f-]{36}', 'GPU-<id>'
    if ($env:COMPUTERNAME) { $t = $t -replace [regex]::Escape($env:COMPUTERNAME), '<pc>' }
    return $t
}

function Publish-Log([string]$Line, [string]$JobId = "", [string[]]$Output = @()) {
    $Line = Get-Scrubbed $Line
    $Output = @($Output | ForEach-Object { Get-Scrubbed $_ })
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
    # Calibration opponents Ayman saved by hand: refreshed into the clone (the
    # job may add its own builds there) from the installer's copy in
    # $RunnerDir\opponents (this account cannot read Ayman's folder), compared by
    # fingerprint so a changed opponent is never left stale.
    $from = Join-Path $RunnerDir "opponents"
    if (-not (Test-Path -LiteralPath $from)) { $from = Join-Path $Data "sprt\opponents" }
    $to = Join-Path $Clone "sprt\opponents"
    try {
        if (Test-Path -LiteralPath $from) {
            New-Item -ItemType Directory -Force -Path $to | Out-Null
            foreach ($f in Get-ChildItem -LiteralPath $from -Filter "*.exe" -File) {
                $dest = Join-Path $to $f.Name
                $same = (Test-Path -LiteralPath $dest) -and
                    ((Get-FileHash -LiteralPath $dest -Algorithm SHA256).Hash -eq (Get-FileHash -LiteralPath $f.FullName -Algorithm SHA256).Hash)
                if (-not $same) { Copy-Item -LiteralPath $f.FullName -Destination $dest -Force; Write-Log "Calibration opponent refreshed: $($f.Name)" }
            }
        }
    } catch {
        Write-Note "Could not refresh the calibration opponents from ${from}: $($_.Exception.Message). Calibrations use the copies already in the runner's folder; reinstall the runner to refresh them."
    }
}

# Minutes since Ayman last used the keyboard or mouse, from the idle beacon
# that runs in his own account (runner\beacon.ps1, task "Trinity Idle
# Beacon"; this account can only read its file). $null when the beacon has
# not written for 3 minutes (not signed in, or the beacon stopped): unknown.
$BeaconFile = Join-Path $RunnerDir "beacon\last-input.txt"
function Get-IdleMinutes {
    try {
        if (-not (Test-Path -LiteralPath $BeaconFile)) { return $null }
        $parts = ([IO.File]::ReadAllText($BeaconFile)).Trim() -split ' '
        if ($parts.Count -ne 2) { return $null }
        $styles = [Globalization.DateTimeStyles]::AdjustToUniversal -bor [Globalization.DateTimeStyles]::AssumeUniversal
        $last = [DateTime]::Parse($parts[0], [Globalization.CultureInfo]::InvariantCulture, $styles)
        $written = [DateTime]::Parse($parts[1], [Globalization.CultureInfo]::InvariantCulture, $styles)
        $now = [DateTime]::UtcNow
        if (($now - $written).TotalMinutes -gt 3) { return $null }
        return [Math]::Max(0, ($now - $last).TotalMinutes)
    } catch { return $null }
}

# Real jobs start only when the PC is free for them (owner and advisor,
# 19 October): 30 minutes without keyboard or mouse use, at any hour. If
# the beacon is silent (idle time unknown), only between 11 pm and 7 am.
function Test-PcFree {
    $idle = Get-IdleMinutes
    if ($null -ne $idle) { return ($idle -ge 30) }
    Write-Note "Idle time unknown (the idle beacon has not written for 3 minutes); real jobs wait for 11 pm."
    $h = (Get-Date).Hour
    return ($h -ge 23 -or $h -lt 7)
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

function Invoke-Job($Job, $Hours, [string]$Commit) {
    $short = $Commit.Substring(0, [Math]::Min(10, $Commit.Length))
    $scriptPath = Join-Path $Clone $Job.Script
    $runArgs = @($Job.Args)
    if ((Get-Content -LiteralPath $scriptPath -Raw) -match '\[switch\]\$NoUpdate') { $runArgs += "-NoUpdate" }
    $ps = Join-Path $env:SystemRoot "System32\WindowsPowerShell\v1.0\powershell.exe"
    $out = Join-Path $StateDir "job-$($Job.Id).out.txt"
    $err = Join-Path $StateDir "job-$($Job.Id).err.txt"
    $psArgs = @("-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File", $scriptPath) + $runArgs
    Set-Awake $true
    Set-JobStatus $Job.Id "STARTED"
    Write-Log "Starting $($Job.Id): $($Job.Script) $($runArgs -join ' ') (limit $Hours h)"
    Publish-Log "STARTED $($Job.Id): $($Job.Script) $($runArgs -join ' ') (approved commit $short)"
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
        # The PC is Ayman's again as soon as he uses it: the job stops, does
        # not count, and runs again from the start later (at most 3 times).
        $idle = Get-IdleMinutes
        if ($Job.Script -ne "runner/jobs/hello.ps1" -and $null -ne $idle -and $idle -lt 1) {
            $reason = "DISTURBED (keyboard or mouse used; stopped, not counted, runs again later)"
            Stop-Tree $p.Id
            break
        }
    }
    $p.WaitForExit()
    if ($reason -and $Job.Script -eq "scripts/calibrate.ps1") {
        # A stopped calibration's partial games must not be read later as the
        # newest calibration (time use, forensics): moved aside, in our own clone.
        $res = Join-Path $Clone "sprt\results"
        $aside = Join-Path $res "incomplete"
        New-Item -ItemType Directory -Force -Path $aside | Out-Null
        Get-ChildItem -LiteralPath $res -File -Filter "*_calibration_*" -ErrorAction SilentlyContinue |
            Where-Object { $_.LastWriteTime -ge $started } |
            ForEach-Object { Move-Item -LiteralPath $_.FullName -Destination $aside -Force }
    }
    $minutes = [Math]::Round(((Get-Date) - $started).TotalMinutes)
    $status = if ($reason) { $reason } elseif ($p.ExitCode -eq 0) { "DONE" } else { "FAILED (exit code $($p.ExitCode))" }
    Set-JobStatus $Job.Id $status
    Write-Log "$status $($Job.Id) after $minutes min"
    $tail = @()
    foreach ($f in @($out, $err)) { if (Test-Path -LiteralPath $f) { $tail += @(Get-Content -LiteralPath $f -Tail 80) } }
    Publish-Log "$status $($Job.Id) after $minutes min (approved commit $short)" $Job.Id $tail
}

function Invoke-Cycle {
    if (Test-Path -LiteralPath (Join-Path $RunnerDir "STOP")) { Write-Note "Stopped by the STOP switch; starting nothing new."; return $false }
    if (Test-RestartPending) { Write-Note "Waiting: Windows wants to restart; restart the machine when convenient."; return $false }
    if ((Get-FreeGB) -lt $MinFreeStartGB) { Write-Note "Waiting: under $MinFreeStartGB GB free on the disk."; return $false }
    if (Test-Busy) { Write-Note "Waiting: another Trinity program is running on this machine."; return $false }
    Initialize-Clone
    Invoke-Git $Clone @("fetch", "-q", "--prune", "origin") | Out-Null
    $status = Get-JobStatus
    $pending = @(Get-Queue | Where-Object { $_.Machine -eq $Machine -and (Test-JobPending $status $_.Id) })
    if ($pending.Count -eq 0) { Write-Note "Queue empty for '$Machine'."; return $false }
    $approval = Get-Approval
    if (-not (Select-ApprovedCommit $approval)) { return $false }
    # The first queued job that can finish before the morning deadline
    # (the hello test always fits); later resumable tuning may fill the gap.
    if (Test-MorningQuiet) { Write-Note "Waiting: the PC is Ayman's from $MorningDeadline to $MorningBack."; return $false }
    $next = $null
    foreach ($job in $pending) {
        $hours = if ($approval.Limits.ContainsKey($job.Script)) { [double]$approval.Limits[$job.Script] } else { 8 }
        if ($job.Script -eq "runner/jobs/hello.ps1" -or (Test-JobFits $job $hours)) { $next = $job; break }
    }
    if (-not $next) {
        $first = $pending[0]
        Write-Note "Waiting: $($first.Id) would not finish before $MorningDeadline; it starts after $MorningBack."
        return $false
    }
    if ($next.Id -ne $pending[0].Id) { Write-Note "$($pending[0].Id) would not finish before $MorningDeadline; running $($next.Id) first." }
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
    if ($next.Script -ne "runner/jobs/hello.ps1" -and -not (Test-PcFree)) {
        Write-Note "Waiting: $($next.Id) starts after 30 minutes without keyboard or mouse use."
        return $false
    }
    $script:LastNote = ""
    $next = [pscustomobject]@{ Id = $next.Id; Machine = $next.Machine; Script = $next.Script; Args = @(Get-FittedArgs $next) }
    Invoke-Job $next $approval.Limits[$next.Script] $approval.Commit
    return $true
}

# --- the 7 am deadline (owner and advisor, 21 October) ---------------------
# Job "kind": the script and its arguments without the version (-Ref x),
# -NoUpdate and the tuning length (-Minutes x), so that every calibration of
# the same size counts as one kind whatever version it measures.
function Get-JobKind([string]$Script, [string[]]$JobArgs) {
    $kept = @(); $skip = $false
    foreach ($a in $JobArgs) {
        if ($skip) { $skip = $false; continue }
        if ($a -eq "-Ref" -or $a -eq "-Minutes") { $skip = $true; continue }
        if ($a -eq "-NoUpdate") { continue }
        $kept += $a
    }
    return (@($Script) + $kept) -join " "
}

# Expected minutes for a job: the longest of its kind's last 3 finished
# runs in this machine's runner log, plus 10% and 5 minutes; with no
# history, the approved time limit (the safe side).
function Get-ExpectedMinutes($Job, [double]$LimitHours) {
    $kind = Get-JobKind $Job.Script $Job.Args
    $log = Join-Path $StateDir "runner.log"
    $kinds = @{}; $done = @()
    if (Test-Path -LiteralPath $log) {
        foreach ($l in Get-Content -LiteralPath $log) {
            if ($l -match '^\S+ \S+ Starting (\S+): (\S+) ?(.*) \(limit ') {
                $kinds[$Matches[1]] = Get-JobKind $Matches[2] @($Matches[3] -split '\s+' | Where-Object { $_ })
            } elseif ($l -match '^\S+ \S+ DONE (\S+) after (\d+) min') {
                if ($kinds[$Matches[1]] -eq $kind) { $done += [int]$Matches[2] }
            }
        }
    }
    if ($done.Count -eq 0) { return [Math]::Round($LimitHours * 60) }
    $recent = @($done | Select-Object -Last 3)
    return [Math]::Round(($recent | Measure-Object -Maximum).Maximum * 1.1 + 5)
}

# The next morning deadline after now.
function Get-MorningDeadline {
    $now = Get-Date
    $d = [DateTime]::ParseExact($MorningDeadline, "HH:mm", $null)
    $t = $now.Date.AddHours($d.Hour).AddMinutes($d.Minute)
    if ($t -le $now) { $t = $t.AddDays(1) }
    return $t
}

# Between the deadline and $MorningBack: the PC is Ayman's; nothing starts.
function Test-MorningQuiet {
    $now = Get-Date
    $a = [DateTime]::ParseExact($MorningDeadline, "HH:mm", $null)
    $b = [DateTime]::ParseExact($MorningBack, "HH:mm", $null)
    $start = $now.Date.AddHours($a.Hour).AddMinutes($a.Minute)
    $end = $now.Date.AddHours($b.Hour).AddMinutes($b.Minute)
    return ($now -ge $start -and $now -lt $end)
}

# Can this job start now and be done by the deadline? A tuning job
# (calibrate.ps1 -Tune) is resumable: it fits if at least 30 minutes are
# left, and its -Minutes is cut to end by the deadline (Get-FittedArgs).
function Test-JobFits($Job, [double]$LimitHours) {
    $left = ((Get-MorningDeadline) - (Get-Date)).TotalMinutes
    if ($Job.Args -contains "-Tune") { return ($left - 20 -ge 30) }
    return ((Get-ExpectedMinutes $Job $LimitHours) -le $left)
}

function Get-FittedArgs($Job) {
    if ($Job.Args -notcontains "-Tune") { return $Job.Args }
    $left = [Math]::Floor(((Get-MorningDeadline) - (Get-Date)).TotalMinutes) - 20   # build time and the last step
    $out = @(); $skip = $false; $minutes = $left
    for ($i = 0; $i -lt $Job.Args.Count; $i++) {
        if ($skip) { $skip = $false; continue }
        if ($Job.Args[$i] -eq "-Minutes" -and $i + 1 -lt $Job.Args.Count) {
            $minutes = [Math]::Min($left, [int]$Job.Args[$i + 1]); $skip = $true; continue
        }
        $out += $Job.Args[$i]
    }
    return @($out + @("-Minutes", "$minutes"))
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

# ES_CONTINUOUS, plus ES_SYSTEM_REQUIRED while a job runs.
function Set-Awake([bool]$On) {
    if (-not $script:CanHoldAwake) { return }
    $flags = if ($On) { [uint32]2147483649 } else { [uint32]2147483648 }
    [TrinityRunner.Power]::SetThreadExecutionState($flags) | Out-Null
}

# Any error that would stop the runner is written to the log first.
trap { Write-Log "Runner stopped by an error: $($_.Exception.Message)"; break }
Write-Log "Runner started on '$Machine' as $env:USERNAME"
# The PC may sleep when the runner has nothing to do; it is kept awake only
# while a job runs (Set-Awake), and a Windows wake timer (installed task
# "Trinity Runner Wake", 11 pm and 6 am) wakes it to check the queue.
$script:CanHoldAwake = $false
if ($env:OS -eq "Windows_NT") {
    try {
        Add-Type -Namespace TrinityRunner -Name Power -MemberDefinition '[DllImport("kernel32.dll")] public static extern uint SetThreadExecutionState(uint esFlags);'
        $script:CanHoldAwake = $true
    } catch { Write-Log "Cannot hold the PC awake during jobs: $($_.Exception.Message)" }
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
    if (-not $ran) { Set-Awake $false }   # nothing running: Windows may let the PC sleep
    if ($Once) { break }
    # Wait by the wall clock in short steps, so a check follows soon after
    # the PC wakes (Windows puts it back to sleep a few minutes after a timer wake).
    $due = (Get-Date).AddSeconds($(if ($ran) { 30 } else { $PollSeconds }))
    $tick = Get-Date
    while ((Get-Date) -lt $due) {
        Start-Sleep -Seconds 15
        $now = Get-Date
        if (($now - $tick).TotalSeconds -gt 120) { Write-Log "The PC woke from sleep (asleep since about $($tick.ToString('HH:mm')))"; break }
        $tick = $now
    }
}
