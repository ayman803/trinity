@echo off
rem Uninstall-Runner the Trinity job runner (asks for administrator rights).
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -File "%~dp0Uninstall-Runner.ps1"
