@echo off
rem Install-Runner the Trinity job runner (asks for administrator rights).
rem Bring the Trinity folder up to date first, so the newest runner is installed.
git -C "%~dp0.." pull --ff-only --quiet
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -File "%~dp0Install-Runner.ps1"
