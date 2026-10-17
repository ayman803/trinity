@echo off
rem Install-Runner the Trinity job runner (asks for administrator rights).
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -File "%~dp0Install-Runner.ps1"
