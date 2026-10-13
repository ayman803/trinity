@echo off
rem Time use (about a minute, no games): how Trinity spends its clock in the calibration games.
rem Use the full path: on some PCs the PowerShell folder is missing from PATH.
rem Bring this folder up to date first, so the newest scripts run.
git -C "%~dp0." pull --ff-only --quiet
set "PS=%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe"
if not exist "%PS%" set "PS=pwsh.exe"
"%PS%" -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\timeuse.ps1" %*
echo.
pause
