# Remove the Trinity job runner from this machine (Ayman runs it as
# administrator by double-clicking runner\Uninstall-Runner.bat): stops and
# removes the task, deletes the trinity-runner account, lifts its
# read-only rule on C:\Trinity and deletes the stored GitHub key.
# C:\TrinityRunner itself is kept (logs); delete it by hand if wanted.

param([string]$Repo = "C:\Trinity", [string]$RunnerDir = "C:\TrinityRunner")

$principal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`""
    exit
}
$Account = "trinity-runner"
Stop-ScheduledTask -TaskName "Trinity Runner" -ErrorAction SilentlyContinue
Unregister-ScheduledTask -TaskName "Trinity Runner" -Confirm:$false -ErrorAction SilentlyContinue
Get-Process -IncludeUserName -ErrorAction SilentlyContinue | Where-Object { $_.UserName -like "*\$Account" } | Stop-Process -Force -ErrorAction SilentlyContinue
& icacls.exe $Repo /remove:d $Account /C /Q | Out-Null
Remove-LocalUser -Name $Account -ErrorAction SilentlyContinue
Remove-Item -LiteralPath "$RunnerDir\secret" -Recurse -Force -ErrorAction SilentlyContinue
Write-Host "Runner removed. (C:\TrinityRunner kept for its logs.)" -ForegroundColor Green
Read-Host "Press Enter to close"
