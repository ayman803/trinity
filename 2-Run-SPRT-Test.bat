@echo off
rem Test a new version against main (SPRT). Uses 14 threads by default.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\sprt.ps1" %*
echo.
pause
