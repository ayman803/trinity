@echo off
rem 16 king buckets: the 1024 network recipe unchanged except the king buckets (16 instead of 8). About 5-8 hours of GPU.
rem Dry run first (about 5 minutes): Dry-Run-KB16.bat
rem Use the full path: on some PCs the PowerShell folder is missing from PATH.
rem Bring the Trinity folder up to date first, so the newest scripts run.
git -C "%~dp0..\.." pull --ff-only --quiet
set "PS=%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe"
if not exist "%PS%" set "PS=pwsh.exe"
"%PS%" -NoProfile -ExecutionPolicy Bypass -File "%~dp0kb16.ps1" %*
echo.
pause
