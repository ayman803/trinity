@echo off
rem Coder 2: training loss of every saved checkpoint on the same positions (about a minute). Compares the 1024 and the wide run.
rem Use the full path: on some PCs the PowerShell folder is missing from PATH.
rem Bring the Trinity folder up to date first, so the newest scripts run.
git -C "%~dp0..\.." pull --ff-only --quiet
set "PS=%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe"
if not exist "%PS%" set "PS=pwsh.exe"
"%PS%" -NoProfile -ExecutionPolicy Bypass -File "%~dp0netloss.ps1" %*
echo.
pause
