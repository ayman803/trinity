@echo off
rem Control run: the 1024 network recipe UNCHANGED (8 king buckets) with a new random start, on the same data as the 16-bucket run. About 5 hours of GPU.
rem Dry run first: Dry-Run-KB16.bat -KingBuckets 8 (or skip it: the 16-bucket dry run already checked every stage)
rem Use the full path: on some PCs the PowerShell folder is missing from PATH.
rem Bring the Trinity folder up to date first, so the newest scripts run.
git -C "%~dp0..\.." pull --ff-only --quiet
set "PS=%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe"
if not exist "%PS%" set "PS=pwsh.exe"
"%PS%" -NoProfile -ExecutionPolicy Bypass -File "%~dp0kb16.ps1" -KingBuckets 8 -Seed 20261018 -Tag control %*
echo.
pause
