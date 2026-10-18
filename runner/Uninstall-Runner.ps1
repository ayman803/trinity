# Remove the Trinity job runner from this machine and undo what
# Install-Runner changed (Ayman runs it as administrator by double-clicking
# runner\Uninstall-Runner.bat):
#  - stops and removes the "Trinity Runner" and "Trinity Runner Wake" tasks and the trinity-runner
#    account (and its hidden-account entry and batch logon right), deletes
#    the stored GitHub key;
#  - removes the account's no-write rule on C:\Trinity;
#  - puts back the sleep and Windows Update settings saved at install time;
#  - removes the desktop shortcuts.
# Kept on purpose: C:\TrinityRunner (logs; delete it by hand if wanted) and
# the tighter C:\Trinity permissions (other accounts no longer get write
# access by default; Ayman keeps full control).

param([string]$Repo = "C:\Trinity", [string]$RunnerDir = "C:\TrinityRunner")

$principal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`""
    exit
}
$Account = "trinity-runner"

# Add or remove an account in the "Log on as a batch job" right, which a
# task that runs whether or not anyone is signed in needs.
function Set-BatchLogon([string]$Sid, [bool]$Allow) {
    $tmp = Join-Path $env:TEMP "trinity-rights"
    Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Force -Path $tmp | Out-Null
    & secedit.exe /export /cfg "$tmp\now.inf" /areas USER_RIGHTS | Out-Null
    $line = (Get-Content -LiteralPath "$tmp\now.inf" | Where-Object { $_ -match '^SeBatchLogonRight\s*=' }) -replace '^SeBatchLogonRight\s*=\s*', ''
    $list = @("$line" -split ',' | ForEach-Object { $_.Trim() } | Where-Object { $_ -and $_ -ne "*$Sid" })
    if ($Allow) { $list += "*$Sid" }
    Set-Content -LiteralPath "$tmp\new.inf" -Encoding Unicode -Value @("[Unicode]", "Unicode=yes", "[Version]", 'signature="$CHICAGO$"', "Revision=1",
        "[Privilege Rights]", "SeBatchLogonRight = $($list -join ',')")
    & secedit.exe /configure /db "$tmp\new.sdb" /cfg "$tmp\new.inf" /areas USER_RIGHTS /quiet | Out-Null
    $code = $LASTEXITCODE
    Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
    if ($code -ne 0) { throw "secedit could not change the batch logon right (exit code $code)" }
}

Stop-ScheduledTask -TaskName "Trinity Runner" -ErrorAction SilentlyContinue
Unregister-ScheduledTask -TaskName "Trinity Runner" -Confirm:$false -ErrorAction SilentlyContinue
Unregister-ScheduledTask -TaskName "Trinity Runner Wake" -Confirm:$false -ErrorAction SilentlyContinue
Get-Process -IncludeUserName -ErrorAction SilentlyContinue | Where-Object { $_.UserName -like "*\$Account" } | Stop-Process -Force -ErrorAction SilentlyContinue
& icacls.exe $Repo /remove:d $Account /T /C /Q | Out-Null
$user = Get-LocalUser -Name $Account -ErrorAction SilentlyContinue
if ($user) { try { Set-BatchLogon $user.SID.Value $false } catch { Write-Host $_.Exception.Message -ForegroundColor Yellow } }
Remove-LocalUser -Name $Account -ErrorAction SilentlyContinue
Remove-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList" -Name $Account -ErrorAction SilentlyContinue
Remove-Item -LiteralPath "$RunnerDir\secret" -Recurse -Force -ErrorAction SilentlyContinue

$saved = "$RunnerDir\settings-before-install.txt"   # not in state: the runner can write there, and the uninstaller reads this as administrator
if (Test-Path -LiteralPath $saved) {
    $v = @{}
    foreach ($l in Get-Content -LiteralPath $saved) { $k, $x = $l -split "=", 2; $v[$k] = $x }
    if ($v["sleep_ac_seconds"] -notmatch '^\d+$') { Write-Host "Saved sleep settings unreadable; left as they are."; $v = @{} }
    else {
    & powercfg.exe /setacvalueindex SCHEME_CURRENT SUB_SLEEP STANDBYIDLE $v["sleep_ac_seconds"]
    & powercfg.exe /setacvalueindex SCHEME_CURRENT SUB_SLEEP HIBERNATEIDLE $v["hibernate_ac_seconds"]
    if ($v["rtcwake_ac"] -match '^\d+$') { & powercfg.exe /setacvalueindex SCHEME_CURRENT SUB_SLEEP RTCWAKE $v["rtcwake_ac"] }
    & powercfg.exe /setactive SCHEME_CURRENT
    }
    $au = "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU"
    if ($v["no_auto_reboot"]) { Set-ItemProperty -Path $au -Name NoAutoRebootWithLoggedOnUsers -Value ([int]$v["no_auto_reboot"]) }
    else { Remove-ItemProperty -Path $au -Name NoAutoRebootWithLoggedOnUsers -ErrorAction SilentlyContinue }
    $ux = "HKLM:\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings"
    foreach ($n in @("ActiveHoursStart", "ActiveHoursEnd")) {
        $key = if ($n -eq "ActiveHoursStart") { "active_hours_start" } else { "active_hours_end" }
        if ($v[$key]) { Set-ItemProperty -Path $ux -Name $n -Value ([int]$v[$key]) } else { Remove-ItemProperty -Path $ux -Name $n -ErrorAction SilentlyContinue }
    }
    Write-Host "Sleep and Windows Update settings put back as they were before the install."
} else {
    Write-Host "No saved settings found; sleep and Windows Update settings left as they are." -ForegroundColor Yellow
}
$desktop = [Environment]::GetFolderPath("Desktop")
foreach ($b in @("Approve-Scripts", "Stop-Runner", "Start-Runner", "Runner-Status")) { Remove-Item -LiteralPath (Join-Path $desktop "Trinity $b.bat") -ErrorAction SilentlyContinue }
Write-Host "Runner removed. (C:\TrinityRunner kept for its logs.)" -ForegroundColor Green
Read-Host "Press Enter to close"
