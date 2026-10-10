@echo off
rem Calibration at a repeating time control (40 moves in 20 seconds, like CCRL's 40/15 but faster),
rem mainly to count games lost on time. 300 games per opponent, about 2.5 hours.
rem Use the full path: on some PCs the PowerShell folder is missing from PATH.
rem Bring this folder up to date first, so the newest scripts run.
git -C "%~dp0." pull --ff-only --quiet
set "PS=%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe"
if not exist "%PS%" set "PS=pwsh.exe"
"%PS%" -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\calibrate.ps1" -TC 40/20 -Rounds 150 %*
echo.
pause
