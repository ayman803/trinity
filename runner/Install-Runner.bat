@echo off
rem Install-Runner the Trinity job runner (asks for administrator rights).
rem Bring the Trinity folder up to date first, so the newest runner is installed.
git -C "%~dp0.." pull --ff-only --quiet
if errorlevel 1 (
  echo.
  echo PROBLEM: getting the newest files into the Trinity folder failed, so nothing was installed.
  echo Send Claude a screenshot of this window.
  pause
  exit /b 1
)
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -File "%~dp0Install-Runner.ps1"
