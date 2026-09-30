@echo off
rem Test a new version against main (SPRT). Uses 14 threads by default.
rem Use the full path: on some PCs the PowerShell folder is missing from PATH.
set "PS=%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe"
if not exist "%PS%" set "PS=pwsh.exe"
"%PS%" -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\sprt.ps1" %*
echo.
pause
