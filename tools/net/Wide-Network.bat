@echo off
rem Coder 2: wider network (2048 neurons) trained from scratch on the Leela data, like 11-Big-Network.bat. Runs overnight (about a day).
rem For 1536 neurons: open a Command Prompt in this folder and type  Wide-Network.bat -Hidden 1536
rem Use the full path: on some PCs the PowerShell folder is missing from PATH.
rem Bring the Trinity folder up to date first, so the newest scripts run.
git -C "%~dp0..\.." pull --ff-only --quiet
set "PS=%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe"
if not exist "%PS%" set "PS=pwsh.exe"
"%PS%" -NoProfile -ExecutionPolicy Bypass -File "%~dp0wide.ps1" %*
echo.
pause
