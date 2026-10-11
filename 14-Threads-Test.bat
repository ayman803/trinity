@echo off
rem Threads test: Trinity with 4 threads against Trinity with 1 thread, same clock (about 4-5 hours).
rem Use the full path: on some PCs the PowerShell folder is missing from PATH.
rem Bring this folder up to date first, so the newest scripts run.
git -C "%~dp0." pull --ff-only --quiet
set "PS=%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe"
if not exist "%PS%" set "PS=pwsh.exe"
"%PS%" -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\threads.ps1" %*
echo.
pause
