@echo off
rem Dry run of KB16-Network.bat: train 1 superbatch with 16 king buckets, save, load it in the engine. About 5 minutes. Publishes nothing.
rem Use the full path: on some PCs the PowerShell folder is missing from PATH.
rem Bring the Trinity folder up to date first, so the newest scripts run.
git -C "%~dp0..\.." pull --ff-only --quiet
set "PS=%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe"
if not exist "%PS%" set "PS=pwsh.exe"
"%PS%" -NoProfile -ExecutionPolicy Bypass -File "%~dp0kb16.ps1" -DryRun %*
echo.
pause
