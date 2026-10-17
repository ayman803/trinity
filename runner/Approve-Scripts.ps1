# Approve a commit of Trinity's scripts for the runner (Ayman runs this; the
# runner account cannot change approved.txt). Shows what changed in the
# scripts since the last approval, asks, and writes
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
    } else {
        Write-Host "First approval: the runner may run the scripts as they are now on GitHub ($($latest.Substring(0, 10)))." -ForegroundColor Yellow
    }
    Write-Host ""
    Write-Host "Time limits (hours):"
    foreach ($k in $Limits.Keys) { Write-Host "   $k : $($Limits[$k])" }
    $answer = Read-Host "Approve this version for the runner? Type Y and press Enter (anything else cancels)"
    if ($answer.Trim().ToUpper() -ne "Y") { Write-Host "Nothing changed." -ForegroundColor Yellow; exit 0 }
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
