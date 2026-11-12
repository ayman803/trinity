@echo off
rem Dry run of resumable training (12 November 2026): trains 3 tiny runs on one data file (uninterrupted twice; stopped after 1 superbatch and resumed), compares them, checks Control2's checkpoint can be read, and loads the resumed network in the engine. About 10 minutes. Publishes nothing.
rem Use the full path: on some PCs the PowerShell folder is missing from PATH.
rem Bring the Trinity folder up to date first, so the newest scripts run.
git -C "%~dp0..\.." pull --ff-only --quiet
set "PS=%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe"
if not exist "%PS%" set "PS=pwsh.exe"
"%PS%" -NoProfile -ExecutionPolicy Bypass -File "%~dp0kb16.ps1" -KingBuckets 8 -DryRun -Resume -From control2 %*
echo.
pause
