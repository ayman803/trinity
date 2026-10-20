@echo off
rem 16 king buckets, second random start (best of several; weight seed 20261020), same data. About 6-7 hours of GPU.
rem Use the full path: on some PCs the PowerShell folder is missing from PATH.
rem Bring the Trinity folder up to date first, so the newest scripts run.
git -C "%~dp0..\.." pull --ff-only --quiet
set "PS=%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe"
if not exist "%PS%" set "PS=pwsh.exe"
"%PS%" -NoProfile -ExecutionPolicy Bypass -File "%~dp0kb16.ps1" -KingBuckets 16 -Seed 20261020 -Tag kb16s2 %*
echo.
pause
