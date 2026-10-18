# Approve a commit of Trinity's scripts for the runner (Ayman runs this; the
# runner account cannot change approved.txt). Shows what changed in the
# scripts since the last approval and lists new risky lines (deletes, moves,
# downloads, git push, Windows settings), asks (at most once a day), and writes
# C:\TrinityRunner\approved.txt. Double-click Approve-Scripts.bat.

param(
    [string]$RunnerDir = "C:\TrinityRunner",
    [string]$Repo = "C:\Trinity"        # Ayman's own copy, used only to read GitHub's history
)

$ErrorActionPreference = "Stop"
$Fingerprinted = @("scripts", "tools", "runner/jobs")
# Time limit per script, in hours (the runner stops a job that runs over).
$Limits = [ordered]@{
    "runner/jobs/hello.ps1" = 0.25; "scripts/timeuse.ps1" = 1; "scripts/v2check.ps1" = 1
    "tools/net/quantcheck.ps1" = 1; "tools/net/netloss.ps1" = 1; "scripts/calibrate.ps1" = 5
    "scripts/forensics.ps1" = 4; "scripts/threads.ps1" = 8
}

function Invoke-Git([string[]]$GitArgs) {
    $old = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        $out = & git -C $Repo @GitArgs 2>&1 | ForEach-Object { "$_" }
        $code = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $old
    }
    if ($code -ne 0) { throw "git $($GitArgs -join ' ') failed: $($out -join ' ')" }
    return $out
}

# Added lines that do something risky, found by the tool itself so Ayman
# does not rely only on Claude's summary (the advisor's rule).
$Risky = [ordered]@{
    "DELETE"   = 'Remove-Item|Clear-Content|\b(rm|del|rmdir|rd)\b|\.Delete\('
    "MOVE"     = 'Move-Item|Rename-Item|\b(move|ren)\b|robocopy'
    "DOWNLOAD" = 'Invoke-WebRequest|Invoke-RestMethod|Start-BitsTransfer|WebClient|DownloadFile|\b(curl|wget)\b|https?://'
    "GIT PUSH" = 'git\b.*\bpush\b|"push"|''push''|\bremote\s+(add|set-url)'
    "WINDOWS"  = 'HKLM:|HKCU:|\breg(\.exe)?\s+(add|delete)|ItemProperty|powercfg|icacls|Set-Acl|ScheduledTask|schtasks|Set-ExecutionPolicy|netsh|Set-Service|bcdedit|MpPreference|RunAs'
}
function Show-RiskyChanges([string]$From, [string]$To) {
    $diff = Invoke-Git (@("diff", "-U0", $From, $To, "--") + $Fingerprinted)
    $file = ""; $found = @()
    foreach ($l in $diff) {
        if ($l -match '^\+\+\+ b/(.*)$') { $file = $Matches[1]; continue }
        if ($l -notmatch '^\+' -or $l -match '^\+\+\+') { continue }
        $code = $l.Substring(1).Trim()
        if ($code.StartsWith("#")) { continue }
        foreach ($k in $Risky.Keys) {
            if ($code -match $Risky[$k]) {
                $short = if ($code.Length -gt 110) { $code.Substring(0, 110) + "..." } else { $code }
                $found += "   [$k] ${file}: $short"
                break
            }
        }
    }
    Write-Host ""
    if ($found.Count -eq 0) { Write-Host "Risky changes: none found (no new deletes, moves, downloads, git push or Windows settings)." -ForegroundColor Green; return }
    Write-Host "Risky changes found by this tool: $($found.Count) new line(s). Check each one is expected:" -ForegroundColor Yellow
    $found | Select-Object -First 40 | ForEach-Object { Write-Host $_ }
    if ($found.Count -gt 40) { Write-Host "   ... and $($found.Count - 40) more." }
}

# Must match Get-Fingerprint in runner.ps1.
function Get-Fingerprint([string]$Commit) {
    $list = Invoke-Git (@("ls-tree", "-r", $Commit, "--") + $Fingerprinted)
    $sha = [Security.Cryptography.SHA256]::Create()
    $hash = $sha.ComputeHash([Text.Encoding]::UTF8.GetBytes(($list -join "`n")))
    return ([BitConverter]::ToString($hash) -replace "-", "")
}

try {
    Write-Host "Reading the latest version from GitHub..." -ForegroundColor Cyan
    Invoke-Git @("fetch", "-q", "origin") | Out-Null
    $latest = (Invoke-Git @("rev-parse", "origin/main")) -join ""
    $file = Join-Path $RunnerDir "approved.txt"
    $current = ""
    if (Test-Path -LiteralPath $file) {
        foreach ($l in Get-Content -LiteralPath $file) {
            $f = @($l -split '\|' | ForEach-Object { $_.Trim() })
            if ($f.Count -ge 3 -and $f[0] -eq "commit") { $current = $f[1] }
        }
    }
    Write-Host ""
    if ($current -eq $latest) {
        Write-Host "Already approved: the runner uses the latest scripts ($($latest.Substring(0, 10)))." -ForegroundColor Green
        exit 0
    }
    if ($current) {
        $changes = @(Invoke-Git (@("diff", "--stat", $current, $latest, "--") + $Fingerprinted))
        if ($changes.Count -eq 0) {
            Write-Host "Newer version on GitHub, but none of the scripts changed." -ForegroundColor Green
        } else {
            Write-Host "Scripts changed since your last approval ($($current.Substring(0, 10)) -> $($latest.Substring(0, 10))):" -ForegroundColor Yellow
            $changes | ForEach-Object { Write-Host "   $_" }
            Write-Host "If you are unsure about a change, paste this list to the advisor or Claude before approving." -ForegroundColor Yellow
        }
        Show-RiskyChanges $current $latest
    } else {
        Write-Host "First approval: the runner may run the scripts as they are now on GitHub ($($latest.Substring(0, 10)))." -ForegroundColor Yellow
        Show-RiskyChanges "4b825dc642cb6eb9a060e54bf8d69288fbee4904" $latest   # git's empty tree: everything counts as new
    }
    Write-Host ""
    Write-Host "Time limits (hours):"
    foreach ($k in $Limits.Keys) { Write-Host "   $k : $($Limits[$k])" }
    # At most one approval a day (the advisor's rule); a second one needs a deliberate word.
    $word = "Y"
    if (Test-Path -LiteralPath $file) {
        $hours = ((Get-Date) - (Get-Item -LiteralPath $file).LastWriteTime).TotalHours
        if ($hours -lt 20) {
            Write-Host ""
            Write-Host ("Your last approval was {0:N0} hour(s) ago. Rule: at most one approval a day." -f $hours) -ForegroundColor Yellow
            $word = "AGAIN"
        }
    }
    $answer = Read-Host "Approve this version for the runner? Type $word and press Enter (anything else cancels)"
    if ($answer.Trim().ToUpper() -ne $word) { Write-Host "Nothing changed." -ForegroundColor Yellow; exit 0 }
    $lines = @(
        "# Written by Approve-Scripts on $(Get-Date -Format 'yyyy-MM-dd HH:mm'). Do not edit by hand.",
        "commit | $latest | $(Get-Fingerprint $latest)"
    )
    foreach ($k in $Limits.Keys) { $lines += "limit | $k | $($Limits[$k])" }
    Set-Content -LiteralPath $file -Value $lines
    Write-Host "Approved. The runner uses this version from its next check (within 5 minutes)." -ForegroundColor Green
} catch {
    Write-Host "PROBLEM: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "Send Claude a screenshot of this window." -ForegroundColor Yellow
    exit 1
}
