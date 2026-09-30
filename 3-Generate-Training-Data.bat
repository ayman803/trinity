@echo off
rem Generate NNUE training data (self-play).
rem Use the full path: on some PCs the PowerShell folder is missing from PATH.
set "PS=%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe"
if not exist "%PS%" set "PS=pwsh.exe"
"%PS%" -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\datagen.ps1" %*
echo.
pause
