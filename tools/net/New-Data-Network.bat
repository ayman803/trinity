@echo off
rem Coder 2: the 1024 network recipe unchanged, plus a second Leela file (put it in C:\Trinity\data\leela2). Overnight.
rem Use the full path: on some PCs the PowerShell folder is missing from PATH.
rem Bring the Trinity folder up to date first, so the newest scripts run.
git -C "%~dp0..\.." pull --ff-only --quiet
set "PS=%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe"
if not exist "%PS%" set "PS=pwsh.exe"
"%PS%" -NoProfile -ExecutionPolicy Bypass -File "%~dp0newdata.ps1" %*
echo.
pause
