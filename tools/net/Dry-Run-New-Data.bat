@echo off
rem Dry run of New-Data-Network.bat: every stage (convert, shuffle, train, save) on 3 million positions, about 2 minutes. Publishes nothing.
rem Use the full path: on some PCs the PowerShell folder is missing from PATH.
rem Bring the Trinity folder up to date first, so the newest scripts run.
git -C "%~dp0..\.." pull --ff-only --quiet
set "PS=%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe"
if not exist "%PS%" set "PS=pwsh.exe"
"%PS%" -NoProfile -ExecutionPolicy Bypass -File "%~dp0newdata.ps1" -DryRun %*
echo.
pause
